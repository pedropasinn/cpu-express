//! Claims and recurring orders use durable counters, never UI notifications.
use super::{
    CargoKind, JourneySnapshot, ServiceState, ServiceWagon,
    catalog::Catalog,
    economy::{DailyOrder, EconomyState, ItemId},
    objectives::{Livery, MISSIONS},
};

pub const ORDER_INTERVAL_SECONDS: f64 = 600.0;
pub const MILESTONE_REWARDS: [u64; 11] = [15, 25, 40, 15, 30, 25, 20, 20, 30, 20, 25];
pub const CARGO_MISSIONS: [(&str, CargoKind); 7] = [
    ("ce.parcels-24", CargoKind::Parcels),
    ("ce.supplies-24", CargoKind::Supplies),
    ("ce.liquids-24", CargoKind::Liquid),
    ("ce.ore-24", CargoKind::Ore),
    ("ce.grain-24", CargoKind::Grain),
    ("ce.timber-24", CargoKind::Timber),
    ("ce.chilled-24", CargoKind::Chilled),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionStatus {
    pub id: &'static str,
    pub title: &'static str,
    pub current: u64,
    pub goal: u64,
    pub reward: u64,
    pub paid: bool,
    pub visible: bool,
}

#[cfg(test)]
mod tests {
    use super::super::{BranchStop, ServiceReward, World};
    use super::*;

    #[test]
    fn all_eighteen_missions_pay_once_and_total_405() {
        let mut snapshot = World::default().journey_snapshot();
        snapshot.discovered_stations = [true; 4];
        let mut service = ServiceState::migrated(snapshot);
        service.passengers_delivered = 300;
        service.cargo_delivered = 168;
        service.contracts_completed = 10;
        service.collection = vec![
            ServiceReward::PassengerPostcard,
            ServiceReward::CargoPostcard,
        ];
        service.branch.discoveries = vec![BranchStop::MiranteDaSerra, BranchStop::CaisDasConchas];
        let mut economy = EconomyState::new(42);
        economy
            .inventory
            .extend(Livery::ORIGINAL.map(ItemId::livery));
        for (_, kind) in CARGO_MISSIONS {
            economy.cargo_by_kind.insert(kind.id().into(), 24);
        }
        let missions = economy.missions(snapshot, &service);
        assert_eq!(missions.len(), 18);
        assert_eq!(missions.iter().map(|m| m.reward).sum::<u64>(), 405);
        for m in missions {
            assert!(m.claimable());
            economy.claim_mission(m.id, snapshot, &service).unwrap();
            assert!(economy.claim_mission(m.id, snapshot, &service).is_err());
        }
        assert_eq!(economy.balance, 435);
    }

    #[test]
    fn cargo_thresholds_do_not_use_aggregate_or_other_species() {
        let snapshot = World::default().journey_snapshot();
        let mut service = ServiceState::migrated(snapshot);
        service.cargo_delivered = 9999;
        for (id, kind) in CARGO_MISSIONS {
            let mut e = EconomyState::new(5);
            assert!(e.claim_mission(id, snapshot, &service).is_err());
            e.cargo_by_kind.insert(kind.id().into(), 23);
            assert!(e.claim_mission(id, snapshot, &service).is_err());
            e.cargo_by_kind.insert(kind.id().into(), 24);
            e.claim_mission(id, snapshot, &service).unwrap();
            assert_eq!(e.paid_missions.len(), 1);
        }
    }

    #[test]
    fn order_baselines_swap_cooldowns_and_invalid_definitions() {
        let mut snapshot = World::default().journey_snapshot();
        let service = ServiceState::migrated(snapshot);
        let mut e = EconomyState::new(12);
        snapshot.active_seconds = 600.0;
        snapshot.stations_visited = 100;
        e.offer_order_on_arrival(snapshot, &service, false);
        let o = e.daily_orders[0].clone();
        assert_eq!(o.progress(&e, snapshot, &service), 0);
        assert!(!o.claimable(&e, snapshot, &service));
        assert!(e.swap_order(o.id, snapshot, &service).is_err());
        snapshot.active_seconds = 1200.0;
        snapshot.at_station = None;
        assert!(e.swap_order(o.id, snapshot, &service).is_err());
        snapshot.at_station = Some(super::super::StationId::Aurora);
        e.swap_order(o.id, snapshot, &service).unwrap();
        let replacement = &e.daily_orders[0];
        assert_ne!(replacement.template_id, o.template_id);
        assert_eq!(e.balance, 30);
        assert!(e.swap_order(replacement.id, snapshot, &service).is_err());
        let count = e.daily_orders.len();
        e.offer_order_on_arrival(snapshot, &service, true);
        assert_eq!(e.daily_orders.len(), count);
        e.validate().unwrap();
        e.daily_orders[0].reward += 1;
        assert!(e.validate().is_err());
    }
}
impl MissionStatus {
    pub fn claimable(&self) -> bool {
        self.current >= self.goal && !self.paid
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OrderCounter {
    Arrivals,
    Passengers,
    Contracts,
    Cargo(CargoKind),
    ActiveSeconds,
}
impl OrderCounter {
    pub fn key(self) -> &'static str {
        match self {
            Self::Arrivals => "arrivals",
            Self::Passengers => "passengers",
            Self::Contracts => "contracts",
            Self::Cargo(kind) => kind.id(),
            Self::ActiveSeconds => "active_seconds",
        }
    }
    fn value(
        self,
        economy: &EconomyState,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> u64 {
        match self {
            Self::Arrivals => snapshot.stations_visited,
            Self::Passengers => service.passengers_delivered,
            Self::Contracts => service.contracts_completed,
            Self::Cargo(kind) => economy.cargo_by_kind.get(kind.id()).copied().unwrap_or(0),
            Self::ActiveSeconds => snapshot.active_seconds as u64,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct OrderTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub counter: OrderCounter,
    pub goal: u64,
    pub reward: u64,
}
pub const ORDER_TEMPLATES: [OrderTemplate; 11] = [
    OrderTemplate {
        id: "arrivals",
        title: "Visite 24 estações",
        counter: OrderCounter::Arrivals,
        goal: 24,
        reward: 10,
    },
    OrderTemplate {
        id: "passengers",
        title: "Entregue 120 passageiros",
        counter: OrderCounter::Passengers,
        goal: 120,
        reward: 10,
    },
    OrderTemplate {
        id: "contracts",
        title: "Conclua 8 encomendas",
        counter: OrderCounter::Contracts,
        goal: 8,
        reward: 15,
    },
    OrderTemplate {
        id: "active",
        title: "Viaje uma hora ativa",
        counter: OrderCounter::ActiveSeconds,
        goal: 3600,
        reward: 10,
    },
    OrderTemplate {
        id: "parcels",
        title: "Entregue 48 encomendas",
        counter: OrderCounter::Cargo(CargoKind::Parcels),
        goal: 48,
        reward: 15,
    },
    OrderTemplate {
        id: "supplies",
        title: "Entregue 48 suprimentos",
        counter: OrderCounter::Cargo(CargoKind::Supplies),
        goal: 48,
        reward: 15,
    },
    OrderTemplate {
        id: "liquid",
        title: "Entregue 48 líquidos",
        counter: OrderCounter::Cargo(CargoKind::Liquid),
        goal: 48,
        reward: 15,
    },
    OrderTemplate {
        id: "ore",
        title: "Entregue 48 de minério",
        counter: OrderCounter::Cargo(CargoKind::Ore),
        goal: 48,
        reward: 15,
    },
    OrderTemplate {
        id: "grain",
        title: "Entregue 48 de grãos",
        counter: OrderCounter::Cargo(CargoKind::Grain),
        goal: 48,
        reward: 15,
    },
    OrderTemplate {
        id: "timber",
        title: "Entregue 48 de madeira",
        counter: OrderCounter::Cargo(CargoKind::Timber),
        goal: 48,
        reward: 15,
    },
    OrderTemplate {
        id: "chilled",
        title: "Entregue 48 refrigerados",
        counter: OrderCounter::Cargo(CargoKind::Chilled),
        goal: 48,
        reward: 15,
    },
];
impl DailyOrder {
    pub fn template(&self) -> Option<&'static OrderTemplate> {
        ORDER_TEMPLATES.iter().find(|t| t.id == self.template_id)
    }
    pub fn progress(
        &self,
        economy: &EconomyState,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> u64 {
        self.template().map_or(0, |t| {
            t.counter
                .value(economy, snapshot, service)
                .saturating_sub(*self.baselines.get(t.counter.key()).unwrap_or(&u64::MAX))
                .min(self.goal)
        })
    }
    pub fn claimable(
        &self,
        economy: &EconomyState,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> bool {
        self.progress(economy, snapshot, service) >= self.goal
    }
}

impl EconomyState {
    pub fn collection_items() -> Vec<ItemId> {
        let Ok(catalog) = Catalog::builtin() else {
            return Vec::new();
        };
        catalog
            .wagons
            .iter()
            .filter(|v| !v.retired)
            .map(|v| ItemId::Wagon(v.id.clone()))
            .chain(
                catalog
                    .locomotives
                    .iter()
                    .filter(|v| !v.retired)
                    .map(|v| ItemId::Locomotive(v.id.clone())),
            )
            .chain(
                catalog
                    .liveries
                    .iter()
                    .filter(|v| !v.retired)
                    .map(|v| ItemId::Livery(v.id.clone())),
            )
            .collect()
    }
    pub fn collection_complete(&self) -> bool {
        let items = Self::collection_items();
        !items.is_empty() && items.iter().all(|i| self.owns(i))
    }
    pub fn missions(
        &self,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> Vec<MissionStatus> {
        let mut missions: Vec<_> = MISSIONS
            .iter()
            .zip(MILESTONE_REWARDS)
            .map(|(m, reward)| {
                // This milestone names the four original paints, not any four paints.
                let current = if m.id == "ce.liveries-4" {
                    Livery::ORIGINAL
                        .iter()
                        .filter(|l| self.owns(&ItemId::livery(**l)))
                        .count() as u64
                } else {
                    m.progress(snapshot, service).current
                };
                MissionStatus {
                    id: m.id,
                    title: m.title,
                    current,
                    goal: m.goal,
                    reward,
                    paid: self.paid_missions.contains(m.id),
                    visible: true,
                }
            })
            .collect();
        missions.extend(CARGO_MISSIONS.iter().map(|&(id, kind)| {
            let current = self
                .cargo_by_kind
                .get(kind.id())
                .copied()
                .unwrap_or(0)
                .min(24);
            MissionStatus {
                id,
                title: kind.name(),
                current,
                goal: 24,
                reward: 20,
                paid: self.paid_missions.contains(id),
                visible: current > 0
                    || ServiceWagon::ALL
                        .iter()
                        .any(|w| w.cargo_capacity(kind) > 0 && self.owns(&ItemId::wagon(*w))),
            }
        }));
        missions
    }
    pub(super) fn claim_mission(
        &mut self,
        id: &str,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> Result<(), String> {
        let mission = self
            .missions(snapshot, service)
            .into_iter()
            .find(|m| m.id == id)
            .ok_or("Missão desconhecida.")?;
        if !mission.claimable() {
            return Err("Missão incompleta ou já recolhida.".into());
        }
        self.record(mission.title, mission.reward, 0)?;
        self.paid_missions.insert(id.into());
        Ok(())
    }
    pub(super) fn settle_legacy_missions(
        &mut self,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> Result<(), String> {
        if !self.legacy_mission_claim_pending {
            return Err("Marcos antigos já conferidos.".into());
        }
        for m in self
            .missions(snapshot, service)
            .into_iter()
            .filter(|m| m.claimable())
        {
            self.claim_mission(m.id, snapshot, service)?;
        }
        self.legacy_mission_claim_pending = false;
        Ok(())
    }
    /// Conservative eligibility: existing stock, or one owned replacement that fits all aboard.
    pub fn order_eligible(
        &self,
        template: &OrderTemplate,
        service: &ServiceState<impl Clone + Eq>,
    ) -> bool {
        let supports = |composition: &[ServiceWagon; 3]| match template.counter {
            OrderCounter::Passengers => composition.iter().any(|w| w.capacity().0 > 0),
            OrderCounter::Cargo(kind) => {
                composition
                    .iter()
                    .map(|w| w.cargo_capacity(kind))
                    .sum::<u32>()
                    >= 12
            }
            OrderCounter::Contracts => CargoKind::ALL.iter().any(|k| {
                composition
                    .iter()
                    .map(|w| w.cargo_capacity(*k))
                    .sum::<u32>()
                    >= 12
            }),
            _ => true,
        };
        if supports(&service.composition) {
            return true;
        }
        for wagon in ServiceWagon::ALL
            .into_iter()
            .filter(|w| self.owns(&ItemId::wagon(*w)))
        {
            for slot in 0..3 {
                let mut composition = service.composition;
                composition[slot] = wagon;
                if supports(&composition) && service.can_carry_with(composition) {
                    return true;
                }
            }
        }
        false
    }
    fn make_order(
        &mut self,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
        telegram: bool,
        excluded: Option<&str>,
    ) -> Option<DailyOrder> {
        let templates: Vec<_> = ORDER_TEMPLATES
            .iter()
            .filter(|t| {
                Some(t.id) != excluded
                    && !self.daily_orders.iter().any(|o| o.template_id == t.id)
                    && self.order_eligible(t, service)
            })
            .collect();
        if templates.is_empty() {
            return None;
        }
        let next = self.next_order_id.checked_add(1)?;
        let t = templates[self.random_below(templates.len() as u64) as usize];
        let order = DailyOrder {
            id: self.next_order_id,
            template_id: t.id.into(),
            baselines: [(
                t.counter.key().into(),
                t.counter.value(self, snapshot, service),
            )]
            .into_iter()
            .collect(),
            goal: t.goal,
            reward: t.reward,
            entered_active_seconds: snapshot.active_seconds,
            telegram,
        };
        self.next_order_id = next;
        Some(order)
    }
    /// Called only by the arrival transaction. A telegram competes for the same interval.
    pub(crate) fn offer_order_on_arrival(
        &mut self,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
        telegram_requested: bool,
    ) -> bool {
        if snapshot.active_seconds - self.last_order_active_seconds < ORDER_INTERVAL_SECONDS {
            return false;
        }
        let telegram = self.daily_orders.iter().filter(|o| !o.telegram).count() >= 3;
        if telegram && (!telegram_requested || self.daily_orders.iter().any(|o| o.telegram)) {
            return false;
        }
        if let Some(order) = self.make_order(snapshot, service, telegram, None) {
            self.daily_orders.push(order);
            self.last_order_active_seconds = snapshot.active_seconds;
            return telegram;
        }
        false
    }
    pub(super) fn claim_order(
        &mut self,
        id: u64,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> Result<(), String> {
        let index = self
            .daily_orders
            .iter()
            .position(|o| o.id == id)
            .ok_or("Ordem já recolhida ou desconhecida.")?;
        let order = &self.daily_orders[index];
        if !order.claimable(self, snapshot, service) {
            return Err("Ordem ainda incompleta.".into());
        }
        self.record("Ordem do dia", order.reward, 0)?;
        self.daily_orders.remove(index);
        Ok(())
    }
    pub(super) fn swap_order(
        &mut self,
        id: u64,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> Result<(), String> {
        if snapshot.at_station.is_none()
            || service.branch.current.is_some()
            || snapshot.active_seconds - self.last_order_active_seconds < ORDER_INTERVAL_SECONDS
            || snapshot.active_seconds - self.last_order_swap_active_seconds
                < ORDER_INTERVAL_SECONDS
        {
            return Err(
                "Troque na próxima parada, após dez minutos ativos da última ordem ou troca."
                    .into(),
            );
        }
        let index = self
            .daily_orders
            .iter()
            .position(|o| o.id == id)
            .ok_or("Ordem desconhecida.")?;
        let old = self.daily_orders[index].clone();
        let order = self
            .make_order(snapshot, service, old.telegram, Some(&old.template_id))
            .ok_or("Nenhuma outra ordem compatível agora.")?;
        self.daily_orders[index] = order;
        self.last_order_active_seconds = snapshot.active_seconds;
        self.last_order_swap_active_seconds = snapshot.active_seconds;
        Ok(())
    }
}
