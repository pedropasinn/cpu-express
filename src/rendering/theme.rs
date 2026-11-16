//! Small, optional presentation controls. No configuration or simulation ownership.

/// A visual environment; it has no effect on CPU sampling or the train's speed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Biome {
    #[default]
    GreenValley,
    Forest,
    Coast,
    HighDesert,
}

/// The original frame remains available as a pixel-exact rollback path.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneTheme {
    #[default]
    Journey,
    Legacy,
}

/// Low removes decorative passes, while retaining travel, light and all wheels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderQuality {
    #[default]
    Full,
    Low,
}

/// Weather is opt-in: a deterministic, bounded number of crisp rain strokes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weather {
    #[default]
    Clear,
    Cloudy,
    Fog,
    Rain,
    Thunderstorm,
    Snow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SceneOptions {
    pub day_cycle: bool,
    pub clouds: bool,
    pub weather: Weather,
    pub details: bool,
}

impl Default for SceneOptions {
    fn default() -> Self {
        Self {
            day_cycle: true,
            clouds: true,
            weather: Weather::Clear,
            details: true,
        }
    }
}

/// Optional adapter for a future journey/domain owner. Blend is normalized by
/// the setter; invalid values become zero and finite values are clamped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneContext {
    pub biome: Biome,
    pub next_biome: Biome,
    pub blend: f32,
}

impl SceneContext {
    pub fn from_world(world: &crate::simulation::World) -> Self {
        let view = world.travel_view();
        let t = ((view.segment_progress - 0.75) / 0.25).clamp(0.0, 1.0) as f32;
        Self {
            biome: view.biome.into(),
            next_biome: view.next_biome.into(),
            blend: t * t * (3.0 - 2.0 * t),
        }
    }
    #[must_use]
    pub const fn new(biome: Biome) -> Self {
        Self {
            biome,
            next_biome: biome,
            blend: 0.0,
        }
    }

    /// Follow the real journey's outgoing segment and approach its next
    /// environment over the final quarter of the leg. All drawing offsets
    /// remain anchored to World; arriving never restarts parallax or weather.
    ///
    /// Serra Clara's Highland content uses the existing authored HighDesert
    /// landscape: an arid highland. The procedural three-biome demo route is
    /// still available through route_at when no journey context is supplied.
    #[must_use]
    pub fn from_journey(journey: &crate::simulation::JourneySnapshot) -> Self {
        let progress = finite_nonnegative(journey.segment_progress).clamp(0.0, 1.0);
        let t = ((progress - 0.75) / 0.25).clamp(0.0, 1.0) as f32;
        Self {
            biome: Biome::from(journey.route_segment.biome_hint),
            next_biome: Biome::from(journey.next_biome_hint),
            blend: t * t * (3.0 - 2.0 * t),
        }
    }

    pub(super) fn normalized(self) -> Self {
        Self {
            blend: if self.blend.is_finite() {
                self.blend.clamp(0.0, 1.0)
            } else {
                0.0
            },
            ..self
        }
    }
}

impl From<crate::simulation::Biome> for Biome {
    fn from(biome: crate::simulation::Biome) -> Self {
        match biome {
            crate::simulation::Biome::Meadow => Self::GreenValley,
            crate::simulation::Biome::Forest => Self::Forest,
            crate::simulation::Biome::Highland => Self::HighDesert,
            crate::simulation::Biome::Coast => Self::Coast,
        }
    }
}

pub const BIOME_LENGTH_PX: f64 = 2_048.0;
pub const TRANSITION_LENGTH_PX: f64 = 512.0;
/// Exactly 30 route cycles fit in the baseline World's scroll wrapping period.
pub const ROUTE_PERIOD_PX: f64 = BIOME_LENGTH_PX * 3.0;
pub const DAY_CYCLE_SECONDS: f64 = 600.0;

/// A pure lookup: no accumulated renderer state, frame count or wall clock.
/// Every layer keeps its own global scroll during the color cross-fade.
#[must_use]
pub fn route_at(distance_px: f64) -> SceneContext {
    let distance = finite_nonnegative(distance_px).rem_euclid(ROUTE_PERIOD_PX);
    let section = (distance / BIOME_LENGTH_PX).floor() as usize;
    let offset = distance.rem_euclid(BIOME_LENGTH_PX);
    let t = ((offset - (BIOME_LENGTH_PX - TRANSITION_LENGTH_PX)) / TRANSITION_LENGTH_PX)
        .clamp(0.0, 1.0) as f32;
    let environments = [Biome::GreenValley, Biome::Coast, Biome::HighDesert];
    SceneContext {
        biome: environments[section],
        next_biome: environments[(section + 1) % environments.len()],
        blend: t * t * (3.0 - 2.0 * t),
    }
}

pub(super) fn finite_nonnegative(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
