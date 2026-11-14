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
