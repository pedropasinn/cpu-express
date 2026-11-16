//! Where a train is on a catalogue line, for any number of stations.
//!
//! Pure arithmetic over the total distance travelled: no clock, no state and no
//! allocation after construction. A circuit repeats its stations in order; a
//! shuttle runs end to end and comes back over the same segments, so a save only
//! needs the distance to know the direction of travel.
use super::catalog::{Layout, LineDef, LineStatus};

/// Journey units per real kilometre on real lines. At the timetable speed of 24
/// units per active second, one kilometre takes twenty seconds: segments of the
/// first real line last one to seven minutes. Confirmed by the owner on 2026-11-14.
pub const DEFAULT_UNITS_PER_KM: f64 = 480.0;

/// Same tolerance the circuit engine uses to decide that a train is at a platform.
const AT_STATION_TOLERANCE: f64 = 1e-6;

#[derive(Clone, Debug, PartialEq)]
pub struct Itinerary {
    line_id: String,
    layout: Layout,
    /// Length of each segment in journey units, in catalogue order.
    lengths: Vec<f64>,
    /// Distance of every station from the first one, in journey units.
    offsets: Vec<f64>,
}

/// One instant of a journey. Station and segment values index the line's lists.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// Stations reached since the start; the origin is not counted.
    pub stations_visited: u64,
    /// Complete circuits, or complete one-way runs of a shuttle.
    pub runs_completed: u64,
    pub segment: usize,
    /// Station the train left.
    pub from: usize,
    /// Station ahead.
    pub to: usize,
    /// False while a shuttle returns toward its first station.
    pub forward: bool,
    pub distance_into_segment: f64,
    pub distance_to_next: f64,
    /// Set while the train stands exactly at a station.
    pub at_station: Option<usize>,
}

impl Itinerary {
    /// Builds the itinerary of a validated line. Fictional lines use their base
    /// lengths; real lines convert kilometres with `units_per_km`.
    pub fn from_line(line: &LineDef, units_per_km: f64) -> Result<Self, String> {
        if !units_per_km.is_finite() || !(1.0..=100_000.0).contains(&units_per_km) {
            return Err("Escala de distância inválida.".into());
        }
        let mut lengths = Vec::with_capacity(line.segments.len());
        for segment in &line.segments {
            let length = match (segment.base_units, segment.distance_km) {
                (Some(units), _) => f64::from(units),
                (None, Some(km)) => km * units_per_km,
                (None, None) => {
                    return Err(format!(
                        "linha `{}` › trecho `{}`: sem comprimento definido",
                        line.id, segment.id
                    ));
                }
            };
            if !length.is_finite() || length <= 0.0 {
                return Err(format!(
                    "linha `{}` › trecho `{}`: comprimento inválido",
                    line.id, segment.id
                ));
            }
            lengths.push(length);
        }
        let stations = line.stations.iter().filter(|s| !s.retired).count();
        let expected = match line.layout {
            Layout::Circuit => stations,
            Layout::Shuttle => stations.saturating_sub(1),
        };
        if stations < 2 || lengths.len() != expected {
            return Err(format!(
                "linha `{}`: estações e trechos não formam um percurso",
                line.id
            ));
        }
        Ok(Self::new(line.id.clone(), line.layout, lengths))
    }

    /// Itinerary from explicit segment lengths, as produced by a seeded circuit.
    pub fn from_lengths(line_id: &str, layout: Layout, lengths: Vec<f64>) -> Result<Self, String> {
        let minimum = match layout {
            Layout::Circuit => 2,
            Layout::Shuttle => 1,
        };
        if lengths.len() < minimum || lengths.iter().any(|l| !l.is_finite() || *l <= 0.0) {
            return Err("Comprimentos de trecho inválidos.".into());
        }
        Ok(Self::new(line_id.to_owned(), layout, lengths))
    }

    fn new(line_id: String, layout: Layout, lengths: Vec<f64>) -> Self {
        let mut offsets = Vec::with_capacity(lengths.len() + 1);
        let mut position = 0.0;
        offsets.push(0.0);
        for length in &lengths {
            position += length;
            offsets.push(position);
        }
        if layout == Layout::Circuit {
            // The last offset is the origin again.
            offsets.pop();
        }
        Self {
            line_id,
            layout,
            lengths,
            offsets,
        }
    }

    #[must_use]
    pub fn line_id(&self) -> &str {
        &self.line_id
    }

    #[must_use]
    pub fn layout(&self) -> Layout {
        self.layout
    }

    #[must_use]
    pub fn station_count(&self) -> usize {
        self.offsets.len()
    }

    /// Length of one circuit, or of one end-to-end run of a shuttle.
    #[must_use]
    pub fn run_length(&self) -> f64 {
        self.lengths.iter().sum()
    }

    #[must_use]
    pub fn segment_length(&self, segment: usize) -> Option<f64> {
        self.lengths.get(segment).copied()
    }

    /// Distance of a station from the first one along the line.
    #[must_use]
    pub fn station_offset(&self, station: usize) -> Option<f64> {
        self.offsets.get(station).copied()
    }

    /// Stations served per run: every station of a circuit, all but the origin of a shuttle.
    fn legs_per_run(&self) -> u64 {
        self.lengths.len() as u64
    }

    /// Station reached by the `visit`-th arrival; visit zero is the origin.
    #[must_use]
    pub fn station_at_visit(&self, visit: u64) -> usize {
        let legs = self.legs_per_run();
        match self.layout {
            Layout::Circuit => (visit % legs) as usize,
            Layout::Shuttle => {
                let step = visit % (2 * legs);
                if step <= legs {
                    step as usize
                } else {
                    (2 * legs - step) as usize
                }
            }
        }
    }

    /// Absolute boundary for an arrival, independent of the last frame's rounding.
    #[must_use]
    pub fn distance_at_visit(&self, visit: u64) -> f64 {
        let legs = self.legs_per_run();
        let runs = visit / legs;
        let leg = (visit % legs) as usize;
        let run = self.run_length();
        let offset = if self.layout == Layout::Circuit || runs.is_multiple_of(2) {
            self.offsets[leg]
        } else {
            run - self.offsets[self.offsets.len() - 1 - leg]
        };
        runs as f64 * run + offset
    }

    /// Locates a train from the distance travelled since the origin. Invalid input
    /// (negative, NaN, infinite) is treated as the origin.
    #[must_use]
    pub fn locate(&self, total_distance: f64) -> Position {
        let total = if total_distance.is_finite() {
            total_distance.max(0.0)
        } else {
            0.0
        };
        let run = self.run_length();
        let legs = self.legs_per_run();
        let runs_completed = ((total + AT_STATION_TOLERANCE) / run).floor() as u64;
        let mut remainder = (total - runs_completed as f64 * run).max(0.0);
        let forward = self.layout == Layout::Circuit || runs_completed.is_multiple_of(2);
        // Walk the segments in travel order for this run.
        let mut leg = 0_usize;
        let segment_at = |leg: usize| {
            if forward {
                leg
            } else {
                self.lengths.len() - 1 - leg
            }
        };
        while leg + 1 < self.lengths.len()
            && remainder + AT_STATION_TOLERANCE >= self.lengths[segment_at(leg)]
        {
            remainder = (remainder - self.lengths[segment_at(leg)]).max(0.0);
            leg += 1;
        }
        let segment = segment_at(leg);
        let length = self.lengths[segment];
        let stations_visited = runs_completed * legs + leg as u64;
        let from = self.station_at_visit(stations_visited);
        let to = self.station_at_visit(stations_visited + 1);
        let at_station = (remainder < AT_STATION_TOLERANCE).then_some(from);
        Position {
            stations_visited,
            runs_completed,
            segment,
            from,
            to,
            forward,
            distance_into_segment: remainder,
            distance_to_next: (length - remainder).max(0.0),
            at_station,
        }
    }
}

/// The itinerary of a line the player may run. Drafts are refused.
pub fn playable(line: &LineDef, units_per_km: f64) -> Result<Itinerary, String> {
    if line.status != LineStatus::Playable {
        return Err(format!("linha `{}` ainda é rascunho", line.id));
    }
    Itinerary::from_line(line, units_per_km)
}
