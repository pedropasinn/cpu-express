//! Original, deterministic pixel art on a 320 x 70 logical canvas.
//!
//! This module only reads simulation state. It never samples the computer,
//! sleeps, moves a native window, or changes the train's speed. The app's
//! nearest-neighbor texture upload and the baseline interface are unchanged.

mod engine_details;
mod engines;
mod landmarks;
mod legacy;
mod palette;
mod rolling_stock;
mod scene;
mod scenery;
mod segment_scenery;
mod sprites;
mod stations;
mod stock_details;
mod theme;

pub use engines::EngineStyle;
pub use palette::Palette;
pub use rolling_stock::CoachStyle;
pub use scene::{
    DISPLAY_HEIGHT, DISPLAY_WIDTH, FRAMEBUFFER_BYTES, HEIGHT, MAX_WIDTH, MIN_WIDTH, SceneRenderer,
    WIDTH,
};
pub use segment_scenery::{SceneryContext, SegmentScenery};
pub use stations::StationService;
pub use theme::{
    BIOME_LENGTH_PX, Biome, DAY_CYCLE_SECONDS, ROUTE_PERIOD_PX, RenderQuality, SceneContext,
    SceneOptions, SceneTheme, TRANSITION_LENGTH_PX, Weather, route_at,
};
