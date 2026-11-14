//! A small deterministic travel domain, independent of the train, UI and OS.
//!
//! Feed the train's integrated displacement to [`Journey::advance`]. Arrivals
//! and departures describe crossing a station; they never change train speed.
//! Only caller-supplied active time counts. There is no offline clock or I/O.

use std::collections::{VecDeque, vec_deque};

use super::{
    route::{Biome, DEFAULT_JOURNEY_SEED, Route, RouteSegment, STATION_COUNT, Station, StationId},
    timing::{AdvanceOutcome, active_delta, active_distance},
};

/// The oldest unread event is discarded before inserting a new event at capacity.
pub const MAX_TRAVEL_EVENTS: usize = 64;
/// A cumulative milestone in fictional journey units, available even at idle CPU.
pub const LONG_JOURNEY_DISTANCE_UNITS: f64 = 36_000.0;
// Snapping only sub-nanounit boundary residue avoids frame-dependent delayed
// arrivals. Total integrated distance is never rounded or reset at a station.
const BOUNDARY_TOLERANCE_UNITS: f64 = 1e-9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Achievement {
    FirstStation,
    FirstCircuit,
    LongJourney,
}

/// Session-local flags. Consuming or dropping notifications never erases them.
/// Persistence belongs to the coordinator; this module never reads/writes files.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Achievements {
    pub first_station: bool,
    pub first_circuit: bool,
    pub long_journey: bool,
}

impl Achievements {
    #[must_use]
    pub fn contains(self, achievement: Achievement) -> bool {
        match achievement {
            Achievement::FirstStation => self.first_station,
            Achievement::FirstCircuit => self.first_circuit,
            Achievement::LongJourney => self.long_journey,
        }
    }

    fn insert(&mut self, achievement: Achievement) -> bool {
        let flag = match achievement {
            Achievement::FirstStation => &mut self.first_station,
            Achievement::FirstCircuit => &mut self.first_circuit,
            Achievement::LongJourney => &mut self.long_journey,
        };
        let newly_unlocked = !*flag;
        *flag = true;
        newly_unlocked
    }
}

/// Logical event positions are in cumulative journey units, not wall-clock
/// timestamps. Delivery happens on the frame crossing that position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TravelEvent {
    StationArrived {
        station: StationId,
        visit: u64,
        distance_units: f64,
    },
    StationDeparted {
        station: StationId,
        distance_units: f64,
    },
    CircuitCompleted {
        completed_laps: u64,
        distance_units: f64,
    },
    AchievementUnlocked {
        achievement: Achievement,
        distance_units: f64,
    },
}

/// A read-only value for UI/rendering, with no borrowed mutable state.
///
/// Progress is cyclic: it returns to zero on completing a circuit. Cumulative
/// distance, active time, visits and achievements do not reset. This is a view,
/// not a versioned save-file format or an unchecked restoration API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JourneySnapshot {
    pub seed: u64,
    pub route_name: &'static str,
    pub route_length_units: f64,
    pub total_distance_units: f64,
    pub route_position_units: f64,
    pub active_seconds: f64,
    pub route_segment: RouteSegment,
    /// Content hint for the outgoing segment after the next station. This lets
    /// presentation blend the actual route without reconstructing its rules.
    pub next_biome_hint: Biome,
    pub next_station: Station,
    pub distance_to_next_station_units: f64,
    pub segment_progress: f64,
    pub route_progress: f64,
    pub at_station: Option<StationId>,
    pub completed_laps: u64,
    pub stations_visited: u64,
    pub discovered_stations: [bool; STATION_COUNT],
    pub achievements: Achievements,
    pub pending_events: usize,
    pub dropped_events: u64,
    pub paused: bool,
}

/// Constant-size progression plus a bounded, explicitly consumed event queue.
#[derive(Clone, Debug)]
pub struct Journey {
    route: Route,
    total_distance: PositiveSum,
    active_seconds: PositiveSum,
    segment_distance: PositiveSum,
    segment_index: usize,
    completed_laps: u64,
    stations_visited: u64,
    at_station: Option<StationId>,
    at_station_distance_units: f64,
    discovered_stations: [bool; STATION_COUNT],
    achievements: Achievements,
    events: VecDeque<TravelEvent>,
    dropped_events: u64,
    paused: bool,
}

impl Default for Journey {
    fn default() -> Self {
        Self::from_seed(DEFAULT_JOURNEY_SEED)
    }
}

impl Journey {
    /// Restore in constant time; milestones are derived, never replayed as rewards.
    pub fn restore_checkpoint(seed: u64, distance: f64, seconds: f64) -> Result<Self, String> {
        if !distance.is_finite()
            || !(0.0..=1e12).contains(&distance)
            || !seconds.is_finite()
            || !(0.0..=1e12).contains(&seconds)
        {
            return Err("Contadores inválidos na jornada; arquivo preservado.".into());
        }
        let mut journey = Self::from_seed(seed);
        let length = journey.route.length_units();
        journey.completed_laps = ((distance + BOUNDARY_TOLERANCE_UNITS) / length).floor() as u64;
        let mut remainder = (distance - journey.completed_laps as f64 * length).max(0.0);
        for index in 0..STATION_COUNT {
            let segment = journey.route.segments()[index];
            journey.segment_index = index;
            if remainder + BOUNDARY_TOLERANCE_UNITS < segment.length_units {
                break;
            }
            remainder = (remainder - segment.length_units).max(0.0);
        }
        journey.total_distance.add(distance);
        journey.active_seconds.add(seconds);
        journey.segment_distance.add(remainder);
        journey.stations_visited =
            journey.completed_laps * STATION_COUNT as u64 + journey.segment_index as u64;
        for index in 0..STATION_COUNT {
            journey.discovered_stations[index] =
                journey.completed_laps > 0 || index <= journey.segment_index;
        }
        journey.achievements = Achievements {
            first_station: journey.stations_visited > 0,
            first_circuit: journey.completed_laps > 0,
            long_journey: distance >= LONG_JOURNEY_DISTANCE_UNITS,
        };
        journey.at_station = (remainder < BOUNDARY_TOLERANCE_UNITS)
            .then_some(journey.route.segments()[journey.segment_index].from);
        journey.at_station_distance_units = distance - remainder;
        Ok(journey)
    }

    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self::new(Route::from_seed(seed))
    }

    #[must_use]
    pub fn new(route: Route) -> Self {
        let origin = route.stations()[0].id;
        let mut discovered_stations = [false; STATION_COUNT];
        discovered_stations[origin.index()] = true;
        Self {
            route,
            total_distance: PositiveSum::default(),
            active_seconds: PositiveSum::default(),
            segment_distance: PositiveSum::default(),
            segment_index: 0,
            completed_laps: 0,
            stations_visited: 0,
            at_station: Some(origin),
            at_station_distance_units: 0.0,
            discovered_stations,
            achievements: Achievements::default(),
            events: VecDeque::with_capacity(MAX_TRAVEL_EVENTS),
            dropped_events: 0,
            paused: false,
        }
    }

    #[must_use]
    pub fn route(&self) -> &Route {
        &self.route
    }

    /// Advance by a measured, integrated scene displacement (1 pixel = 1
    /// fictional unit by convention). Accepted time steps are in `(0, 1]` s.
    ///
    /// A gap over 1 s, NaN/infinity, negative inputs or an impossible distance
    /// discard the whole step. No teleport, catch-up events, or offline reward
    /// is generated. Pausing freezes both counters and the queue. Zero distance
    /// with valid active time counts that time but cannot arrive or depart.
    ///
    /// The maximum legal step is shorter than every route segment, so at most
    /// one station can be crossed per call. Work never scales with a sleep gap.
    pub fn advance(&mut self, dt_seconds: f64, distance_units: f64) -> AdvanceOutcome {
        if self.paused {
            return AdvanceOutcome::Paused;
        }
        let dt = match active_delta(dt_seconds) {
            Ok(dt) => dt,
            Err(outcome) => return outcome,
        };
        let distance = match active_distance(dt, distance_units) {
            Ok(distance) => distance,
            Err(outcome) => return outcome,
        };
        self.active_seconds.add(dt);
        if distance > 0.0 {
            self.advance_distance(distance);
        }
        AdvanceOutcome::Advanced {
            seconds: dt,
            distance_units: distance,
        }
    }

    fn advance_distance(&mut self, distance: f64) {
        let old_total = self.total_distance.value;
        self.depart_if_at_station();

        let segment = self.route.segments()[self.segment_index];
        let to_station = segment.length_units - self.segment_distance.value;
        self.total_distance.add(distance);
        self.segment_distance.add(distance);

        if self.segment_distance.value + BOUNDARY_TOLERANCE_UNITS >= segment.length_units {
            let arrival_distance = (old_total + to_station).min(f64::MAX);
            // If a cumulative milestone occurs before the station within this
            // frame, its event precedes the arrival in logical distance order.
            self.unlock_long_journey_until(arrival_distance);
            let remainder = (self.segment_distance.value - segment.length_units).max(0.0);
            self.segment_index = (self.segment_index + 1) % STATION_COUNT;
            self.segment_distance = PositiveSum::default();
            if remainder > BOUNDARY_TOLERANCE_UNITS {
                self.segment_distance.add(remainder);
            }
            self.at_station = Some(segment.to);
            self.at_station_distance_units = arrival_distance;
            self.stations_visited = self.stations_visited.saturating_add(1);
            self.discovered_stations[segment.to.index()] = true;
            self.push_event(TravelEvent::StationArrived {
                station: segment.to,
                visit: self.stations_visited,
                distance_units: arrival_distance,
            });
            self.unlock(Achievement::FirstStation, arrival_distance);

            if self.segment_index == 0 {
                self.completed_laps = self.completed_laps.saturating_add(1);
                self.push_event(TravelEvent::CircuitCompleted {
                    completed_laps: self.completed_laps,
                    distance_units: arrival_distance,
                });
                self.unlock(Achievement::FirstCircuit, arrival_distance);
            }
            // Arrival/departure are visual cues, not a forced dwell. If the
            // step ends exactly at the station, departure waits for movement.
            if self.segment_distance.value > 0.0 {
                self.depart_if_at_station();
            }
        }
        self.unlock_long_journey_until(self.total_distance.value);
    }

    fn depart_if_at_station(&mut self) {
        if let Some(station) = self.at_station.take() {
            self.push_event(TravelEvent::StationDeparted {
                station,
                // Preserve the logical arrival position even if a tiny
                // boundary residue was snapped without rounding the total.
                distance_units: self.at_station_distance_units,
            });
        }
    }

    fn unlock_long_journey_until(&mut self, distance: f64) {
        if distance >= LONG_JOURNEY_DISTANCE_UNITS {
            self.unlock(Achievement::LongJourney, LONG_JOURNEY_DISTANCE_UNITS);
        }
    }

    fn unlock(&mut self, achievement: Achievement, distance_units: f64) {
        if self.achievements.insert(achievement) {
            self.push_event(TravelEvent::AchievementUnlocked {
                achievement,
                distance_units,
            });
        }
    }

    fn push_event(&mut self, event: TravelEvent) {
        if self.events.len() == MAX_TRAVEL_EVENTS {
            self.events.pop_front();
            self.dropped_events = self.dropped_events.saturating_add(1);
        }
        self.events.push_back(event);
    }

    #[must_use]
    pub fn snapshot(&self) -> JourneySnapshot {
        let segment = self.route.segments()[self.segment_index];
        let position = self.route.station(segment.from).distance_from_origin_units
            + self.segment_distance.value;
        JourneySnapshot {
            seed: self.route.seed(),
            route_name: self.route.name(),
            route_length_units: self.route.length_units(),
            total_distance_units: self.total_distance.value,
            route_position_units: position,
            active_seconds: self.active_seconds.value,
            route_segment: segment,
            next_biome_hint: self.route.segments()[(self.segment_index + 1) % STATION_COUNT]
                .biome_hint,
            next_station: *self.route.station(segment.to),
            distance_to_next_station_units: (segment.length_units - self.segment_distance.value)
                .max(0.0),
            segment_progress: (self.segment_distance.value / segment.length_units).clamp(0.0, 1.0),
            route_progress: (position / self.route.length_units()).clamp(0.0, 1.0),
            at_station: self.at_station,
            completed_laps: self.completed_laps,
            stations_visited: self.stations_visited,
            discovered_stations: self.discovered_stations,
            achievements: self.achievements,
            pending_events: self.events.len(),
            dropped_events: self.dropped_events,
            paused: self.paused,
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    #[must_use]
    pub fn pending_event_count(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn dropped_event_count(&self) -> u64 {
        self.dropped_events
    }

    pub fn pop_event(&mut self) -> Option<TravelEvent> {
        self.events.pop_front()
    }

    /// Removes every queued event, including unconsumed items if this iterator
    /// is dropped early. Use `pop_event` for a limited per-frame event budget.
    pub fn drain_events(&mut self) -> vec_deque::Drain<'_, TravelEvent> {
        self.events.drain(..)
    }
}

/// Compensated positive accumulation limits frame-count drift. A cyclic
/// segment accumulator stays small even after the lifetime total becomes huge.
#[derive(Clone, Copy, Debug, Default)]
struct PositiveSum {
    value: f64,
    correction: f64,
}

impl PositiveSum {
    fn add(&mut self, increment: f64) {
        let adjusted = increment - self.correction;
        let next = self.value + adjusted;
        if next.is_finite() {
            self.correction = (next - self.value) - adjusted;
            self.value = next.max(self.value);
        } else {
            self.value = f64::MAX;
            self.correction = 0.0;
        }
    }
}
