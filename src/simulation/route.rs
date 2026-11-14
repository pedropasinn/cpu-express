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
const STATION_KEYS: [&str; STATION_COUNT] = ["aurora", "pines", "ridge", "harbor"];

/// Authored content of the four-station circuit, read from the catalogue.
struct CircuitDefinition {
    line_name: &'static str,
    names: [&'static str; STATION_COUNT],
    base_units: [u32; STATION_COUNT],
    variation_units: [u32; STATION_COUNT],
    biomes: [Biome; STATION_COUNT],
}

/// Used only if the embedded catalogue cannot be read; a test keeps both identical.
const FALLBACK: CircuitDefinition = CircuitDefinition {
    line_name: "Circuito Aurora",
    names: [
        "Vila Aurora",
        "Bosque dos Pinheiros",
        "Serra Clara",
        "Porto do Sol",
    ],
    base_units: [1_800, 2_400, 2_100, 2_700],
    variation_units: [100; STATION_COUNT],
    biomes: [Biome::Meadow, Biome::Forest, Biome::Highland, Biome::Coast],
};

/// Identifier of the line this circuit engine runs.
pub const CIRCUIT_LINE_ID: &str = "aurora-v1";

fn from_catalogue() -> Option<CircuitDefinition> {
    use super::catalog::{Catalog, Layout};
    let line = Catalog::builtin().as_ref().ok()?.line(CIRCUIT_LINE_ID)?;
    if line.layout != Layout::Circuit
        || line.stations.len() != STATION_COUNT
        || line.segments.len() != STATION_COUNT
    {
        return None;
    }
    let mut definition = CircuitDefinition {
        line_name: line.name.as_str(),
        ..FALLBACK
    };
    for index in 0..STATION_COUNT {
        let station = &line.stations[index];
        let segment = &line.segments[index];
        if station.id != STATION_KEYS[index] {
            return None;
        }
        let base = segment.base_units?;
        let variation = segment.variation_units.unwrap_or(0);
        // The seed moves a length in eight equal steps around its base.
        if variation % 4 != 0 || variation >= base {
            return None;
        }
        definition.names[index] = station.name.as_str();
        definition.base_units[index] = base;
        definition.variation_units[index] = variation;
        definition.biomes[index] = match segment.biome.as_str() {
            "meadow" => Biome::Meadow,
            "forest" => Biome::Forest,
            "highland" => Biome::Highland,
            "coast" => Biome::Coast,
            _ => return None,
        };
    }
    Some(definition)
}

fn definition() -> &'static CircuitDefinition {
    static DEFINITION: std::sync::OnceLock<CircuitDefinition> = std::sync::OnceLock::new();
    DEFINITION.get_or_init(|| from_catalogue().unwrap_or(FALLBACK))
}

/// True when the running circuit comes from the catalogue and not from the fallback.
#[must_use]
pub fn circuit_from_catalogue() -> bool {
    from_catalogue().is_some()
}

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

    /// Display name from the catalogue; never a persistence key.
    #[must_use]
    pub fn name(self) -> &'static str {
        definition().names[self.index()]
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
        let content = definition();
        let mut random_state = seed;
        let mut lengths = [0.0; STATION_COUNT];
        for (index, length) in lengths.iter_mut().enumerate() {
            let variation_step = (splitmix64(&mut random_state) % 9) as u32;
            let variation = content.variation_units[index];
            *length =
                f64::from(content.base_units[index] - variation + variation_step * (variation / 4));
        }
        let segments = std::array::from_fn(|index| RouteSegment {
            index,
            from: STATION_IDS[index],
            to: STATION_IDS[(index + 1) % STATION_COUNT],
            length_units: lengths[index],
            biome_hint: content.biomes[index],
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
        definition().line_name
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_runs_on_catalogue_content_and_the_fallback_is_identical() {
        assert!(circuit_from_catalogue());
        let content = from_catalogue().unwrap();
        assert_eq!(content.line_name, FALLBACK.line_name);
        assert_eq!(content.names, FALLBACK.names);
        assert_eq!(content.base_units, FALLBACK.base_units);
        assert_eq!(content.variation_units, FALLBACK.variation_units);
        assert_eq!(content.biomes, FALLBACK.biomes);
        for (index, id) in STATION_IDS.iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(id.name(), content.names[index]);
        }
    }

    #[test]
    fn seeded_lengths_keep_the_rc7_values() {
        // Lengths recorded from the default seed before the catalogue existed.
        let route = Route::default();
        let lengths: Vec<f64> = route.segments().iter().map(|s| s.length_units).collect();
        assert_eq!(lengths, [1900.0, 2375.0, 2025.0, 2725.0]);
        assert_eq!(route.length_units(), 9025.0);
    }
}
