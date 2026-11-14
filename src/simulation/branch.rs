//! Bounded side trips anchored to Vila Aurora; never reinterpret the saved circuit.
use super::{Biome, JourneySnapshot, StationId};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchKind {
    Serra,
    Litoral,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchStop {
    MiranteDaSerra,
    CaisDasConchas,
}
impl BranchStop {
    pub const fn name(self) -> &'static str {
        match self {
            Self::MiranteDaSerra => "Mirante da Serra",
            Self::CaisDasConchas => "Cais das Conchas",
        }
    }
}
impl BranchKind {
    pub const fn stop(self) -> BranchStop {
        match self {
            Self::Serra => BranchStop::MiranteDaSerra,
            Self::Litoral => BranchStop::CaisDasConchas,
        }
    }
    pub const fn biome(self) -> Biome {
        match self {
            Self::Serra => Biome::Highland,
            Self::Litoral => Biome::Coast,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchPhase {
    Outbound,
    Service,
    Return,
}
impl BranchPhase {
    pub const fn duration(self) -> f64 {
        match self {
            Self::Service => 8.0,
            _ => 90.0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchExpedition {
    pub kind: BranchKind,
    pub phase: BranchPhase,
    pub phase_seconds: f64,
}
impl BranchExpedition {
    pub fn progress(self) -> f64 {
        (self.phase_seconds / self.phase.duration()).clamp(0.0, 1.0)
    }
    pub fn active_seconds(self) -> f64 {
        self.phase_seconds
            + match self.phase {
                BranchPhase::Outbound => 0.0,
                BranchPhase::Service => 90.0,
                BranchPhase::Return => 98.0,
            }
    }
    pub fn remaining_seconds(self) -> f64 {
        188.0 - self.active_seconds()
    }
    pub fn station_name(self) -> &'static str {
        if self.phase == BranchPhase::Return {
            "Vila Aurora"
        } else {
            self.kind.stop().name()
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchState {
    pub pending: Option<BranchKind>,
    pub current: Option<BranchExpedition>,
    pub discoveries: Vec<BranchStop>,
}
impl BranchState {
    pub fn unlocked(snapshot: JourneySnapshot) -> bool {
        snapshot.completed_laps >= 1
    }
    pub fn request(&mut self, kind: BranchKind, snapshot: JourneySnapshot) -> Result<(), String> {
        if !Self::unlocked(snapshot) {
            return Err("Complete o primeiro circuito para liberar excursões.".into());
        }
        if self.current.is_some() || self.pending.is_some() {
            return Err("Já existe uma excursão em andamento ou agendada.".into());
        }
        self.pending = Some(kind);
        Ok(())
    }
    pub(crate) fn try_depart(&mut self, snapshot: JourneySnapshot, station_wait: f64) {
        if self.current.is_none()
            && station_wait == 0.0
            && snapshot.at_station == Some(StationId::Aurora)
            && let Some(kind) = self.pending.take()
        {
            self.current = Some(BranchExpedition {
                kind,
                phase: BranchPhase::Outbound,
                phase_seconds: 0.0,
            });
        }
    }
    /// Caller splits at phase boundaries. Reaching destination records a set membership,
    /// never currency or passenger/cargo transactions; restart cannot replay a reward.
    pub(crate) fn advance(&mut self, seconds: f64) {
        let Some(current) = &mut self.current else {
            return;
        };
        current.phase_seconds += seconds;
        if current.phase_seconds + 1e-9 >= current.phase.duration() {
            current.phase_seconds = 0.0;
            match current.phase {
                BranchPhase::Outbound => {
                    let stop = current.kind.stop();
                    if !self.discoveries.contains(&stop) {
                        self.discoveries.push(stop);
                    }
                    current.phase = BranchPhase::Service;
                }
                BranchPhase::Service => current.phase = BranchPhase::Return,
                BranchPhase::Return => self.current = None,
            }
        }
    }
    pub fn validate(&self, snapshot: JourneySnapshot) -> Result<(), String> {
        let invalid = || "Excursão inválida; arquivo preservado.".to_owned();
        if self.discoveries.len() > 2
            || self
                .discoveries
                .iter()
                .enumerate()
                .any(|(i, d)| self.discoveries[..i].contains(d))
            || (self.pending.is_some() && self.current.is_some())
        {
            return Err(invalid());
        }
        if !Self::unlocked(snapshot)
            && (self.pending.is_some() || self.current.is_some() || !self.discoveries.is_empty())
        {
            return Err(invalid());
        }
        if let Some(c) = self.current
            && (snapshot.at_station != Some(StationId::Aurora)
                || !c.phase_seconds.is_finite()
                || c.phase_seconds < 0.0
                || c.phase_seconds >= c.phase.duration()
                || c.active_seconds() > snapshot.active_seconds
                || (c.phase != BranchPhase::Outbound && !self.discoveries.contains(&c.kind.stop())))
        {
            return Err(invalid());
        }
        Ok(())
    }
}
