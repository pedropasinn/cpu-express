//! Deterministic economy. Mutations are staged by World and committed by the caller's writer.
use super::{JourneySnapshot, ServiceState, ServiceWagon, catalog::Catalog, objectives::Livery};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const TRANSACTION_LIMIT: usize = 128;
pub const ECONOMY_RULE_VERSION: u32 = 1;
/// Balance calibration, applied before the machine bonus; fractions remain durable.
pub const ARRIVAL_INCOME_MILLIS: u64 = 350;

pub use super::catalog::Rarity;

/// Categories have separate namespaces in the catalogue.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ItemId {
    Wagon(String),
    Locomotive(String),
    Livery(String),
}

impl ItemId {
    pub fn wagon(wagon: ServiceWagon) -> Self {
        Self::Wagon(wagon.id().into())
    }
    pub fn livery(livery: Livery) -> Self {
        Self::Livery(livery.id().into())
    }
    pub fn rarity(&self) -> Option<Rarity> {
        let catalog = Catalog::builtin().as_ref().ok()?;
        match self {
            Self::Wagon(id) => catalog
                .wagons
                .iter()
                .find(|v| &v.id == id && !v.retired)
                .map(|v| v.rarity),
            Self::Locomotive(id) => catalog
                .locomotives
                .iter()
                .find(|v| &v.id == id && !v.retired)
                .map(|v| v.rarity),
            Self::Livery(id) => catalog
                .liveries
                .iter()
                .find(|v| &v.id == id && !v.retired)
                .map(|v| v.rarity),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrateKind {
    GeneralYard,
    CargoWorkshop,
    PlatformComfort,
    EngineGarage,
    PaintAtelier,
}
impl CrateKind {
    pub const ALL: [Self; 5] = [
        Self::GeneralYard,
        Self::CargoWorkshop,
        Self::PlatformComfort,
        Self::EngineGarage,
        Self::PaintAtelier,
    ];
    pub fn definition(self) -> Result<CrateDef, String> {
        let (price, weights, guarantee, wagons, engines, paints) = match self {
            Self::GeneralYard => (
                150,
                [70, 22, 7, 1],
                10,
                GENERAL_WAGONS,
                GENERAL_ENGINES,
                &[][..],
            ),
            Self::CargoWorkshop => (
                225,
                [60, 29, 10, 1],
                8,
                CARGO_WAGONS,
                CARGO_ENGINES,
                &[][..],
            ),
            Self::PlatformComfort => (225, [60, 29, 11, 0], 8, PLATFORM_WAGONS, &[][..], &[][..]),
            Self::EngineGarage => (450, [55, 30, 13, 2], 6, &[][..], GENERAL_ENGINES, &[][..]),
            Self::PaintAtelier => (188, [65, 25, 10, 0], 8, &[][..], &[][..], PAINTS),
        };
        Ok(CrateDef {
            kind: self,
            version: if self == Self::GeneralYard { 2 } else { 1 },
            price,
            weights,
            guarantee,
            items: wagons
                .iter()
                .map(|id| ItemId::Wagon((*id).into()))
                .chain(engines.iter().map(|id| ItemId::Locomotive((*id).into())))
                .chain(paints.iter().map(|id| ItemId::Livery((*id).into())))
                .collect(),
        })
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::GeneralYard => "Pátio geral",
            Self::CargoWorkshop => "Cargas e oficina",
            Self::PlatformComfort => "Plataforma e conforto",
            Self::EngineGarage => "Garagem de locomotivas",
            Self::PaintAtelier => "Ateliê de pinturas",
        }
    }
}
pub const CARGO_WAGONS: &[&str] = &[
    "freight",
    "tank",
    "mail",
    "gondola",
    "hopper",
    "timber",
    "refrigerator",
    "livestock",
    "container",
    "autorack",
    "pressure_tank",
    "crane",
    "bicycle",
    "coil",
    "fair",
];
pub const CARGO_ENGINES: &[&str] = &[
    "diesel",
    "freight_steam",
    "shunter",
    "articulated_steam",
    "twin_diesel",
];
pub const PLATFORM_WAGONS: &[&str] = &[
    "passenger",
    "sleeper",
    "observation",
    "dining",
    "mail",
    "bicycle",
    "fair",
];
pub const PAINTS: &[&str] = &[
    "aurora",
    "ocean",
    "sunset",
    "midnight",
    "cherry",
    "ivory",
    "lagoon",
    "heather",
    "sunflower",
    "slate",
];
pub const GENERAL_WAGONS: &[&str] = &[
    "passenger",
    "sleeper",
    "observation",
    "freight",
    "tank",
    "dining",
    "mail",
    "gondola",
    "hopper",
    "timber",
    "refrigerator",
    "livestock",
    "container",
    "autorack",
    "pressure_tank",
    "crane",
    "bicycle",
    "coil",
    "fair",
];
pub const GENERAL_ENGINES: &[&str] = &[
    "steam",
    "diesel",
    "electric",
    "freight_steam",
    "shunter",
    "railcar",
    "express_steam",
    "articulated_steam",
    "twin_diesel",
    "rack_electric",
];

/// Explicit, versioned pools, validated before sale.
#[derive(Clone, Debug)]
pub struct CrateDef {
    pub kind: CrateKind,
    pub version: u32,
    pub price: u64,
    pub weights: [u32; 4],
    pub guarantee: u32,
    pub items: Vec<ItemId>,
}
impl CrateDef {
    pub fn validate(&self) -> Result<(), String> {
        if self.version == 0
            || self.price == 0
            || self.guarantee == 0
            || self.weights.iter().map(|&v| u64::from(v)).sum::<u64>() != 100
            || self.weights[2] + self.weights[3] == 0
            || self.items.is_empty()
            || self.items.iter().collect::<BTreeSet<_>>().len() != self.items.len()
            || self.items.iter().any(|item| item.rarity().is_none())
            || Rarity::ALL.iter().enumerate().any(|(i, rarity)| {
                self.weights[i] > 0 && !self.items.iter().any(|item| item.rarity() == Some(*rarity))
            })
        {
            return Err("Tabela de remessa inválida.".into());
        }
        Ok(())
    }
    pub fn effective_weights(&self, misses: u32) -> [u32; 4] {
        if misses >= self.guarantee.saturating_sub(1) {
            [0, 0, self.weights[2], self.weights[3]]
        } else {
            self.weights
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transaction {
    pub id: u64,
    pub reason: String,
    pub credited: u64,
    pub debited: u64,
    pub balance: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingPurchase {
    pub transaction_id: u64,
    pub crate_kind: CrateKind,
    pub pool_version: u32,
    pub item: ItemId,
    pub rarity: Rarity,
    pub paid: u64,
    pub duplicate: bool,
    pub refund: u64,
}

/// Reserved v5 contract for E1b: baselines are captured when an order enters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DailyOrder {
    pub id: u64,
    pub template_id: String,
    pub baselines: BTreeMap<String, u64>,
    pub goal: u64,
    pub reward: u64,
    pub entered_active_seconds: f64,
    pub telegram: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EconomyState {
    pub rule_version: u32,
    pub balance: u64,
    pub total_earned: u64,
    pub total_spent: u64,
    pub next_transaction_id: u64,
    pub transactions: VecDeque<Transaction>,
    pub inventory: BTreeSet<ItemId>,
    pub pending_purchase: Option<PendingPurchase>,
    pub crate_misses: BTreeMap<CrateKind, u32>,
    pub random_state: u64,
    pub welcome_crate_available: bool,
    pub passenger_remainder: u64,
    pub contract_remainder: u64,
    /// Thousandths of a coin, never rounded away between arrivals.
    pub bonus_remainder: u64,
    pub cargo_by_kind: BTreeMap<String, u64>,
    pub paid_missions: BTreeSet<String>,
    pub legacy_mission_claim_pending: bool,
    pub daily_orders: Vec<DailyOrder>,
    pub next_order_id: u64,
    pub last_order_active_seconds: f64,
    pub last_order_swap_active_seconds: f64,
    pub museum_stamps: u64,
}

#[derive(Clone, Debug)]
pub enum EconomyAction {
    ClaimMission(String),
    ClaimOrder {
        order_id: u64,
    },
    SwapOrder {
        order_id: u64,
    },
    SettleLegacyMissions,
    BuyMuseumStamp,
    BuyCrate {
        kind: CrateKind,
        free: bool,
    },
    BuyDirect(ItemId),
    /// Acknowledge the already committed result; never draws or pays again.
    Reveal {
        transaction_id: u64,
    },
}

impl EconomyState {
    pub fn new(seed: u64) -> Self {
        let inventory = [
            ItemId::Locomotive("steam".into()),
            ItemId::wagon(ServiceWagon::Passenger),
            ItemId::wagon(ServiceWagon::Freight),
            ItemId::wagon(ServiceWagon::Tank),
            ItemId::livery(Livery::Aurora),
        ]
        .into_iter()
        .collect();
        let mut state = Self {
            rule_version: ECONOMY_RULE_VERSION,
            balance: 0,
            total_earned: 0,
            total_spent: 0,
            next_transaction_id: 1,
            transactions: VecDeque::new(),
            inventory,
            pending_purchase: None,
            crate_misses: CrateKind::ALL.into_iter().map(|kind| (kind, 0)).collect(),
            random_state: seed,
            welcome_crate_available: true,
            passenger_remainder: 0,
            contract_remainder: 0,
            bonus_remainder: 0,
            cargo_by_kind: super::CargoKind::ALL
                .into_iter()
                .map(|kind| (kind.id().into(), 0))
                .collect(),
            paid_missions: BTreeSet::new(),
            legacy_mission_claim_pending: false,
            daily_orders: Vec::new(),
            next_order_id: 1,
            last_order_active_seconds: 0.0,
            last_order_swap_active_seconds: 0.0,
            museum_stamps: 0,
        };
        state
            .record("Boas-vindas", 30, 0)
            .expect("initial balance fits");
        state
    }
    pub fn migrated(snapshot: JourneySnapshot, service: &ServiceState<impl Clone + Eq>) -> Self {
        let mut state = Self::new(snapshot.seed);
        state.legacy_mission_claim_pending = true;
        for wagon in ServiceWagon::ALL {
            if wagon.available(snapshot.stations_visited) {
                state.inventory.insert(ItemId::wagon(wagon));
            }
        }
        for livery in Livery::ALL {
            if livery.available(snapshot) || service.owns(livery) {
                state.inventory.insert(ItemId::livery(livery));
            }
        }
        if let Ok(catalog) = Catalog::builtin() {
            for engine in &catalog.locomotives {
                let owned = match engine.unlocked_by.as_deref() {
                    None => true,
                    Some("cargo_postcard") => service
                        .collection
                        .contains(&super::ServiceReward::CargoPostcard),
                    Some("passenger_postcard") => service
                        .collection
                        .contains(&super::ServiceReward::PassengerPostcard),
                    _ => false,
                };
                if owned {
                    state
                        .inventory
                        .insert(ItemId::Locomotive(engine.id.clone()));
                }
            }
        }
        state
    }
    pub fn owns(&self, item: &ItemId) -> bool {
        self.inventory.contains(item)
    }

    /// E1b may use this inside a staged World transaction for validated mission rewards.
    pub fn record(&mut self, reason: &str, credited: u64, debited: u64) -> Result<u64, String> {
        let balance = self
            .balance
            .checked_sub(debited)
            .ok_or("Réis insuficientes.")?
            .checked_add(credited)
            .ok_or("Limite de réis atingido.")?;
        let earned = self
            .total_earned
            .checked_add(credited)
            .ok_or("Limite de réis atingido.")?;
        let spent = self
            .total_spent
            .checked_add(debited)
            .ok_or("Limite de réis atingido.")?;
        let next = self
            .next_transaction_id
            .checked_add(1)
            .ok_or("Limite de transações atingido.")?;
        let id = self.next_transaction_id;
        self.balance = balance;
        self.total_earned = earned;
        self.total_spent = spent;
        self.next_transaction_id = next;
        if self.transactions.len() == TRANSACTION_LIMIT {
            self.transactions.pop_front();
        }
        self.transactions.push_back(Transaction {
            id,
            reason: reason.into(),
            credited,
            debited,
            balance,
        });
        Ok(id)
    }

    pub(crate) fn arrive(
        &mut self,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
        multiplier: u32,
    ) {
        let Some(arrival) = service.arrivals.last() else {
            return;
        };
        let passengers = self.passenger_remainder + u64::from(arrival.alighted);
        self.passenger_remainder = passengers % 4;
        let mut coins = 1 + passengers / 4;
        if arrival.cargo_delivered > 0
            && let Some(contract) = &service.contract
        {
            let reward = self.contract_remainder + contract.reward;
            self.contract_remainder = reward % 20;
            coins += (reward / 20).max(2);
            let count = self
                .cargo_by_kind
                .entry(contract.kind.id().into())
                .or_default();
            *count = count.saturating_add(u64::from(arrival.cargo_delivered));
        }
        let scaled =
            coins * u64::from(multiplier) * ARRIVAL_INCOME_MILLIS / 1000 + self.bonus_remainder;
        self.bonus_remainder = scaled % 1000;
        // Saturation is only reachable with manually constructed extreme profiles.
        let _ = self.record("Chegada e serviços", scaled / 1000, 0);
        if snapshot.completed_laps >= 1 {
            self.inventory.insert(ItemId::wagon(ServiceWagon::Sleeper));
        }
        if service
            .collection
            .contains(&super::ServiceReward::CargoPostcard)
        {
            self.inventory.insert(ItemId::Locomotive("diesel".into()));
        }
        for livery in Livery::ORIGINAL {
            if livery.available(snapshot) {
                self.inventory.insert(ItemId::livery(livery));
            }
        }
    }

    pub(super) fn random_below(&mut self, bound: u64) -> u64 {
        // SplitMix64 with rejection avoids modulo bias and the zero-seed trap.
        loop {
            self.random_state = self.random_state.wrapping_add(0x9e3779b97f4a7c15);
            let mut value = self.random_state;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
            value ^= value >> 31;
            if value >= bound.wrapping_neg() % bound {
                return value % bound;
            }
        }
    }
    pub(crate) fn apply(
        &mut self,
        action: EconomyAction,
        snapshot: JourneySnapshot,
        service: &ServiceState<impl Clone + Eq>,
    ) -> Result<(), String> {
        match action {
            EconomyAction::ClaimMission(id) => self.claim_mission(&id, snapshot, service)?,
            EconomyAction::ClaimOrder { order_id } => {
                self.claim_order(order_id, snapshot, service)?
            }
            EconomyAction::SwapOrder { order_id } => {
                self.swap_order(order_id, snapshot, service)?
            }
            EconomyAction::SettleLegacyMissions => {
                self.settle_legacy_missions(snapshot, service)?
            }
            EconomyAction::BuyMuseumStamp => {
                if self.pending_purchase.is_some() || !self.collection_complete() {
                    return Err(
                        "Complete a coleção e confira a remessa antes de comprar selos.".into(),
                    );
                }
                let next = self
                    .museum_stamps
                    .checked_add(1)
                    .ok_or("Limite de selos atingido.")?;
                self.record("Selo do museu", 0, 120)?;
                self.museum_stamps = next;
            }
            EconomyAction::Reveal { transaction_id } => {
                if self
                    .pending_purchase
                    .as_ref()
                    .is_none_or(|p| p.transaction_id != transaction_id)
                {
                    return Err("Esta remessa já foi revelada ou não existe.".into());
                }
                self.pending_purchase = None;
            }
            EconomyAction::BuyDirect(item) => {
                if self.pending_purchase.is_some() {
                    return Err("Confira a remessa pendente primeiro.".into());
                }
                let rarity = item.rarity().ok_or("Peça desconhecida.")?;
                if self.owns(&item) {
                    return Err("Você já possui este modelo.".into());
                }
                self.record("Compra direta", 0, rarity.direct_price())?;
                self.inventory.insert(item);
            }
            EconomyAction::BuyCrate { kind, free } => {
                if self.pending_purchase.is_some() {
                    return Err("Confira a remessa pendente primeiro.".into());
                }
                let definition = kind.definition()?;
                definition.validate()?;
                if free && (kind != CrateKind::GeneralYard || !self.welcome_crate_available) {
                    return Err("Remessa de boas-vindas já recolhida.".into());
                }
                let paid = if free { 0 } else { definition.price };
                if self.balance < paid {
                    return Err("Réis insuficientes.".into());
                }
                let misses = self.crate_misses.get(&kind).copied().unwrap_or(0);
                let weights = definition.effective_weights(misses);
                let mut draw = self.random_below(weights.iter().map(|&w| u64::from(w)).sum());
                let index = weights
                    .iter()
                    .position(|&weight| {
                        if draw < u64::from(weight) {
                            true
                        } else {
                            draw -= u64::from(weight);
                            false
                        }
                    })
                    .ok_or("Tabela de remessa inválida.")?;
                let rarity = Rarity::ALL[index];
                let pool: Vec<_> = definition
                    .items
                    .iter()
                    .filter(|i| i.rarity() == Some(rarity))
                    .cloned()
                    .collect();
                let fresh: Vec<_> = pool.iter().filter(|i| !self.owns(i)).cloned().collect();
                let duplicate = fresh.is_empty();
                let pool = if duplicate { pool } else { fresh };
                let item = pool[self.random_below(pool.len() as u64) as usize].clone();
                let refund = if duplicate { paid / 5 } else { 0 };
                let transaction_id = self.record("Remessa", refund, paid)?;
                self.inventory.insert(item.clone());
                self.crate_misses.insert(
                    kind,
                    if rarity >= Rarity::Rare {
                        0
                    } else {
                        misses + 1
                    },
                );
                if free {
                    self.welcome_crate_available = false;
                }
                self.pending_purchase = Some(PendingPurchase {
                    transaction_id,
                    crate_kind: kind,
                    pool_version: definition.version,
                    item,
                    rarity,
                    paid,
                    duplicate,
                    refund,
                });
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        let invalid = || "Economia inválida; arquivo preservado.".to_owned();
        if self.rule_version != ECONOMY_RULE_VERSION
            || self.total_earned.checked_sub(self.total_spent) != Some(self.balance)
            || self.next_transaction_id < 2
            || self.transactions.is_empty()
            || self.transactions.len() > TRANSACTION_LIMIT
            || self.inventory.len() > 256
            || self.inventory.iter().any(|i| i.rarity().is_none())
            || self.passenger_remainder >= 4
            || self.contract_remainder >= 20
            || self.bonus_remainder >= 1000
            || self.crate_misses.len() != CrateKind::ALL.len()
            || CrateKind::ALL
                .iter()
                .any(|k| !self.crate_misses.contains_key(k))
            || self
                .crate_misses
                .iter()
                .any(|(k, n)| k.definition().is_ok_and(|d| *n >= d.guarantee))
            || self.cargo_by_kind.len() != super::CargoKind::ALL.len()
            || super::CargoKind::ALL
                .iter()
                .any(|k| !self.cargo_by_kind.contains_key(k.id()))
            || self.paid_missions.len() > 1024
            || self
                .paid_missions
                .iter()
                .any(|id| id.is_empty() || id.len() > 96)
            || self.daily_orders.len() > 4
            || self.next_order_id == 0
            || !self.last_order_active_seconds.is_finite()
            || self.last_order_active_seconds < 0.0
            || !self.last_order_swap_active_seconds.is_finite()
            || self.last_order_swap_active_seconds < 0.0
        {
            return Err(invalid());
        }
        let mut previous: Option<&Transaction> = None;
        for transaction in &self.transactions {
            if transaction.id == 0
                || transaction.id >= self.next_transaction_id
                || transaction.reason.is_empty()
                || transaction.reason.len() > 256
                || previous.is_some_and(|p| {
                    p.id.checked_add(1) != Some(transaction.id)
                        || p.balance
                            .checked_sub(transaction.debited)
                            .and_then(|v| v.checked_add(transaction.credited))
                            != Some(transaction.balance)
                })
            {
                return Err(invalid());
            }
            previous = Some(transaction);
        }
        if previous
            .is_none_or(|t| t.id + 1 != self.next_transaction_id || t.balance != self.balance)
        {
            return Err(invalid());
        }
        if self.daily_orders.iter().filter(|o| !o.telegram).count() > 3
            || self.daily_orders.iter().filter(|o| o.telegram).count() > 1
            || self.daily_orders.iter().enumerate().any(|(index, o)| {
                o.id == 0
                    || o.id >= self.next_order_id
                    || self.daily_orders[..index]
                        .iter()
                        .any(|previous| previous.id == o.id)
                    || o.template_id.is_empty()
                    || o.template_id.len() > 96
                    || o.goal == 0
                    || o.reward == 0
                    || !o.entered_active_seconds.is_finite()
                    || o.entered_active_seconds < 0.0
                    || o.baselines.len() > 32
                    || o.baselines.keys().any(|id| id.is_empty() || id.len() > 96)
                    || o.template().is_none_or(|t| {
                        o.goal != t.goal
                            || o.reward != t.reward
                            || o.baselines.len() != 1
                            || !o.baselines.contains_key(t.counter.key())
                    })
            })
        {
            return Err(invalid());
        }
        if let Some(pending) = &self.pending_purchase {
            let definition = pending.crate_kind.definition()?;
            // Preserve paid v1 results byte-for-byte across the price calibration.
            let legacy = pending.crate_kind == CrateKind::GeneralYard && pending.pool_version == 1;
            let paid_price = if legacy { 60 } else { definition.price };
            if pending.transaction_id == 0
                || pending.transaction_id >= self.next_transaction_id
                || (!legacy && pending.pool_version != definition.version)
                || !definition.items.contains(&pending.item)
                || !self.owns(&pending.item)
                || pending.item.rarity() != Some(pending.rarity)
                || (pending.paid != 0 && pending.paid != paid_price)
                || pending.refund
                    != if pending.duplicate {
                        pending.paid / 5
                    } else {
                        0
                    }
                || (pending.paid == 0
                    && (pending.crate_kind != CrateKind::GeneralYard
                        || self.welcome_crate_available))
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_deliveries_keep_both_division_remainders_and_contract_minimum() {
        let snapshot = super::super::World::default().journey_snapshot();
        let mut service = ServiceState::migrated(snapshot);
        let mut state = EconomyState::new(42);
        service.contract = Some(super::super::CargoContract {
            id: 1,
            kind: super::super::CargoKind::Ore,
            origin: super::super::StationId::Aurora,
            destination: super::super::StationId::Pines,
            units: 1,
            reward: 11,
            status: super::super::ContractStatus::Delivered,
        });
        service.arrivals.push(super::super::ArrivalSummary {
            visit: 1,
            station: super::super::StationId::Pines,
            boarded: 0,
            alighted: 1,
            cargo_loaded: 0,
            cargo_delivered: 1,
            cargo_returned: 0,
            earned: 12,
        });
        for _ in 0..4 {
            state.arrive(snapshot, &service, 1000);
        }
        // Four arrivals + one passenger unit + four minimum contract awards, calibrated.
        assert_eq!(state.balance, 30 + 13 * ARRIVAL_INCOME_MILLIS / 1000);
        assert_eq!(state.bonus_remainder, 13 * ARRIVAL_INCOME_MILLIS % 1000);
        assert_eq!(state.passenger_remainder, 0);
        assert_eq!(state.contract_remainder, 4);
        assert_eq!(state.cargo_by_kind["ore"], 4);
        // A return/no delivery is only an arrival, with neither contract nor unit payment.
        service.arrivals[0].alighted = 0;
        service.arrivals[0].cargo_delivered = 0;
        service.arrivals[0].cargo_returned = 1;
        state.arrive(snapshot, &service, 1000);
        assert_eq!(state.balance, 30 + 14 * ARRIVAL_INCOME_MILLIS / 1000);
        assert_eq!(state.contract_remainder, 4);
        assert_eq!(state.cargo_by_kind["ore"], 4);
    }
}
