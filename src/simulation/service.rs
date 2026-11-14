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
}
impl ServiceWagon {
    pub const fn capacity(self) -> (u32, u32) {
        match self {
            Self::Passenger => (24, 0),
            Self::Sleeper => (12, 0),
            Self::Observation => (16, 0),
            Self::Freight => (0, 20),
            Self::Tank => (0, 12),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassengerGroup {
    pub origin: StationId,
    pub destination: StationId,
    pub count: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Accepted,
    Loaded,
    Delivered,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoKind {
    Parcels,
    Supplies,
    Liquid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceReward {
    PassengerPostcard,
    CargoPostcard,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoContract {
    pub kind: CargoKind,
    pub id: u64,
    pub origin: StationId,
    pub destination: StationId,
    pub units: u32,
    pub reward: u64,
    pub status: ContractStatus,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalSummary {
    pub visit: u64,
    pub station: StationId,
    pub boarded: u32,
    pub alighted: u32,
    pub cargo_loaded: u32,
    pub cargo_delivered: u32,
    pub earned: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceState {
    #[serde(default)]
    pub branch: super::BranchState,
    pub composition: [ServiceWagon; 3],
    pub passengers: Vec<PassengerGroup>,
    /// At most one automatic contract; a delivered contract remains visible until next arrival.
    pub contract: Option<CargoContract>,
    pub last_processed_visit: u64,
    pub passengers_boarded: u64,
    pub passengers_delivered: u64,
    pub cargo_delivered: u64,
    pub contracts_completed: u64,
    /// Cumulative service points, not spendable currency.
    pub credits: u64,
    pub owned_rewards: Vec<Livery>,
    pub collection: Vec<ServiceReward>,
    pub arrivals: Vec<ArrivalSummary>,
}
const STATIONS: [StationId; 4] = [
    StationId::Aurora,
    StationId::Pines,
    StationId::Ridge,
    StationId::Harbor,
];
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
            last_processed_visit: snapshot.stations_visited,
            passengers_boarded: 0,
            passengers_delivered: 0,
            cargo_delivered: 0,
            contracts_completed: 0,
            credits: 0,
            collection: Vec::new(),
            owned_rewards: Livery::ALL
                .into_iter()
                .filter(|l| l.available(snapshot))
                .collect(),
            arrivals: Vec::new(),
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
            .map(|w| match (w, kind) {
                (ServiceWagon::Freight, CargoKind::Parcels | CargoKind::Supplies) => 20,
                (ServiceWagon::Tank, CargoKind::Liquid) => 12,
                _ => 0,
            })
            .sum()
    }
    fn cargo_fits(&self) -> bool {
        self.contract.as_ref().is_none_or(|c| {
            c.status != ContractStatus::Loaded || c.units <= self.cargo_capacity(c.kind)
        })
    }
    pub fn passenger_count(&self) -> u32 {
        self.passengers.iter().map(|p| p.count).sum()
    }
    pub fn cargo_count(&self) -> u32 {
        self.contract
            .as_ref()
            .filter(|c| c.status == ContractStatus::Loaded)
            .map_or(0, |c| c.units)
    }
    /// Reject downsizing occupied stock; never silently discard passengers or cargo.
    pub fn set_composition(&mut self, composition: [ServiceWagon; 3]) -> Result<(), String> {
        let old = self.composition;
        self.composition = composition;
        let (p, c) = self.capacity();
        if self.passenger_count() > p || self.cargo_count() > c || !self.cargo_fits() {
            self.composition = old;
            return Err("Capacidade insuficiente para o serviço a bordo.".into());
        }
        Ok(())
    }
    pub fn owns(&self, reward: Livery) -> bool {
        self.owned_rewards.contains(&reward)
    }
    pub fn validate(&self, snapshot: JourneySnapshot) -> Result<(), String> {
        self.branch.validate(snapshot)?;
        let invalid = || "Estado de serviços inválido; arquivo preservado.".to_owned();
        if self.last_processed_visit != snapshot.stations_visited
            || self.passengers.len() > 12
            || self.arrivals.len() > 32
            || self.owned_rewards.len() > 4
        {
            return Err(invalid());
        }
        if self.passengers_boarded > self.last_processed_visit.saturating_mul(72) {
            return Err(invalid());
        }
        if self
            .passengers
            .iter()
            .any(|p| p.count == 0 || p.count > 72 || p.origin == p.destination)
        {
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
        if Livery::ALL
            .iter()
            .any(|r| r.available(snapshot) && !self.owns(*r))
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
                != (self.passengers_delivered >= 20)
            || self.collection.contains(&ServiceReward::CargoPostcard)
                != (self.cargo_delivered >= 20)
        {
            return Err(invalid());
        }
        if let Some(c) = &self.contract
            && (c.id == 0
                || c.id > self.last_processed_visit
                || c.origin == c.destination
                || c.origin != STATIONS[(c.id % 4) as usize]
                || !(1..=12).contains(&c.units)
                || c.reward != u64::from(c.units) * 10)
        {
            return Err(invalid());
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
                || a.station != STATIONS[(a.visit % 4) as usize]
                || a.boarded > 72
                || a.alighted > 72
                || a.cargo_loaded > 12
                || a.cargo_delivered > 12
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
    /// Single synchronous transaction before any visual event consumption.
    pub(crate) fn arrive(&mut self, snapshot: JourneySnapshot) {
        let visit = snapshot.stations_visited;
        if visit <= self.last_processed_visit {
            return;
        }
        let station = STATIONS[(visit % 4) as usize];
        let mut summary = ArrivalSummary {
            visit,
            station,
            boarded: 0,
            alighted: 0,
            cargo_loaded: 0,
            cargo_delivered: 0,
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
        if self
            .contract
            .as_ref()
            .is_some_and(|c| c.status == ContractStatus::Delivered)
        {
            self.contract = None;
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
        let entropy = snapshot
            .seed
            .wrapping_add(visit.wrapping_mul(0x9e3779b97f4a7c15));
        let destination = STATIONS[(station.index() + 1 + (entropy % 3) as usize) % 4];
        let (seats, _) = self.capacity();
        let count = (4 + (entropy % 17) as u32).min(seats - self.passenger_count());
        if count > 0 {
            if let Some(group) = self
                .passengers
                .iter_mut()
                .find(|p| p.origin == station && p.destination == destination)
            {
                group.count += count;
            } else {
                self.passengers.push(PassengerGroup {
                    origin: station,
                    destination,
                    count,
                });
            }
            summary.boarded = count;
            self.passengers_boarded += u64::from(count);
        }
        if self.contract.is_none() {
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
            self.contract = Some(CargoContract {
                kind,
                id: visit,
                origin: station,
                destination,
                units,
                reward: u64::from(units) * 10,
                status: ContractStatus::Accepted,
            });
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
        for reward in Livery::ALL {
            if reward.available(snapshot) && !self.owns(reward) {
                self.owned_rewards.push(reward);
            }
        }
        for (reward, unlocked) in [
            (
                ServiceReward::PassengerPostcard,
                self.passengers_delivered >= 20,
            ),
            (ServiceReward::CargoPostcard, self.cargo_delivered >= 20),
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
