//! Generic journey/service domain for integration, not a second application save format.
//! Drafts can be exercised by tools; the product must use `playable` and persist
//! a global transaction before exposing a line switch. No economy is paid here.
use super::{
    AdvanceOutcome, BranchState, ContractStatus, ServiceState, ServiceWagon,
    catalog::{ArchitectureStyle, ContentKind, Layout, LineDef},
    itinerary::{self, Itinerary, Position},
    map_layout::MapLayout,
    objectives::Livery,
    timing::active_delta,
    world::LineProgress,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Stable qualified identity; neither display names nor ordinals are saved.
/// Legacy Aurora ServiceState keeps StationId and its exact JSON representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogStationId {
    pub line: String,
    pub station: String,
}

#[derive(Clone, Debug)]
pub struct LineSession {
    itinerary: Itinerary,
    stations: Vec<CatalogStationId>,
    dwell: Vec<f64>,
    progress: LineProgress,
    service: ServiceState<CatalogStationId>,
    paused: bool,
}

impl LineSession {
    pub const SERVICE_KIND: ContentKind = ContentKind::Fictional;
    pub const SERVICE_NOTICE: &str =
        "Os serviços mostrados são do jogo e não representam a operação real.";
    pub const SPEED: f64 = 24.0;

    /// Product gate: draft lines remain unavailable.
    pub fn playable(line: &LineDef, seed: u64) -> Result<Self, String> {
        let mut itinerary = itinerary::playable(line, itinerary::DEFAULT_UNITS_PER_KM)?;
        if line.id == super::route::CIRCUIT_LINE_ID {
            itinerary = Itinerary::from_lengths(
                &line.id,
                line.layout,
                super::Route::from_seed(seed)
                    .segments()
                    .iter()
                    .map(|s| s.length_units)
                    .collect(),
            )?;
        }
        Self::for_inspection(line, itinerary, seed)
    }

    /// Explicit tool path; constructing this value does not enable a draft in World.
    /// Aurora callers must supply seeded lengths to preserve aurora-v1 distances.
    pub fn for_inspection(line: &LineDef, itinerary: Itinerary, seed: u64) -> Result<Self, String> {
        MapLayout::new(line, &itinerary)?;
        if (0..line.segments.len()).any(|i| {
            itinerary
                .segment_length(i)
                .is_none_or(|length| length < Self::SPEED)
        }) {
            return Err("Trecho curto demais para o horário de inspeção.".into());
        }
        let stations: Vec<_> = line
            .stations
            .iter()
            .filter(|s| !s.retired)
            .map(|s| CatalogStationId {
                line: line.id.clone(),
                station: s.id.clone(),
            })
            .collect();
        let dwell = line
            .stations
            .iter()
            .filter(|s| !s.retired)
            .map(|s| {
                if line.id != super::route::CIRCUIT_LINE_ID
                    && s.architecture.as_ref().is_some_and(|a| {
                        matches!(
                            a.style,
                            ArchitectureStyle::TimberHalt | ArchitectureStyle::ServicePost
                        )
                    })
                {
                    4.0
                } else {
                    8.0
                }
            })
            .collect();
        let progress = LineProgress {
            seed,
            distance: 0.0,
            active_seconds: 0.0,
            station_wait: 0.0,
            stations_visited: 0,
            completed_runs: 0,
            discovered_stations: BTreeSet::from([stations[0].station.clone()]),
        };
        let service = ServiceState {
            branch: BranchState::default(),
            composition: [
                ServiceWagon::Passenger,
                ServiceWagon::Passenger,
                ServiceWagon::Freight,
            ],
            passengers: Vec::new(),
            automatic_contracts: true,
            contract: None,
            last_processed_visit: 0,
            passengers_boarded: 0,
            passengers_delivered: 0,
            cargo_delivered: 0,
            contracts_completed: 0,
            credits: 0,
            owned_rewards: vec![Livery::Aurora],
            collection: Vec::new(),
            arrivals: Vec::new(),
        };
        Ok(Self {
            itinerary,
            stations,
            dwell,
            progress,
            service,
            paused: false,
        })
    }

    pub fn itinerary(&self) -> &Itinerary {
        &self.itinerary
    }
    pub fn progress(&self) -> &LineProgress {
        &self.progress
    }
    pub fn service(&self) -> &ServiceState<CatalogStationId> {
        &self.service
    }
    pub fn position(&self) -> Position {
        self.itinerary.locate(self.progress.distance)
    }
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    pub fn set_automatic_contracts(&mut self, automatic: bool) {
        self.service.automatic_contracts = automatic;
    }
    pub fn accept_contract(&mut self) -> Result<(), String> {
        self.service.accept_contract(self.platform())
    }
    pub fn cancel_contract(&mut self) -> Result<(), String> {
        self.service.cancel_contract(self.platform())
    }
    fn platform(&self) -> Option<CatalogStationId> {
        self.position().at_station.map(|i| self.stations[i].clone())
    }

    /// Eligibility only; neither resets services nor transfers global counters.
    pub fn can_leave_line(&self) -> bool {
        let p = self.position();
        p.at_station.is_some_and(|s| {
            self.itinerary.layout() == Layout::Circuit || s == 0 || s + 1 == self.stations.len()
        }) && self.service.passenger_count() == 0
            && self.service.cargo_count() == 0
            && self.service.contract.as_ref().is_none_or(|c| {
                matches!(
                    c.status,
                    ContractStatus::Offered | ContractStatus::Delivered | ContractStatus::Cancelled
                )
            })
            && self.service.branch == BranchState::default()
    }

    /// Applies at most one arrival per active step. Wait consumes the same clock.
    pub fn advance(&mut self, seconds: f64) -> AdvanceOutcome {
        if self.paused {
            return AdvanceOutcome::Paused;
        }
        let dt = match active_delta(seconds) {
            Ok(dt) => dt,
            Err(error) => return error,
        };
        let waiting = dt.min(self.progress.station_wait);
        self.progress.station_wait -= waiting;
        let moving = dt - waiting;
        // Match locate's station tolerance, but derive the next boundary from
        // the last committed arrival. A rounded position must never skip service.
        let visit = self.progress.stations_visited + 1;
        let boundary = self.itinerary.distance_at_visit(visit);
        let remaining = (boundary - self.progress.distance).max(0.0);
        let arrived = moving > 0.0 && moving * Self::SPEED + 1e-6 >= remaining;
        let distance = if arrived {
            remaining
        } else {
            moving * Self::SPEED
        };
        self.progress.distance += distance;
        self.progress.active_seconds += dt;
        if arrived {
            self.progress.distance = boundary;
            let station = self.itinerary.station_at_visit(visit);
            let entropy = self
                .progress
                .seed
                .wrapping_add(visit.wrapping_mul(0x9e3779b97f4a7c15));
            let destination = (station + 1 + (entropy % (self.stations.len() as u64 - 1)) as usize)
                % self.stations.len();
            self.service.arrive_at(
                self.progress.seed,
                visit,
                self.stations[station].clone(),
                self.stations[destination].clone(),
            );
            self.progress.station_wait =
                (self.dwell[station] - (moving - distance / Self::SPEED).max(0.0)).max(0.0);
            self.progress
                .discovered_stations
                .insert(self.stations[station].station.clone());
        }
        let now = self.position();
        self.progress.stations_visited = now.stations_visited;
        self.progress.completed_runs = now.runs_completed;
        AdvanceOutcome::Advanced {
            seconds: dt,
            distance_units: distance,
        }
    }

    /// Validates the v5 per-line record and generic services before publishing either.
    /// The application checkpoint deliberately still rejects non-Aurora profiles.
    pub fn restore(
        &mut self,
        progress: LineProgress,
        service: ServiceState<CatalogStationId>,
    ) -> Result<(), String> {
        let invalid = || "Progresso da linha inválido; estado preservado.".to_owned();
        if progress.seed != self.progress.seed
            || !progress.distance.is_finite()
            || !(0.0..=1e12).contains(&progress.distance)
            || !progress.active_seconds.is_finite()
            || !(0.0..=1e12).contains(&progress.active_seconds)
            || !progress.station_wait.is_finite()
            || !(0.0..=8.0).contains(&progress.station_wait)
        {
            return Err(invalid());
        }
        let position = self.itinerary.locate(progress.distance);
        let discovered: BTreeSet<_> = self
            .stations
            .iter()
            .enumerate()
            .filter(|(i, _)| *i as u64 <= position.stations_visited)
            .map(|(_, s)| s.station.clone())
            .collect();
        if progress.stations_visited != position.stations_visited
            || progress.completed_runs != position.runs_completed
            || progress.discovered_stations != discovered
            || (progress.station_wait > 0.0 && position.at_station.is_none())
            || position
                .at_station
                .is_some_and(|s| progress.station_wait > self.dwell[s])
            || service.branch != BranchState::default()
        {
            return Err(invalid());
        }
        service.validate_route(
            progress.stations_visited,
            |visit| self.stations[self.itinerary.station_at_visit(visit)].clone(),
            |station| self.stations.contains(station),
            72,
        )?;
        self.progress = progress;
        self.service = service;
        Ok(())
    }
}
