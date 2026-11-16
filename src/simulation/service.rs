//! Authoritative station transactions, independent of presentation and notifications.
use super::{JourneySnapshot, StationId, objectives::Livery};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceWagon {
    Passenger,
    Sleeper,
    Freight,
    Tank,
    Observation,
    Dining,
    Mail,
    Gondola,
    Hopper,
    Timber,
    Refrigerator,
    Livestock,
    Container,
    Autorack,
    PressureTank,
    Crane,
    Bicycle,
    Coil,
    Fair,
}

impl ServiceWagon {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Passenger => "passenger",
            Self::Sleeper => "sleeper",
            Self::Freight => "freight",
            Self::Tank => "tank",
            Self::Observation => "observation",
            Self::Dining => "dining",
            Self::Mail => "mail",
            Self::Gondola => "gondola",
            Self::Hopper => "hopper",
            Self::Timber => "timber",
            Self::Refrigerator => "refrigerator",
            Self::Livestock => "livestock",
            Self::Container => "container",
            Self::Autorack => "autorack",
            Self::PressureTank => "pressure_tank",
            Self::Crane => "crane",
            Self::Bicycle => "bicycle",
            Self::Coil => "coil",
            Self::Fair => "fair",
        }
    }

    pub const ALL: [Self; 19] = [
        Self::Passenger,
        Self::Sleeper,
        Self::Freight,
        Self::Tank,
        Self::Observation,
        Self::Dining,
        Self::Mail,
        Self::Gondola,
        Self::Hopper,
        Self::Timber,
        Self::Refrigerator,
        Self::Livestock,
        Self::Container,
        Self::Autorack,
        Self::PressureTank,
        Self::Crane,
        Self::Bicycle,
        Self::Coil,
        Self::Fair,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Passenger => "Passageiros",
            Self::Sleeper => "Dormitório",
            Self::Freight => "Carga",
            Self::Tank => "Tanque",
            Self::Observation => "Panorâmico",
            Self::Dining => "Restaurante",
            Self::Mail => "Correio e bagagem",
            Self::Gondola => "Gôndola de minério",
            Self::Hopper => "Graneleiro",
            Self::Timber => "Plataforma de toras",
            Self::Refrigerator => "Frigorífico",
            Self::Livestock => "Vagão de gado",
            Self::Container => "Porta-contêineres",
            Self::Autorack => "Cegonha de automóveis",
            Self::PressureTank => "Tanque de gás",
            Self::Crane => "Guindaste de socorro",
            Self::Bicycle => "Bagagem com bicicletas",
            Self::Coil => "Berço de bobinas",
            Self::Fair => "Feira itinerante",
        }
    }

    /// Provisional arrival milestones, derived from an already durable counter.
    pub const fn required_stops(self) -> u64 {
        match self {
            Self::Mail => 1,
            Self::Dining => 2,
            Self::Gondola => 4,
            Self::Hopper => 6,
            Self::Timber => 8,
            Self::Refrigerator => 10,
            Self::Livestock => 3,
            Self::Container => 5,
            Self::Autorack => 7,
            Self::PressureTank => 9,
            Self::Crane => 16,
            Self::Bicycle => 11,
            Self::Coil => 12,
            Self::Fair => 14,
            _ => 0,
        }
    }

    pub const fn available(self, arrivals: u64) -> bool {
        arrivals >= self.required_stops()
    }

    pub const fn cargo_capacity(self, kind: CargoKind) -> u32 {
        match (self, kind) {
            (Self::Freight, CargoKind::Parcels | CargoKind::Supplies)
            | (Self::Mail, CargoKind::Parcels)
            | (Self::Tank, CargoKind::Liquid)
            | (Self::Gondola, CargoKind::Ore)
            | (Self::Hopper, CargoKind::Grain)
            | (Self::Timber, CargoKind::Timber)
            | (Self::Refrigerator, CargoKind::Chilled)
            | (Self::Livestock, CargoKind::Livestock)
            | (Self::Container, CargoKind::Containers)
            | (Self::Autorack, CargoKind::Vehicles)
            | (Self::PressureTank, CargoKind::Gas)
            | (Self::Crane, CargoKind::Machinery)
            | (Self::Bicycle, CargoKind::Bicycles)
            | (Self::Coil, CargoKind::SteelCoils)
            | (Self::Fair, CargoKind::FairEquipment) => self.capacity().1,
            _ => 0,
        }
    }

    pub const fn capacity(self) -> (u32, u32) {
        match self {
            Self::Passenger => (24, 0),
            Self::Sleeper => (12, 0),
            Self::Observation => (16, 0),
            Self::Freight => (0, 20),
            Self::Tank => (0, 12),
            Self::Dining => (8, 0),
            Self::Mail => (0, 16),
            Self::Gondola | Self::Hopper => (0, 24),
            Self::Timber => (0, 18),
            Self::Refrigerator => (0, 12),
            Self::Livestock | Self::PressureTank | Self::Crane | Self::Autorack => (0, 12),
            Self::Container => (0, 20),
            Self::Bicycle | Self::Fair => (0, 16),
            Self::Coil => (0, 18),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassengerGroup<S = StationId> {
    pub origin: S,
    pub destination: S,
    pub count: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Offered,
    Accepted,
    Loaded,
    Delivered,
    Returning,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoKind {
    Parcels,
    Supplies,
    Liquid,
    Ore,
    Grain,
    Timber,
    Chilled,
    Livestock,
    Containers,
    Vehicles,
    Gas,
    Machinery,
    Bicycles,
    SteelCoils,
    FairEquipment,
}
impl CargoKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Parcels => "parcels",
            Self::Supplies => "supplies",
            Self::Liquid => "liquid",
            Self::Ore => "ore",
            Self::Grain => "grain",
            Self::Timber => "timber",
            Self::Chilled => "chilled",
            Self::Livestock => "livestock",
            Self::Containers => "containers",
            Self::Vehicles => "vehicles",
            Self::Gas => "gas",
            Self::Machinery => "machinery",
            Self::Bicycles => "bicycles",
            Self::SteelCoils => "steel_coils",
            Self::FairEquipment => "fair_equipment",
        }
    }

    pub const ALL: [Self; 15] = [
        Self::Parcels,
        Self::Supplies,
        Self::Liquid,
        Self::Ore,
        Self::Grain,
        Self::Timber,
        Self::Chilled,
        Self::Livestock,
        Self::Containers,
        Self::Vehicles,
        Self::Gas,
        Self::Machinery,
        Self::Bicycles,
        Self::SteelCoils,
        Self::FairEquipment,
    ];
    pub const SPECIALIZED: [Self; 12] = [
        Self::Ore,
        Self::Grain,
        Self::Timber,
        Self::Chilled,
        Self::Livestock,
        Self::Containers,
        Self::Vehicles,
        Self::Gas,
        Self::Machinery,
        Self::Bicycles,
        Self::SteelCoils,
        Self::FairEquipment,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Parcels => "Encomendas",
            Self::Supplies => "Suprimentos",
            Self::Liquid => "Líquidos",
            Self::Ore => "Minério",
            Self::Grain => "Grãos",
            Self::Timber => "Madeira",
            Self::Chilled => "Refrigerados",
            Self::Livestock => "Gado",
            Self::Containers => "Contêineres lacrados",
            Self::Vehicles => "Automóveis",
            Self::Gas => "Gás pressurizado",
            Self::Machinery => "Máquinas de manutenção",
            Self::Bicycles => "Bicicletas",
            Self::SteelCoils => "Bobinas de aço",
            Self::FairEquipment => "Equipamentos de feira",
        }
    }
    pub const fn wagon_hint(self) -> &'static str {
        match self {
            Self::Parcels => "vagão de carga (20 un.) ou correio (16 un.)",
            Self::Supplies => "vagão de carga (20 un.)",
            Self::Liquid => "vagão-tanque (12 un.)",
            Self::Ore => "gôndola de minério (24 un.)",
            Self::Grain => "graneleiro (24 un.)",
            Self::Timber => "plataforma de toras (18 un.)",
            Self::Chilled => "frigorífico (12 un.)",
            Self::Livestock => "vagão de gado (12 un.)",
            Self::Containers => "porta-contêineres (20 un.)",
            Self::Vehicles => "cegonha de automóveis (12 un.)",
            Self::Gas => "tanque de gás (12 un.)",
            Self::Machinery => "guindaste de socorro (12 un.)",
            Self::Bicycles => "bagagem com bicicletas (16 un.)",
            Self::SteelCoils => "berço de bobinas (18 un.)",
            Self::FairEquipment => "feira itinerante (16 un.)",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceReward {
    PassengerPostcard,
    CargoPostcard,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoContract<S = StationId> {
    pub kind: CargoKind,
    pub id: u64,
    pub origin: S,
    pub destination: S,
    pub units: u32,
    pub reward: u64,
    pub status: ContractStatus,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalSummary<S = StationId> {
    pub visit: u64,
    pub station: S,
    pub boarded: u32,
    pub alighted: u32,
    pub cargo_loaded: u32,
    pub cargo_delivered: u32,
    #[serde(default)]
    pub cargo_returned: u32,
    pub earned: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceState<S = StationId> {
    #[serde(default)]
    pub branch: super::BranchState,
    pub composition: [ServiceWagon; 3],
    pub passengers: Vec<PassengerGroup<S>>,
    #[serde(default = "automatic_default")]
    pub automatic_contracts: bool,
    /// At most one contract; a delivered contract remains visible until next arrival.
    pub contract: Option<CargoContract<S>>,
    pub last_processed_visit: u64,
    pub passengers_boarded: u64,
    pub passengers_delivered: u64,
    pub cargo_delivered: u64,
    pub contracts_completed: u64,
    /// Cumulative service points, not spendable currency.
    pub credits: u64,
    pub owned_rewards: Vec<Livery>,
    pub collection: Vec<ServiceReward>,
    pub arrivals: Vec<ArrivalSummary<S>>,
}
fn automatic_default() -> bool {
    true
}

/// Deliveries that earn each service postcard; mirrored by the content catalogue.
pub const PASSENGER_POSTCARD_GOAL: u64 = 20;
pub const CARGO_POSTCARD_GOAL: u64 = 20;

const STATIONS: [StationId; 4] = [
    StationId::Aurora,
    StationId::Pines,
    StationId::Ridge,
    StationId::Harbor,
];
impl<S: Clone + Eq> ServiceState<S> {
    /// Presentation/economy projection; durable identities remain in each line's service.
    pub fn map_stations<T: Clone + Eq>(&self, map: impl Fn(&S) -> T) -> ServiceState<T> {
        ServiceState {
            branch: self.branch.clone(),
            composition: self.composition,
            passengers: self
                .passengers
                .iter()
                .map(|p| PassengerGroup {
                    origin: map(&p.origin),
                    destination: map(&p.destination),
                    count: p.count,
                })
                .collect(),
            automatic_contracts: self.automatic_contracts,
            contract: self.contract.as_ref().map(|c| CargoContract {
                kind: c.kind,
                id: c.id,
                origin: map(&c.origin),
                destination: map(&c.destination),
                units: c.units,
                reward: c.reward,
                status: c.status,
            }),
            last_processed_visit: self.last_processed_visit,
            passengers_boarded: self.passengers_boarded,
            passengers_delivered: self.passengers_delivered,
            cargo_delivered: self.cargo_delivered,
            contracts_completed: self.contracts_completed,
            credits: self.credits,
            owned_rewards: self.owned_rewards.clone(),
            collection: self.collection.clone(),
            arrivals: self
                .arrivals
                .iter()
                .map(|a| ArrivalSummary {
                    visit: a.visit,
                    station: map(&a.station),
                    boarded: a.boarded,
                    alighted: a.alighted,
                    cargo_loaded: a.cargo_loaded,
                    cargo_delivered: a.cargo_delivered,
                    cargo_returned: a.cargo_returned,
                    earned: a.earned,
                })
                .collect(),
        }
    }
    pub fn capacity(&self) -> (u32, u32) {
        self.composition.iter().fold((0, 0), |(p, c), w| {
            let (a, b) = w.capacity();
            (p + a, c + b)
        })
    }
    pub fn cargo_capacity(&self, kind: CargoKind) -> u32 {
        self.composition
            .iter()
            .map(|w| w.cargo_capacity(kind))
            .sum()
    }
    fn cargo_fits(&self) -> bool {
        self.contract.as_ref().is_none_or(|c| {
            !matches!(c.status, ContractStatus::Loaded | ContractStatus::Returning)
                || c.units <= self.cargo_capacity(c.kind)
        })
    }
    pub fn passenger_count(&self) -> u32 {
        self.passengers.iter().map(|p| p.count).sum()
    }
    pub fn cargo_count(&self) -> u32 {
        self.contract
            .as_ref()
            .filter(|c| matches!(c.status, ContractStatus::Loaded | ContractStatus::Returning))
            .map_or(0, |c| c.units)
    }
    /// Reject downsizing occupied stock; never silently discard passengers or cargo.
    pub fn set_composition(&mut self, composition: [ServiceWagon; 3]) -> Result<(), String> {
        if let Some(wagon) = composition
            .iter()
            .find(|w| !w.available(self.last_processed_visit))
        {
            return Err(format!(
                "{}: complete {} chegadas para liberar.",
                wagon.name(),
                wagon.required_stops()
            ));
        }
        self.set_owned_composition(composition)
    }
    /// Ownership is checked by World; service checks only occupied capacity.
    pub fn can_carry_with(&self, composition: [ServiceWagon; 3]) -> bool {
        let seats: u32 = composition.iter().map(|w| w.capacity().0).sum();
        self.passenger_count() <= seats
            && self.contract.as_ref().is_none_or(|c| {
                !matches!(c.status, ContractStatus::Loaded | ContractStatus::Returning)
                    || c.units
                        <= composition
                            .iter()
                            .map(|w| w.cargo_capacity(c.kind))
                            .sum::<u32>()
            })
    }
    /// Ownership is checked by World; service checks only occupied capacity.
    pub(crate) fn set_owned_composition(
        &mut self,
        composition: [ServiceWagon; 3],
    ) -> Result<(), String> {
        let old = self.composition;
        self.composition = composition;
        let (p, c) = self.capacity();
        if self.passenger_count() > p || self.cargo_count() > c || !self.cargo_fits() {
            self.composition = old;
            return Err("Capacidade insuficiente para o serviço a bordo.".into());
        }
        Ok(())
    }
    pub fn acceptance_error(&self) -> Option<String> {
        let c = self.contract.as_ref()?;
        if c.status != ContractStatus::Offered {
            return Some("Esta oferta já foi resolvida.".into());
        }
        if c.units > self.cargo_capacity(c.kind) {
            return Some(format!(
                "São necessárias {} unidades de capacidade compatível. Ajuste os vagões na oficina.",
                c.units
            ));
        }
        None
    }
    pub(crate) fn accept_contract(&mut self, platform: Option<S>) -> Result<(), String> {
        if self.contract.is_none() {
            return Err("Não há oferta disponível.".into());
        }
        if let Some(error) = self.acceptance_error() {
            return Err(error);
        }
        self.contract.as_mut().unwrap().status = ContractStatus::Accepted;
        self.load_at_platform(platform);
        Ok(())
    }
    pub(crate) fn cancel_contract(&mut self, platform: Option<S>) -> Result<(), String> {
        let c = self
            .contract
            .as_mut()
            .ok_or("Não há contrato para cancelar.")?;
        c.status = match c.status {
            ContractStatus::Offered | ContractStatus::Accepted => ContractStatus::Cancelled,
            ContractStatus::Loaded => ContractStatus::Returning,
            _ => return Err("Este contrato já está resolvido ou em devolução.".into()),
        };
        self.return_at_platform(platform);
        Ok(())
    }
    fn load_at_platform(&mut self, platform: Option<S>) {
        let hold = self
            .contract
            .as_ref()
            .map_or(0, |c| self.cargo_capacity(c.kind));
        if let Some(c) = &mut self.contract
            && c.status == ContractStatus::Accepted
            && platform.as_ref() == Some(&c.origin)
            && c.units <= hold
        {
            c.status = ContractStatus::Loaded;
            if let Some(a) = self.arrivals.last_mut() {
                a.cargo_loaded += c.units;
            }
        }
    }
    fn return_at_platform(&mut self, platform: Option<S>) {
        if let Some(c) = &mut self.contract
            && c.status == ContractStatus::Returning
            && platform.as_ref() == Some(&c.origin)
        {
            c.status = ContractStatus::Cancelled;
            if let Some(a) = self.arrivals.last_mut() {
                a.cargo_returned += c.units;
            }
        }
    }
    pub fn owns(&self, reward: Livery) -> bool {
        self.owned_rewards.contains(&reward)
    }
    pub(crate) fn validate_route(
        &self,
        visits: u64,
        station_at_visit: impl Fn(u64) -> S,
        contains: impl Fn(&S) -> bool,
        max_groups: usize,
    ) -> Result<(), String> {
        let invalid = || "Estado de serviços inválido; arquivo preservado.".to_owned();
        if self.last_processed_visit != visits
            || self.passengers.len() > max_groups
            || self.arrivals.len() > 32
            || self.owned_rewards.len() > Livery::ALL.len()
        {
            return Err(invalid());
        }
        if self.passengers_boarded > self.last_processed_visit.saturating_mul(72) {
            return Err(invalid());
        }
        if self.passengers.iter().any(|p| {
            p.count == 0
                || p.count > 72
                || p.origin == p.destination
                || !contains(&p.origin)
                || !contains(&p.destination)
        }) {
            return Err(invalid());
        }
        let (seats, hold) = self.capacity();
        if !self.cargo_fits()
            || self.passenger_count() > seats
            || self.cargo_count() > hold
            || self.passengers_delivered > self.passengers_boarded
            || self.passengers_boarded - self.passengers_delivered
                != u64::from(self.passenger_count())
        {
            return Err(invalid());
        }
        if self
            .owned_rewards
            .iter()
            .enumerate()
            .any(|(i, r)| self.owned_rewards[..i].contains(r))
            || !self.owns(Livery::Aurora)
        {
            return Err(invalid());
        }
        if self.collection.len() > 2
            || self
                .collection
                .iter()
                .enumerate()
                .any(|(i, r)| self.collection[..i].contains(r))
            || self.collection.contains(&ServiceReward::PassengerPostcard)
                != (self.passengers_delivered >= PASSENGER_POSTCARD_GOAL)
            || self.collection.contains(&ServiceReward::CargoPostcard)
                != (self.cargo_delivered >= CARGO_POSTCARD_GOAL)
        {
            return Err(invalid());
        }
        if let Some(c) = &self.contract
            && (c.id == 0
                || c.id > self.last_processed_visit
                || c.origin == c.destination
                || c.origin != station_at_visit(c.id)
                || !contains(&c.destination)
                || !(1..=12).contains(&c.units)
                || c.reward != u64::from(c.units) * 10)
        {
            return Err(invalid());
        }
        if let Some(c) = &self.contract {
            let mut loaded = 0_u64;
            let mut delivered = 0_u64;
            let mut returned = 0_u64;
            for a in self.arrivals.iter().filter(|a| a.visit >= c.id) {
                loaded += u64::from(a.cargo_loaded);
                delivered += u64::from(a.cargo_delivered);
                returned += u64::from(a.cargo_returned);
            }
            if loaded > u64::from(c.units)
                || delivered > u64::from(c.units)
                || returned > u64::from(c.units)
                || (returned > 0 && c.status != ContractStatus::Cancelled)
                || (delivered > 0 && c.status != ContractStatus::Delivered)
                || returned + delivered > u64::from(c.units)
            {
                return Err(invalid());
            }
            // When the bounded log includes creation, it contains the entire lot's history.
            if self.arrivals.first().is_some_and(|a| a.visit <= c.id)
                && loaded.checked_sub(delivered + returned) != Some(u64::from(self.cargo_count()))
            {
                return Err(invalid());
            }
        }
        if self.credits
            != self
                .passengers_delivered
                .saturating_mul(2)
                .saturating_add(self.cargo_delivered.saturating_mul(10))
            || self.contracts_completed > self.last_processed_visit
            || self.cargo_delivered > self.contracts_completed.saturating_mul(12)
            || self.cargo_delivered < self.contracts_completed
        {
            return Err(invalid());
        }
        for (i, a) in self.arrivals.iter().enumerate() {
            if a.visit == 0
                || a.visit > self.last_processed_visit
                || a.station != station_at_visit(a.visit)
                || a.boarded > 72
                || a.alighted > 72
                || a.cargo_loaded > 12
                || a.cargo_delivered > 12
                || a.cargo_returned > 12
                || a.earned != u64::from(a.alighted) * 2 + u64::from(a.cargo_delivered) * 10
                || (i > 0 && self.arrivals[i - 1].visit + 1 != a.visit)
            {
                return Err(invalid());
            }
        }
        if self
            .arrivals
            .last()
            .is_some_and(|a| a.visit != self.last_processed_visit)
        {
            return Err(invalid());
        }
        Ok(())
    }
    /// Shared transaction; the route owner supplies stable endpoints and visit identity.
    pub(crate) fn arrive_at(&mut self, seed: u64, visit: u64, station: S, destination: S) {
        self.arrive_at_with_boarding(seed, visit, station, destination, true);
    }
    pub(crate) fn arrive_at_with_boarding(
        &mut self,
        seed: u64,
        visit: u64,
        station: S,
        destination: S,
        boarding: bool,
    ) {
        if visit <= self.last_processed_visit {
            return;
        }
        let mut summary = ArrivalSummary {
            visit,
            station: station.clone(),
            boarded: 0,
            alighted: 0,
            cargo_loaded: 0,
            cargo_delivered: 0,
            cargo_returned: 0,
            earned: 0,
        };
        self.passengers.retain(|p| {
            if p.destination == station {
                summary.alighted += p.count;
                false
            } else {
                true
            }
        });
        self.passengers_delivered += u64::from(summary.alighted);
        // Replace only an already-settled contract, never replay its reward.
        if self.contract.as_ref().is_some_and(|c| {
            matches!(
                c.status,
                ContractStatus::Delivered | ContractStatus::Cancelled
            )
        }) {
            self.contract = None;
        }
        if let Some(c) = &mut self.contract
            && c.status == ContractStatus::Returning
            && c.origin == station
        {
            c.status = ContractStatus::Cancelled;
            summary.cargo_returned = c.units;
        }
        if let Some(c) = &mut self.contract
            && c.status == ContractStatus::Loaded
            && c.destination == station
        {
            c.status = ContractStatus::Delivered;
            summary.cargo_delivered = c.units;
            self.cargo_delivered += u64::from(c.units);
            self.contracts_completed += 1;
        }
        let entropy = seed.wrapping_add(visit.wrapping_mul(0x9e3779b97f4a7c15));
        let (seats, _) = self.capacity();
        let count = if boarding {
            (4 + (entropy % 17) as u32).min(seats - self.passenger_count())
        } else {
            0
        };
        if count > 0 {
            if let Some(group) = self
                .passengers
                .iter_mut()
                .find(|p| p.origin == station && p.destination == destination)
            {
                group.count += count;
            } else {
                self.passengers.push(PassengerGroup {
                    origin: station.clone(),
                    destination: destination.clone(),
                    count,
                });
            }
            summary.boarded = count;
            self.passengers_boarded += u64::from(count);
        }
        if boarding && self.contract.is_none() {
            let units = 4 + ((entropy >> 8) % 9) as u32;
            let kind = if self.cargo_capacity(CargoKind::Liquid) > 0
                && (self.cargo_capacity(CargoKind::Parcels) == 0 || (entropy >> 16) % 3 == 2)
            {
                CargoKind::Liquid
            } else if (entropy >> 16).is_multiple_of(2) {
                CargoKind::Parcels
            } else {
                CargoKind::Supplies
            };
            // Old compositions keep the exact entropy, type and unit sequence.
            // Specialized offers are opt-in by equipping their wagon. With mixed
            // stock, every other offer keeps the legacy draw (when transportable).
            let mut compatible = CargoKind::SPECIALIZED
                .into_iter()
                .filter(|kind| self.cargo_capacity(*kind) >= units);
            let count = compatible.clone().count();
            let kind = if count > 0
                && (self.cargo_capacity(kind) < units || (entropy >> 24).is_multiple_of(2))
            {
                compatible
                    .nth(((entropy >> 32) % count as u64) as usize)
                    .unwrap()
            } else if self.composition.contains(&ServiceWagon::Mail)
                && self.cargo_capacity(kind) < units
                && self.cargo_capacity(CargoKind::Parcels) >= units
            {
                CargoKind::Parcels
            } else {
                kind
            };
            self.contract = Some(CargoContract {
                kind,
                id: visit,
                origin: station.clone(),
                destination,
                units,
                reward: u64::from(units) * 10,
                status: if self.automatic_contracts {
                    ContractStatus::Accepted
                } else {
                    ContractStatus::Offered
                },
            });
        }
        if boarding
            && self.automatic_contracts
            && let Some(c) = &mut self.contract
            && c.status == ContractStatus::Offered
        {
            c.status = ContractStatus::Accepted;
        }
        let hold = self
            .contract
            .as_ref()
            .map_or(0, |c| self.cargo_capacity(c.kind));
        if let Some(c) = &mut self.contract
            && c.status == ContractStatus::Accepted
            && c.origin == station
            && c.units <= hold
        {
            c.status = ContractStatus::Loaded;
            summary.cargo_loaded = c.units;
        }
        summary.earned = u64::from(summary.alighted) * 2 + u64::from(summary.cargo_delivered) * 10;
        self.credits += summary.earned;
        for (reward, unlocked) in [
            (
                ServiceReward::PassengerPostcard,
                self.passengers_delivered >= PASSENGER_POSTCARD_GOAL,
            ),
            (
                ServiceReward::CargoPostcard,
                self.cargo_delivered >= CARGO_POSTCARD_GOAL,
            ),
        ] {
            if unlocked && !self.collection.contains(&reward) {
                self.collection.push(reward);
            }
        }
        if self.arrivals.len() == 32 {
            self.arrivals.remove(0);
        }
        self.arrivals.push(summary);
        self.last_processed_visit = visit;
    }
}

// Aurora keeps its original public API, reward rules and serialized station enums.
impl ServiceState {
    pub fn migrated(snapshot: JourneySnapshot) -> Self {
        Self {
            branch: super::BranchState::default(),
            composition: [
                ServiceWagon::Observation,
                ServiceWagon::Sleeper,
                ServiceWagon::Freight,
            ],
            passengers: Vec::new(),
            contract: None,
            automatic_contracts: true,
            last_processed_visit: snapshot.stations_visited,
            passengers_boarded: 0,
            passengers_delivered: 0,
            cargo_delivered: 0,
            contracts_completed: 0,
            credits: 0,
            collection: Vec::new(),
            // Legacy migration restores only the original earned content.
            // Later paints are granted by the next arrival, as with older v4.
            owned_rewards: Livery::ORIGINAL
                .into_iter()
                .filter(|l| l.available(snapshot))
                .collect(),
            arrivals: Vec::new(),
        }
    }

    pub fn validate(&self, snapshot: JourneySnapshot) -> Result<(), String> {
        self.branch.validate(snapshot)?;
        self.validate_route(
            snapshot.stations_visited,
            |visit| STATIONS[(visit % STATIONS.len() as u64) as usize],
            |station| STATIONS.contains(station),
            12,
        )?;
        if Livery::ORIGINAL
            .iter()
            .any(|r| r.available(snapshot) && !self.owns(*r))
        {
            return Err("Estado de serviços inválido; arquivo preservado.".into());
        }
        Ok(())
    }
    pub(crate) fn arrive(&mut self, snapshot: JourneySnapshot) {
        self.arrive_with_boarding(snapshot, true);
    }
    pub(crate) fn arrive_with_boarding(&mut self, snapshot: JourneySnapshot, boarding: bool) {
        let visit = snapshot.stations_visited;
        if visit <= self.last_processed_visit {
            return;
        }
        let station = STATIONS[(visit % STATIONS.len() as u64) as usize];
        let entropy = snapshot
            .seed
            .wrapping_add(visit.wrapping_mul(0x9e3779b97f4a7c15));
        let destination = STATIONS[(station.index() + 1 + (entropy % 3) as usize) % STATIONS.len()];
        self.arrive_at_with_boarding(snapshot.seed, visit, station, destination, boarding);
        for reward in Livery::ORIGINAL {
            if reward.available(snapshot) && !self.owns(reward) {
                self.owned_rewards.push(reward);
            }
        }
    }
}

#[cfg(test)]
mod content_tests {
    use super::*;

    #[test]
    fn old_stock_keeps_offer_kind_units_destination_and_reward() {
        let legacy = [
            ServiceWagon::Passenger,
            ServiceWagon::Sleeper,
            ServiceWagon::Observation,
            ServiceWagon::Freight,
            ServiceWagon::Tank,
        ];
        for a in legacy {
            for b in legacy {
                for c in legacy {
                    for seed in [0, 1, 42, u64::MAX] {
                        for visit in 1_u64..=16 {
                            let mut snapshot =
                                super::super::World::with_seed(seed).journey_snapshot();
                            snapshot.stations_visited = visit;
                            let mut state = ServiceState::migrated(snapshot);
                            state.last_processed_visit = visit - 1;
                            state.composition = [a, b, c];
                            let entropy = seed.wrapping_add(visit.wrapping_mul(0x9e3779b97f4a7c15));
                            let parcels = [a, b, c].contains(&ServiceWagon::Freight);
                            let liquid = [a, b, c].contains(&ServiceWagon::Tank);
                            let kind = if liquid && (!parcels || (entropy >> 16) % 3 == 2) {
                                CargoKind::Liquid
                            } else if (entropy >> 16).is_multiple_of(2) {
                                CargoKind::Parcels
                            } else {
                                CargoKind::Supplies
                            };
                            state.arrive(snapshot);
                            let contract = state.contract.unwrap();
                            assert_eq!(contract.kind, kind);
                            assert_eq!(contract.units, 4 + ((entropy >> 8) % 9) as u32);
                            assert_eq!(contract.reward, u64::from(contract.units) * 10);
                            assert_eq!(contract.origin, STATIONS[(visit % 4) as usize]);
                            assert_eq!(
                                contract.destination,
                                STATIONS[((visit % 4) as usize + 1 + (entropy % 3) as usize) % 4]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn mixed_stock_still_offers_legacy_and_each_equipped_specialty() {
        let mut found = [false; 4];
        for visit in 20_u64..200 {
            let mut snapshot = super::super::World::with_seed(42).journey_snapshot();
            snapshot.stations_visited = visit;
            let mut state = ServiceState::migrated(snapshot);
            state.last_processed_visit = visit - 1;
            state.composition = [
                ServiceWagon::Freight,
                ServiceWagon::Gondola,
                ServiceWagon::Hopper,
            ];
            state.arrive(snapshot);
            match state.contract.unwrap().kind {
                CargoKind::Parcels => found[0] = true,
                CargoKind::Supplies => found[1] = true,
                CargoKind::Ore => found[2] = true,
                CargoKind::Grain => found[3] = true,
                other => panic!("unexpected unequipped cargo: {other:?}"),
            }
        }
        assert_eq!(found, [true; 4]);
    }
}
