//! Pure, stack-sized scenery hints. No catalog access, I/O or saved state.
use crate::simulation::catalog::SceneryTag;

/// A set of segment tags; order and duplicates never affect the image.
/// Empty means the original Aurora landscape, including its exact pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SegmentScenery(u16);

impl SegmentScenery {
    pub const EMPTY: Self = Self(0);

    #[must_use]
    pub fn from_tags(tags: &[SceneryTag]) -> Self {
        Self(tags.iter().fold(0, |bits, tag| bits | bit(*tag)))
    }

    #[must_use]
    pub const fn contains(self, tag: SceneryTag) -> bool {
        self.0 & bit(tag) != 0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Uses the same blend weight as SceneContext, even for equal adjacent biomes.
/// The route adapter supplies both sets; the renderer never looks up a segment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneryContext {
    pub current: SegmentScenery,
    pub next: SegmentScenery,
}

impl SceneryContext {
    #[must_use]
    pub const fn new(scenery: SegmentScenery) -> Self {
        Self {
            current: scenery,
            next: scenery,
        }
    }
}

const fn bit(tag: SceneryTag) -> u16 {
    1 << match tag {
        SceneryTag::Plain => 0,
        SceneryTag::Farmland => 1,
        SceneryTag::Forest => 2,
        SceneryTag::Mountain => 3,
        SceneryTag::River => 4,
        SceneryTag::Bridge => 5,
        SceneryTag::Viaduct => 6,
        SceneryTag::Tunnel => 7,
        SceneryTag::Urban => 8,
        SceneryTag::Port => 9,
        SceneryTag::Coast => 10,
        SceneryTag::Plateau => 11,
    }
}
