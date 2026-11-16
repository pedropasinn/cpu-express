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
    economy: super::economy::EconomyState,
    machine_bonus_millis: u32,
    pub(crate) lines: std::collections::BTreeMap<String, LineProgress>,
}

/// Per-line progress uses catalogue station IDs so E3 can retain any itinerary.
/// Services are global and line switching must leave no passengers or contract aboard.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineProgress {
    pub seed: u64,
    pub distance: f64,
    pub active_seconds: f64,
    pub station_wait: f64,
    pub stations_visited: u64,
    pub completed_runs: u64,
    pub discovered_stations: std::collections::BTreeSet<String>,
}

impl Default for World {
    fn default() -> Self {
        Self::with_seed(DEFAULT_JOURNEY_SEED)
    }
}

impl World {
    pub fn economy(&self) -> &super::economy::EconomyState {
        &self.economy
    }
    pub fn set_machine_bonus(&mut self, multiplier: f32) {
        self.machine_bonus_millis = if multiplier.is_finite() {
            (multiplier.clamp(1.0, 1.3) * 1000.0).round() as u32
        } else {
            1000
        };
    }
    pub fn machine_bonus(&self) -> f32 {
        self.machine_bonus_millis as f32 / 1000.0
    }
    pub fn line_progress(&self) -> std::collections::BTreeMap<String, LineProgress> {
        let mut lines = self.lines.clone();
        let snapshot = self.journey_snapshot();
        lines.insert(
            super::route::CIRCUIT_LINE_ID.into(),
            LineProgress {
                seed: snapshot.seed,
                distance: snapshot.total_distance_units,
                active_seconds: snapshot.active_seconds,
                station_wait: self.station_wait,
                stations_visited: snapshot.stations_visited,
                completed_runs: snapshot.completed_laps,
                discovered_stations: self
                    .journey
                    .route()
                    .stations()
                    .iter()
                    .filter(|s| snapshot.discovered_stations[s.id.index()])
                    .map(|s| {
                        // Stable catalogue IDs, not translated display names.
                        ["aurora", "pines", "ridge", "harbor"][s.id.index()].to_owned()
                    })
                    .collect(),
            },
        );
        lines
    }
    pub(crate) fn restore_economy(
        &mut self,
        economy: super::economy::EconomyState,
    ) -> Result<(), String> {
        economy.validate()?;
        if self
            .service
            .composition
            .iter()
            .any(|w| !economy.owns(&super::economy::ItemId::wagon(*w)))
        {
            return Err("Composição sem posse; arquivo preservado.".into());
        }
        self.economy = economy;
        self.sync_owned_liveries();
        Ok(())
    }
    pub(crate) fn migrate_economy(&mut self) {
        self.economy =
            super::economy::EconomyState::migrated(self.journey_snapshot(), &self.service);
        self.sync_owned_liveries();
    }
    fn sync_owned_liveries(&mut self) {
        self.service.owned_rewards = super::objectives::Livery::ALL
            .into_iter()
            .filter(|l| self.economy.owns(&super::economy::ItemId::livery(*l)))
            .collect();
    }
    /// Stage all domain changes; only publish after the writer succeeds.
    /// The writer must atomically persist the entire candidate. Err means no disk commit.
    pub fn transact_economy<T>(
        &mut self,
        change: impl FnOnce(&mut super::economy::EconomyState) -> Result<T, String>,
        persist: impl FnOnce(&World) -> Result<(), String>,
    ) -> Result<T, String> {
        let mut candidate = self.clone();
        let result = change(&mut candidate.economy)?;
        candidate.economy.validate()?;
        candidate.sync_owned_liveries();
        persist(&candidate)?;
        *self = candidate;
        Ok(result)
    }
    pub fn economy_action(
        &mut self,
        action: super::economy::EconomyAction,
        persist: impl FnOnce(&World) -> Result<(), String>,
    ) -> Result<(), String> {
        self.transact_economy(|state| state.apply(action), persist)
    }
    /// Clone only on a possible arrival, never on an ordinary rendering frame.
    pub fn may_arrive(&self, seconds: f64) -> bool {
        seconds.is_finite()
            && seconds > 0.0
            && !self.is_paused()
            && self.service.branch.current.is_none()
            && self.journey_snapshot().distance_to_next_station_units
                <= seconds.min(1.0) * 24.0 + 1e-6
    }
    pub fn excursion_snapshot(&self) -> &super::BranchState {
        &self.service.branch
    }
    pub fn request_excursion(&mut self, kind: super::BranchKind) -> Result<(), String> {
        self.service.branch.request(kind, self.journey.snapshot())?;
        self.service
            .branch
            .try_depart(self.journey.snapshot(), self.station_wait);
        Ok(())
    }

    pub fn service_state(&self) -> &super::ServiceState {
        &self.service
    }
    pub fn service_snapshot(&self) -> &super::ServiceState {
        &self.service
    }
    fn service_platform(&self) -> Option<super::StationId> {
        if self.station_wait > 0.0 && self.service.branch.current.is_none() {
            self.journey.snapshot().at_station
        } else {
            None
        }
    }
    pub fn set_automatic_contracts(&mut self, automatic: bool) {
        self.service.automatic_contracts = automatic;
    }
    pub fn accept_cargo_contract(&mut self) -> Result<(), String> {
        self.service.accept_contract(self.service_platform())
    }
    pub fn cancel_cargo_contract(&mut self) -> Result<(), String> {
        self.service.cancel_contract(self.service_platform())
    }
    pub fn set_service_composition(
        &mut self,
        composition: [super::ServiceWagon; 3],
    ) -> Result<(), String> {
        if let Some(wagon) = composition
            .iter()
            .find(|w| !self.economy.owns(&super::economy::ItemId::wagon(**w)))
        {
            return Err(format!("Você ainda não possui o modelo {}.", wagon.name()));
        }
        self.service.set_owned_composition(composition)
    }
    pub fn restore_service(&mut self, service: super::ServiceState) -> Result<(), String> {
        service.validate(self.journey_snapshot())?;
        if service.branch.current.is_some() && self.station_wait > 0.0 {
            return Err("Excursão e serviço na base simultâneos.".into());
        }
        self.service = service;
        Ok(())
    }

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
            self.service
                .branch
                .try_depart(self.journey.snapshot(), self.station_wait);
            if let Some(excursion) = self.service.branch.current {
                let step = remaining.min(excursion.phase.duration() - excursion.phase_seconds);
                let distance = if excursion.phase == super::BranchPhase::Service {
                    train.stop_at_station();
                    0.0
                } else {
                    train.update(cpu, step, config)
                };
                self.advance_scene_with_route(step, distance, train.speed_kmh(), 0.0);
                self.service.branch.advance(step);
                remaining -= step;
            } else if self.station_wait > 0.0 {
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
                let braking = (to_station / 120.0).clamp(0.05, 1.0);
                let distance = train.update_with_braking(cpu, step, config, braking);
                let previous_visits = self.journey.snapshot().stations_visited;
                self.advance_scene_with_route(step, distance, train.speed_kmh(), step * 24.0);
                if self.journey.snapshot().stations_visited > previous_visits {
                    self.service.arrive(self.journey.snapshot());
                    self.economy.arrive(
                        self.journey.snapshot(),
                        &self.service,
                        self.machine_bonus_millis,
                    );
                    self.sync_owned_liveries();
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
        self.economy = super::economy::EconomyState::migrated(snapshot, &self.service);
        self.sync_owned_liveries();
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
            service: {
                let mut service =
                    super::ServiceState::migrated(Journey::from_seed(seed).snapshot());
                service.composition = [
                    super::ServiceWagon::Passenger,
                    super::ServiceWagon::Passenger,
                    super::ServiceWagon::Freight,
                ];
                service
            },
            economy: super::economy::EconomyState::new(seed),
            machine_bonus_millis: 1000,
            lines: Default::default(),
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
