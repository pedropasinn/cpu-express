use super::*;
use crate::simulation::{
    Biome, ContractStatus, ServiceReward, ServiceState,
    catalog::{Catalog, LineDef},
    line_session::{CatalogStationId, LineSession},
    route::CIRCUIT_LINE_ID,
};

/// Allocation-free route view shared by the strip, preview and scenery adapter.
#[derive(Clone, Copy, Debug)]
pub struct TravelView {
    pub seed: u64,
    pub line_name: &'static str,
    pub segment: usize,
    pub from_index: usize,
    pub to_index: usize,
    pub from_name: &'static str,
    pub next_name: &'static str,
    pub forward: bool,
    pub biome: Biome,
    pub next_biome: Biome,
    pub segment_length: f64,
    pub segment_progress: f64,
    pub distance_to_next: f64,
    pub stations_visited: u64,
}

impl World {
    pub fn missions(&self) -> Vec<crate::simulation::economy_goals::MissionStatus> {
        let mut missions = self
            .economy
            .missions(self.economic_snapshot(), &self.service_view);
        let discovered = self
            .sessions
            .get("curitiba-paranagua")
            .map(|s| &s.progress().discovered_stations);
        for (id, title, current, goal, reward) in [
            (
                "line.curitiba-paranagua.stations",
                "Conheça as 14 paradas",
                discovered.map_or(0, |d| d.len() as u64),
                14,
                30,
            ),
            (
                "line.curitiba-paranagua.ascent",
                "Suba a serra",
                u64::from(discovered.is_some_and(|d| d.contains("roca-nova"))),
                1,
                20,
            ),
        ] {
            missions.push(crate::simulation::economy_goals::MissionStatus {
                id,
                title,
                current,
                goal,
                reward,
                paid: self.economy.paid_missions.contains(id),
                visible: true,
            });
        }
        missions
    }
    pub fn active_line_id(&self) -> &str {
        &self.active_line
    }
    pub fn active_session(&self) -> Option<&LineSession> {
        self.sessions.get(&self.active_line)
    }
    pub fn boarding_closed(&self) -> bool {
        self.boarding_closed
    }
    /// Global counters with the active line's occupancy and qualified endpoints.
    /// This cache is rebuilt only on domain changes, never by the renderer.
    pub fn service_view(&self) -> &ServiceState<CatalogStationId> {
        &self.service_view
    }
    pub fn pop_catalog_arrival(
        &mut self,
    ) -> Option<crate::simulation::ArrivalSummary<CatalogStationId>> {
        self.catalog_arrival.take()
    }

    pub fn economic_snapshot(&self) -> JourneySnapshot {
        let mut snapshot = self.journey.snapshot();
        for session in self.sessions.values() {
            snapshot.stations_visited += session.progress().stations_visited;
            snapshot.active_seconds += session.progress().active_seconds;
            snapshot.total_distance_units += session.progress().distance;
        }
        // Legacy economic consumers only use presence of a platform, never this ID.
        if let Some(session) = self.active_session() {
            snapshot.at_station = session
                .position()
                .at_station
                .map(|_| super::super::StationId::Aurora);
        }
        snapshot
    }

    pub(super) fn refresh_service_view(&mut self) {
        let mut view = if let Some(session) = self.active_session() {
            session.service().clone()
        } else {
            self.service
                .map_stations(|id| CatalogStationId::aurora(*id))
        };
        view.passengers_boarded = self.service.passengers_boarded;
        view.passengers_delivered = self.service.passengers_delivered;
        view.cargo_delivered = self.service.cargo_delivered;
        view.contracts_completed = self.service.contracts_completed;
        view.credits = self.service.credits;
        for session in self.sessions.values() {
            let s = session.service();
            view.passengers_boarded += s.passengers_boarded;
            view.passengers_delivered += s.passengers_delivered;
            view.cargo_delivered += s.cargo_delivered;
            view.contracts_completed += s.contracts_completed;
            view.credits += s.credits;
        }
        view.last_processed_visit = self.economic_snapshot().stations_visited;
        view.branch = self.service.branch.clone();
        view.owned_rewards = self.service.owned_rewards.clone();
        view.collection.clear();
        if view.passengers_delivered >= 20 {
            view.collection.push(ServiceReward::PassengerPostcard);
        }
        if view.cargo_delivered >= 20 {
            view.collection.push(ServiceReward::CargoPostcard);
        }
        self.service_view = view;
    }

    pub(super) fn pay_arrival(&mut self) {
        self.refresh_service_view();
        self.economy.arrive(
            self.economic_snapshot(),
            &self.service_view,
            self.machine_bonus_millis,
        );
        self.sync_owned_liveries();
        if self.economy.offer_order_on_arrival(
            self.economic_snapshot(),
            &self.service_view,
            self.telegram_requested,
        ) {
            self.telegram_requested = false;
        }
    }

    pub fn can_leave_line(&self) -> bool {
        if self.service.branch.current.is_some() || self.service.branch.pending.is_some() {
            return false;
        }
        if let Some(session) = self.active_session() {
            return session.can_leave_line();
        }
        self.journey.snapshot().at_station.is_some()
            && self.service.passenger_count() == 0
            && self.service.cargo_count() == 0
            && self.service.contract.as_ref().is_none_or(|c| {
                matches!(
                    c.status,
                    ContractStatus::Offered | ContractStatus::Delivered | ContractStatus::Cancelled
                )
            })
    }

    pub fn set_boarding_closed(
        &mut self,
        closed: bool,
        persist: impl FnOnce(&World) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut candidate = self.clone();
        candidate.boarding_closed = closed;
        persist(&candidate)?;
        *self = candidate;
        Ok(())
    }

    pub fn switch_line(
        &mut self,
        id: &str,
        persist: impl FnOnce(&World) -> Result<(), String>,
    ) -> Result<(), String> {
        if id == self.active_line {
            return Err("Você já está nesta linha.".into());
        }
        if !self.can_leave_line() {
            return Err("Encerre embarques e conclua os serviços. Troque numa parada (terminal na linha real), fora de excursão.".into());
        }
        let catalog = Catalog::builtin()
            .as_ref()
            .map_err(|_| "Catálogo indisponível.")?;
        let line = catalog.line(id).ok_or("Linha desconhecida.")?;
        let mut candidate = self.clone();
        if id != CIRCUIT_LINE_ID && !candidate.sessions.contains_key(id) {
            let mut session = LineSession::playable(line, self.journey.snapshot().seed)?;
            session.service_mut().composition = self.service.composition;
            session.set_automatic_contracts(self.service.automatic_contracts);
            candidate.sessions.insert(id.into(), session);
        } else {
            crate::simulation::itinerary::playable(line, 480.0)?;
        }
        candidate.active_line = id.into();
        candidate.journey.drain_events().for_each(drop);
        candidate.catalog_arrival = None;
        // Keep embarkation closed until explicitly resumed; switching never drops an offer.
        candidate.sync_owned_liveries();
        persist(&candidate)?;
        *self = candidate;
        Ok(())
    }

    pub(crate) fn restore_lines(
        &mut self,
        active: String,
        lines: std::collections::BTreeMap<String, LineProgress>,
        services: std::collections::BTreeMap<String, ServiceState<CatalogStationId>>,
        boarding_closed: bool,
    ) -> Result<(), String> {
        let catalog = Catalog::builtin()
            .as_ref()
            .map_err(|_| "Catálogo indisponível.")?;
        if lines.get(CIRCUIT_LINE_ID) != self.line_progress().get(CIRCUIT_LINE_ID)
            || services.len() + 1 != lines.len()
        {
            return Err("Progresso por linha divergente.".into());
        }
        let mut sessions = std::collections::BTreeMap::new();
        for (id, service) in services {
            if id == CIRCUIT_LINE_ID {
                return Err("Serviços Aurora duplicados.".into());
            }
            let progress = lines.get(&id).ok_or("Linha sem progresso.")?;
            let line = catalog
                .line(&id)
                .ok_or("Linha desconhecida; arquivo preservado.")?;
            let mut session = LineSession::playable(line, progress.seed)?;
            if service.composition != self.service.composition
                || service.automatic_contracts != self.service.automatic_contracts
                || service.owned_rewards != self.service.owned_rewards
            {
                return Err("Preferências de serviço divergentes.".into());
            }
            session.restore(progress.clone(), service)?;
            if id != active && !session.can_leave_line() {
                return Err("Linha inativa com serviço pendente.".into());
            }
            sessions.insert(id, session);
        }
        if active != CIRCUIT_LINE_ID && (!sessions.contains_key(&active) || !self.can_leave_line())
        {
            return Err("Linha ativa inválida ou Aurora com serviço pendente.".into());
        }
        self.sessions = sessions;
        self.active_line = active;
        self.boarding_closed = boarding_closed;
        self.elapsed_seconds = self.economic_snapshot().active_seconds;
        self.refresh_service_view();
        Ok(())
    }

    pub(super) fn advance_catalog_line(
        &mut self,
        train: &mut Train,
        cpu: f32,
        dt: f64,
        config: &Config,
    ) -> AdvanceOutcome {
        let before = self.active_session().unwrap().progress().stations_visited;
        let waiting = self.station_wait();
        let outcome = self
            .sessions
            .get_mut(&self.active_line)
            .unwrap()
            .advance_with_boarding(dt, !self.boarding_closed);
        let AdvanceOutcome::Advanced { distance_units, .. } = outcome else {
            return outcome;
        };
        let moving = distance_units / LineSession::SPEED;
        let distance = if moving > 0.0 {
            let braking = (self.travel_view().distance_to_next / 120.0).clamp(0.05, 1.0);
            train.update_with_braking(cpu, moving, config, braking)
        } else {
            0.0
        };
        if waiting >= dt || self.station_wait() > 0.0 {
            train.stop_at_station();
        }
        self.advance_visual(dt, distance, train.speed_kmh());
        if self.active_session().unwrap().progress().stations_visited > before {
            self.pay_arrival();
            self.catalog_arrival = self.service_view.arrivals.last().cloned();
        }
        outcome
    }

    pub fn station_dwell(&self) -> f64 {
        self.active_session()
            .map_or(8.0, LineSession::dwell_seconds)
    }

    pub fn travel_view(&self) -> TravelView {
        if let Some(session) = self.active_session() {
            let line: &'static LineDef = Catalog::builtin()
                .as_ref()
                .unwrap()
                .line(&self.active_line)
                .unwrap();
            let p = session.position();
            let next = if p.forward {
                (p.segment + 1).min(line.segments.len() - 1)
            } else {
                p.segment.saturating_sub(1)
            };
            let biome = |s: &str| match s {
                "forest" => Biome::Forest,
                "coast" => Biome::Coast,
                "highland" => Biome::Highland,
                _ => Biome::Meadow,
            };
            let length = session.itinerary().segment_length(p.segment).unwrap();
            TravelView {
                seed: session.progress().seed,
                line_name: &line.name,
                segment: p.segment,
                from_index: p.from,
                to_index: p.to,
                from_name: &line
                    .stations
                    .iter()
                    .filter(|s| !s.retired)
                    .nth(p.from)
                    .unwrap()
                    .name,
                next_name: &line
                    .stations
                    .iter()
                    .filter(|s| !s.retired)
                    .nth(p.to)
                    .unwrap()
                    .name,
                forward: p.forward,
                biome: biome(&line.segments[p.segment].biome),
                next_biome: biome(&line.segments[next].biome),
                segment_length: length,
                segment_progress: p.distance_into_segment / length,
                distance_to_next: p.distance_to_next,
                stations_visited: session.progress().stations_visited,
            }
        } else {
            let s = self.journey.snapshot();
            TravelView {
                seed: s.seed,
                line_name: s.route_name,
                segment: s.route_segment.index,
                from_index: s.route_segment.from.index(),
                to_index: s.route_segment.to.index(),
                from_name: s.route_segment.from.name(),
                next_name: s.next_station.name,
                forward: true,
                biome: s.route_segment.biome_hint,
                next_biome: s.next_biome_hint,
                segment_length: s.route_segment.length_units,
                segment_progress: s.segment_progress,
                distance_to_next: s.distance_to_next_station_units,
                stations_visited: s.stations_visited,
            }
        }
    }
}
