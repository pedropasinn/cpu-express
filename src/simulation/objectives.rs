//! Small authored objective catalogue. Rewards are derived from durable journey
//! counters, so loading or replaying UI notifications cannot grant them twice.
use super::JourneySnapshot;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Livery {
    #[default]
    Aurora,
    Ocean,
    Sunset,
    Midnight,
}

impl Livery {
    pub const ALL: [Self; 4] = [Self::Aurora, Self::Ocean, Self::Sunset, Self::Midnight];
    pub fn name(self) -> &'static str {
        match self {
            Self::Aurora => "Verde Aurora",
            Self::Ocean => "Azul do Porto",
            Self::Sunset => "Cobre do Poente",
            Self::Midnight => "Expresso Noturno",
        }
    }
    pub fn required_stops(self) -> u64 {
        match self {
            Self::Aurora => 0,
            Self::Ocean => 1,
            Self::Sunset => 4,
            Self::Midnight => 12,
        }
    }
    pub fn available(self, journey: JourneySnapshot) -> bool {
        journey.stations_visited >= self.required_stops()
    }
}

pub struct Objective {
    pub id: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    pub goal: u64,
    pub reward: Livery,
}

pub const OBJECTIVES: [Objective; 3] = [
    Objective {
        id: "ce.first-stop",
        title: "Primeiro serviço",
        detail: "Chegue à próxima estação e conheça a plataforma.",
        goal: 1,
        reward: Livery::Ocean,
    },
    Objective {
        id: "ce.aurora-circuit",
        title: "Cartão de explorador",
        detail: "Visite as quatro estações do Circuito Aurora.",
        goal: 4,
        reward: Livery::Sunset,
    },
    Objective {
        id: "ce.regular-service",
        title: "Uma ferrovia de confiança",
        detail: "Complete doze chegadas, sem prazo nem pressa.",
        goal: 12,
        reward: Livery::Midnight,
    },
];

/// Read-only milestones. Currency proposals live in docs/MISSOES.md, never in
/// the v4 reward transaction. No claim button or ephemeral event is a counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissionCounter {
    Passengers,
    Cargo,
    Contracts,
    Stations,
    Branches,
    ServicePostcards,
    Liveries,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mission {
    pub id: &'static str,
    pub title: &'static str,
    pub counter: MissionCounter,
    pub goal: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissionProgress {
    pub current: u64,
    pub goal: u64,
    pub complete: bool,
}

impl Mission {
    pub const KIND: super::catalog::ContentKind = super::catalog::ContentKind::Fictional;

    /// Validated durable state is the only input; evaluating cannot grant items.
    pub fn progress(
        self,
        journey: JourneySnapshot,
        service: &super::ServiceState,
    ) -> MissionProgress {
        let value = match self.counter {
            MissionCounter::Passengers => service.passengers_delivered,
            MissionCounter::Cargo => service.cargo_delivered,
            MissionCounter::Contracts => service.contracts_completed,
            MissionCounter::Stations => {
                journey.discovered_stations.iter().filter(|&&v| v).count() as u64
            }
            MissionCounter::Branches => service.branch.discoveries.len() as u64,
            MissionCounter::ServicePostcards => service.collection.len() as u64,
            MissionCounter::Liveries => service.owned_rewards.len() as u64,
        };
        MissionProgress {
            current: value.min(self.goal),
            goal: self.goal,
            complete: value >= self.goal,
        }
    }
}

pub const MISSIONS: [Mission; 11] = [
    Mission {
        id: "ce.passengers-20",
        title: "Companhia de viagem",
        counter: MissionCounter::Passengers,
        goal: 20,
    },
    Mission {
        id: "ce.passengers-100",
        title: "Plataformas acolhedoras",
        counter: MissionCounter::Passengers,
        goal: 100,
    },
    Mission {
        id: "ce.passengers-300",
        title: "Encontros sobre trilhos",
        counter: MissionCounter::Passengers,
        goal: 300,
    },
    Mission {
        id: "ce.cargo-20",
        title: "Correio sobre trilhos",
        counter: MissionCounter::Cargo,
        goal: 20,
    },
    Mission {
        id: "ce.cargo-100",
        title: "Abastecendo o circuito",
        counter: MissionCounter::Cargo,
        goal: 100,
    },
    Mission {
        id: "ce.contracts-10",
        title: "Dez entregas bem cuidadas",
        counter: MissionCounter::Contracts,
        goal: 10,
    },
    Mission {
        id: "ce.stations-4",
        title: "Quatro lugares no caderno",
        counter: MissionCounter::Stations,
        goal: 4,
    },
    Mission {
        id: "ce.branches-1",
        title: "Além do circuito",
        counter: MissionCounter::Branches,
        goal: 1,
    },
    Mission {
        id: "ce.branches-2",
        title: "Entre a serra e o mar",
        counter: MissionCounter::Branches,
        goal: 2,
    },
    Mission {
        id: "ce.postcards-2",
        title: "Álbum de serviços",
        counter: MissionCounter::ServicePostcards,
        goal: 2,
    },
    Mission {
        id: "ce.liveries-4",
        title: "Quatro cores de viagem",
        counter: MissionCounter::Liveries,
        goal: 4,
    },
];
