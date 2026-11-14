use crate::config::Config;

use super::{
    AdvanceOutcome, DEFAULT_JOURNEY_SEED, Journey, JourneySnapshot, TRAVEL_UNITS_PER_SCENE_PIXEL,
    Train, TravelEvent, timing::active_delta,
};

/// A common multiple of the rendering layer periods, including their parallax.
/// Keep layer periods compatible with this value to avoid a seam on wrapping.
pub const SCROLL_PERIOD_PX: f64 = 184_320.0;
pub const MAX_SMOKE_PARTICLES: usize = 48;
const MAX_SMOKE_LIFETIME: f64 = 3.4;

/// Coordinates are relative to the chimney, in pixels of the 640 × 140 scene.
/// Negative x moves behind the train; negative y rises into the sky.
#[derive(Clone, Debug)]
pub struct SmokeParticle {
    pub x: f32,
    pub y: f32,
    pub age: f32,
    pub lifetime: f32,
    /// Current visual radius in scene pixels.
    pub size: f32,
    drift_speed: f32,
    rise_speed: f32,
}

impl SmokeParticle {
    fn advance(&mut self, seconds: f32, ground_distance: f32) {
        self.age += seconds;
        self.x -= ground_distance * 0.30 + self.drift_speed * seconds;
        self.y -= self.rise_speed * seconds;
        self.size += seconds * 0.65;
    }
}

/// Small, bounded scene state independent of egui, sysinfo and the operating OS.
#[derive(Clone, Debug)]
pub struct World {
    station_wait: f64,
    elapsed_seconds: f64,
    scroll_px: f64,
    smoke: Vec<SmokeParticle>,
    spawn_fraction: f64,
    random_state: u64,
    journey: Journey,
    service: super::ServiceState,
}

impl Default for World {
    fn default() -> Self {
        Self::with_seed(DEFAULT_JOURNEY_SEED)
    }
}

impl World {
    pub fn service_snapshot(&self) -> &super::ServiceState { &self.service }
    pub fn set_service_composition(&mut self, composition: [super::ServiceWagon;3]) -> Result<(),String> { self.service.set_composition(composition) }
    pub fn restore_service(&mut self, service: super::ServiceState) -> Result<(),String> { service.validate(self.journey_snapshot())?; self.service=service; Ok(()) }

    pub fn station_wait(&self) -> f64 {
        self.station_wait
    }

    pub fn set_station_wait(&mut self, seconds: f64) {
        self.station_wait = if seconds.is_finite() {
            seconds.clamp(0.0, 8.0)
        } else {
            0.0
        };
    }

    /// Product journey: a fixed timetable, independent of CPU and rendering cadence.
    /// Arrival includes eight active seconds of service. Legacy adapters remain
    /// available for renderer exports and mathematical comparisons.
    pub fn advance_expedition(
        &mut self,
        train: &mut Train,
        cpu: f32,
        seconds: f64,
        config: &Config,
    ) -> AdvanceOutcome {
        if self.is_paused() {
            return AdvanceOutcome::Paused;
        }
        let dt = match active_delta(seconds) {
            Ok(dt) => dt,
            Err(outcome) => return outcome,
        };
        let mut remaining = dt;
        let initial_distance = self.journey.snapshot().total_distance_units;
        // At most a departure and an arrival can occur within a <=1 second step.
        while remaining > 1e-10 {
            if self.station_wait > 0.0 {
                let step = remaining.min(self.station_wait);
                train.stop_at_station();
                self.advance_scene_with_route(step, 0.0, 0.0, 0.0);
                self.station_wait = (self.station_wait - step).max(0.0);
                remaining -= step;
            } else {
                let to_station = self.journey.snapshot().distance_to_next_station_units;
                let step = remaining.min(to_station / 24.0);
                if step <= 0.0 {
                    break;
                }
                let mut approach = config.clone();
                let braking = (to_station / 120.0).clamp(0.05, 1.0);
                approach.min_speed_kmh *= braking;
                approach.max_speed_kmh *= braking;
                let distance = train.update(cpu, step, &approach);
                let previous_visits = self.journey.snapshot().stations_visited;
                self.advance_scene_with_route(step, distance, train.speed_kmh(), step * 24.0);
                if self.journey.snapshot().stations_visited > previous_visits {
                    self.service.arrive(self.journey.snapshot());
                    self.station_wait = 8.0;
                    train.stop_at_station();
                }
                remaining -= step;
            }
        }
        AdvanceOutcome::Advanced {
            seconds: dt,
            distance_units: self.journey.snapshot().total_distance_units - initial_distance,
        }
    }

    pub fn restore_journey(&mut self, journey: Journey) {
        let snapshot = journey.snapshot();
        self.elapsed_seconds = snapshot.active_seconds;
        self.scroll_px =
            (snapshot.total_distance_units / TRAVEL_UNITS_PER_SCENE_PIXEL) % SCROLL_PERIOD_PX;
        self.service = super::ServiceState::migrated(snapshot);
        self.journey = journey;
        self.smoke.clear();
    }
    /// Same baseline scenery with a reproducible route and smoke seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            station_wait: 0.0,
            elapsed_seconds: 0.0,
            scroll_px: 0.0,
            smoke: Vec::with_capacity(MAX_SMOKE_PARTICLES),
            spawn_fraction: 0.0,
            random_state: seed,
            journey: Journey::from_seed(seed),
            service: super::ServiceState::migrated(Journey::from_seed(seed).snapshot()),
        }
    }

    /// Advance scenery and smoke without allocating an unbounded history.
    ///
    /// `distance_px` comes from the train's analytical speed integral. The same
    /// accepted displacement also advances the journey. Paused, invalid or
    /// suspended steps leave scenery and journey unchanged. Call `advance_train`
    /// when the caller also needs to freeze the train during a pause.
    pub fn update(&mut self, dt_seconds: f64, distance_px: f64, speed_kmh: f64) {
        self.advance_scene(dt_seconds, distance_px, speed_kmh);
    }

    /// Optional integration adapter: advance train, scenery and journey as one
    /// active step, or freeze all three on pause/suspension/invalid elapsed time.
    /// Supply the raw elapsed delta so a sleep interval is distinguishable from
    /// an ordinary frame. This adapter performs no waiting and measures no CPU.
    pub fn advance_train(
        &mut self,
        train: &mut Train,
        cpu_percent: f32,
        dt_seconds: f64,
        config: &Config,
    ) -> AdvanceOutcome {
        if self.is_paused() {
            return AdvanceOutcome::Paused;
        }
        let dt = match active_delta(dt_seconds) {
            Ok(dt) => dt,
            Err(outcome) => return outcome,
        };
        // Train is only three scalars. Commit it only after the shared input
        // checks accept the resulting displacement, including at tiny deltas.
        let mut next_train = train.clone();
        let distance = next_train.update(cpu_percent, dt, config);
        let outcome = self.advance_scene(dt, distance, next_train.speed_kmh());
        if matches!(outcome, AdvanceOutcome::Advanced { .. }) {
            *train = next_train;
        }
        outcome
    }

    fn advance_scene(
        &mut self,
        dt_seconds: f64,
        distance_px: f64,
        speed_kmh: f64,
    ) -> AdvanceOutcome {
        self.advance_scene_with_route(
            dt_seconds,
            distance_px,
            speed_kmh,
            distance_px * TRAVEL_UNITS_PER_SCENE_PIXEL,
        )
    }

    fn advance_scene_with_route(
        &mut self,
        dt_seconds: f64,
        distance_px: f64,
        speed_kmh: f64,
        route_distance: f64,
    ) -> AdvanceOutcome {
        let outcome = self.journey.advance(dt_seconds, route_distance);
        let AdvanceOutcome::Advanced { seconds: dt, .. } = outcome else {
            return outcome;
        };
        let distance = distance_px;
        self.elapsed_seconds = (self.elapsed_seconds + dt).min(f64::MAX);
        self.scroll_px =
            (self.scroll_px + distance.rem_euclid(SCROLL_PERIOD_PX)).rem_euclid(SCROLL_PERIOD_PX);

        if dt >= MAX_SMOKE_LIFETIME {
            self.smoke.clear();
        } else {
            let smoke_distance = distance.min(10_000.0) as f32;
            for particle in &mut self.smoke {
                particle.advance(dt as f32, smoke_distance);
            }
            self.smoke
                .retain(|particle| particle.age < particle.lifetime);
        }

        let speed = if speed_kmh.is_finite() {
            speed_kmh.clamp(0.0, 500.0)
        } else {
            0.0
        };
        let particles_per_second = (1.8 + speed * 0.075).min(12.0);
        let accumulated = self.spawn_fraction + dt * particles_per_second;
        let spawn_count = accumulated.floor().min(MAX_SMOKE_PARTICLES as f64) as usize;
        self.spawn_fraction = if accumulated.is_finite() {
            accumulated.fract()
        } else {
            0.0
        };

        // Emit at the correct point within this frame instead of bunching every
        // new puff at age zero. At most 48 iterations, including after suspension.
        for index in 0..spawn_count {
            let age =
                (self.spawn_fraction + (spawn_count - index - 1) as f64) / particles_per_second;
            if age >= MAX_SMOKE_LIFETIME || self.smoke.len() >= MAX_SMOKE_PARTICLES {
                continue;
            }
            let mut particle = self.new_particle();
            let ground_distance = (distance * (age / dt)).min(10_000.0) as f32;
            particle.advance(age as f32, ground_distance);
            if particle.age < particle.lifetime {
                self.smoke.push(particle);
            }
        }
        outcome
    }

    #[must_use]
    pub fn journey(&self) -> &Journey {
        &self.journey
    }

    #[must_use]
    pub fn journey_snapshot(&self) -> JourneySnapshot {
        self.journey.snapshot()
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.journey.set_paused(paused);
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.journey.is_paused()
    }

    /// Consume all pending visual events. Dropping the drain removes its tail;
    /// use `pop_travel_event` if only a fixed number should be handled this frame.
    pub fn drain_travel_events(&mut self) -> std::collections::vec_deque::Drain<'_, TravelEvent> {
        self.journey.drain_events()
    }

    pub fn pop_travel_event(&mut self) -> Option<TravelEvent> {
        self.journey.pop_event()
    }

    fn random_unit(&mut self) -> f32 {
        // Deterministic visual variation; no randomness dependency or I/O.
        self.random_state = self
            .random_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.random_state >> 40) as u32) as f32 / 16_777_216.0
    }

    fn new_particle(&mut self) -> SmokeParticle {
        SmokeParticle {
            x: 0.0,
            y: 0.0,
            age: 0.0,
            lifetime: 2.4 + self.random_unit(),
            size: 1.5 + self.random_unit() * 1.6,
            drift_speed: 5.0 + self.random_unit() * 3.0,
            rise_speed: 7.0 + self.random_unit() * 4.0,
        }
    }

    #[must_use]
    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }

    #[must_use]
    pub fn scroll_px(&self) -> f64 {
        self.scroll_px
    }

    #[must_use]
    pub fn smoke(&self) -> &[SmokeParticle] {
        &self.smoke
    }
}
