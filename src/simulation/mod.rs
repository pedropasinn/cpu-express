//! Deterministic, platform-independent simulation. No OS or renderer imports.

pub mod branch;
pub use branch::{BranchExpedition, BranchKind, BranchPhase, BranchState, BranchStop};
pub mod journey;
pub mod objectives;
pub mod route;
pub mod service;
pub use service::{
    ArrivalSummary, CargoContract, CargoKind, ContractStatus, PassengerGroup, ServiceReward,
    ServiceState, ServiceWagon,
};
pub mod timing;
pub mod train;
pub mod world;

pub use journey::{
    Achievement, Achievements, Journey, JourneySnapshot, LONG_JOURNEY_DISTANCE_UNITS,
    MAX_TRAVEL_EVENTS, TravelEvent,
};
pub use route::{
    Biome, DEFAULT_JOURNEY_SEED, Route, RouteSegment, STATION_COUNT, Station, StationId,
};
pub use timing::{
    AdvanceOutcome, MAX_ACTIVE_DELTA_SECONDS, MAX_TRAVEL_UNITS_PER_SECOND,
    TRAVEL_UNITS_PER_SCENE_PIXEL,
};
pub use train::{PIXELS_PER_SECOND_PER_KMH, Train, WHEEL_RADIUS_PX};
pub use world::{MAX_SMOKE_PARTICLES, SCROLL_PERIOD_PX, SmokeParticle, World};

fn valid_delta(seconds: f64) -> f64 {
    timing::active_delta(seconds).unwrap_or(0.0)
}
