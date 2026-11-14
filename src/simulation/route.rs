//! The built-in circular route, independent of graphics, clocks and OS metrics.
//!
//! All distances below are fictional journey units. They are neither real
//! kilometres nor a measurement of work performed by the computer. A seed
//! changes the length of the four legs; station identity, order and biome hints
//! stay stable so discovery state can be associated with an explicit identifier.

/// Every built-in route has four stations and four connecting legs.
pub const STATION_COUNT: usize = 4;
/// Stable default; no clock, operating-system randomness or external I/O is used.
pub const DEFAULT_JOURNEY_SEED: u64 = 0x4350_5520_4558_5052;

const STATION_IDS: [StationId; STATION_COUNT] = [
    StationId::Aurora,
    StationId::Pines,
    StationId::Ridge,
    StationId::Harbor,
];
const BIOMES: [Biome; STATION_COUNT] =
    [Biome::Meadow, Biome::Forest, Biome::Highland, Biome::Coast];
const BASE_LENGTHS: [u32; STATION_COUNT] = [1_800, 2_400, 2_100, 2_700];

/// Stable station identity; display names are not persistence keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StationId {
    Aurora,
    Pines,
    Ridge,
    Harbor,
}

impl StationId {
    /// Index in the fixed circular station order.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Aurora => 0,
            Self::Pines => 1,
            Self::Ridge => 2,
            Self::Harbor => 3,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aurora => "Vila Aurora",
            Self::Pines => "Bosque dos Pinheiros",
            Self::Ridge => "Serra Clara",
            Self::Harbor => "Porto do Sol",
        }
    }
}

/// A content hint for the renderer; it deliberately contains no visual styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    Meadow,
    Forest,
    Highland,
    Coast,
}

/// A point along one circuit. The final leg returns to the station at zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    pub id: StationId,
    pub name: &'static str,
    pub distance_from_origin_units: f64,
}

/// One directed leg, including the leg from the last station back to the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteSegment {
    pub index: usize,
    pub from: StationId,
    pub to: StationId,
    pub length_units: f64,
    /// Hint for the outgoing leg, not a request to replace a renderer palette.
    pub biome_hint: Biome,
}

/// Immutable, bounded route data. Construction and cloning do not allocate.
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    seed: u64,
    stations: [Station; STATION_COUNT],
    segments: [RouteSegment; STATION_COUNT],
    length_units: f64,
}

impl Default for Route {
    fn default() -> Self {
        Self::from_seed(DEFAULT_JOURNEY_SEED)
    }
}

impl Route {
    /// Build a repeatable circuit with small, discrete variations in leg length.
    ///
    /// Lengths are multiples of 25, at most 100 units from their base length.
    /// The ranges do not overlap, so every circuit has four distinct leg lengths.
    /// All `u64` seeds, including zero, are valid. The explicit SplitMix64 step
    /// avoids dependence on a library's RNG algorithm or the host architecture.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        let mut random_state = seed;
        let lengths = BASE_LENGTHS.map(|base| {
            let variation_step = (splitmix64(&mut random_state) % 9) as u32;
            f64::from(base - 100 + variation_step * 25)
        });
        let segments = std::array::from_fn(|index| RouteSegment {
            index,
            from: STATION_IDS[index],
            to: STATION_IDS[(index + 1) % STATION_COUNT],
            length_units: lengths[index],
            biome_hint: BIOMES[index],
        });
        let mut position = 0.0;
        let stations = std::array::from_fn(|index| {
            let id = STATION_IDS[index];
            let station = Station {
                id,
                name: id.name(),
                distance_from_origin_units: position,
            };
            position += lengths[index];
            station
        });
        Self {
            seed,
            stations,
            segments,
            length_units: position,
        }
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        "Circuito Aurora"
    }

    #[must_use]
    pub fn stations(&self) -> &[Station; STATION_COUNT] {
        &self.stations
    }

    #[must_use]
    pub fn segments(&self) -> &[RouteSegment; STATION_COUNT] {
        &self.segments
    }

    /// Full length of the circular route in fictional journey units.
    #[must_use]
    pub fn length_units(&self) -> f64 {
        self.length_units
    }

    #[must_use]
    pub fn station(&self, id: StationId) -> &Station {
        &self.stations[id.index()]
    }

    #[must_use]
    pub fn segment(&self, index: usize) -> Option<&RouteSegment> {
        self.segments.get(index)
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
