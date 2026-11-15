//! Cached, UI-independent map geometry. Construction allocates; queries do not.
//! Station ordinals follow the itinerary's active list, not raw catalogue indices.
use super::{
    catalog::{Layout, LineDef},
    itinerary::Itinerary,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapPoint {
    pub x: f64,
    pub y: f64,
}

impl MapPoint {
    fn between(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MapStation {
    pub ordinal: usize,
    pub catalog_index: usize,
    pub point: MapPoint,
    /// Distance-scaled x and altitude-scaled y; None means unknown elevation.
    pub profile: Option<MapPoint>,
    pub elevation_m: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapTrain {
    pub point: MapPoint,
    pub profile: Option<MapPoint>,
    pub from: usize,
    pub to: usize,
    pub forward: bool,
    pub at_station: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct MapLayout {
    itinerary: Itinerary,
    stations: Vec<MapStation>,
    elevation_range: Option<(f64, f64)>,
    rows: usize,
}

impl MapLayout {
    /// Use the actual seeded itinerary for Aurora, not catalogue base lengths.
    /// Draft lines may be previewed, but this never makes them playable.
    pub fn new(line: &LineDef, itinerary: &Itinerary) -> Result<Self, String> {
        let active: Vec<_> = line
            .stations
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.retired)
            .collect();
        if line.id != itinerary.line_id()
            || line.layout != itinerary.layout()
            || active.len() != itinerary.station_count()
            || line.segments.len()
                != match line.layout {
                    Layout::Circuit => active.len(),
                    Layout::Shuttle => active.len().saturating_sub(1),
                }
            || !itinerary.run_length().is_finite()
        {
            return Err("Linha e itinerário não correspondem.".into());
        }
        for (i, segment) in line.segments.iter().enumerate() {
            if segment.from != active[i].1.id || segment.to != active[(i + 1) % active.len()].1.id {
                return Err("Ordem das estações incompatível com o itinerário.".into());
            }
        }
        let elevation_range = active
            .iter()
            .filter_map(|(_, s)| s.elevation_m.filter(|h| h.is_finite()))
            .fold(None, |range, h| {
                Some(match range {
                    None => (h, h),
                    Some((lo, hi)) => (f64::min(lo, h), f64::max(hi, h)),
                })
            });
        let columns = active.len().min(7);
        let rows = active.len().div_ceil(columns);
        let stations = active
            .iter()
            .enumerate()
            .map(|(ordinal, (catalog_index, station))| {
                let point = match line.layout {
                    Layout::Circuit => {
                        let angle = ordinal as f64 * std::f64::consts::TAU / active.len() as f64
                            - std::f64::consts::FRAC_PI_2;
                        MapPoint {
                            x: 0.5 + 0.45 * angle.cos(),
                            y: 0.5 + 0.45 * angle.sin(),
                        }
                    }
                    Layout::Shuttle => {
                        let row = ordinal / columns;
                        let col = if row.is_multiple_of(2) {
                            ordinal % columns
                        } else {
                            columns - 1 - ordinal % columns
                        };
                        MapPoint {
                            x: col as f64 / (columns - 1) as f64,
                            y: if rows == 1 {
                                0.5
                            } else {
                                row as f64 / (rows - 1) as f64
                            },
                        }
                    }
                };
                let elevation_m = station.elevation_m.filter(|h| h.is_finite());
                let profile = elevation_m
                    .zip(elevation_range)
                    .map(|(h, (lo, hi))| MapPoint {
                        x: itinerary.station_offset(ordinal).unwrap() / itinerary.run_length(),
                        y: if hi == lo {
                            0.5
                        } else {
                            1.0 - (h - lo) / (hi - lo)
                        },
                    });
                MapStation {
                    ordinal,
                    catalog_index: *catalog_index,
                    point,
                    profile,
                    elevation_m,
                }
            })
            .collect();
        Ok(Self {
            itinerary: itinerary.clone(),
            stations,
            elevation_range,
            rows,
        })
    }

    pub fn stations(&self) -> &[MapStation] {
        &self.stations
    }
    pub fn elevation_range(&self) -> Option<(f64, f64)> {
        self.elevation_range
    }
    /// Suggested minimum logical height, before margins and labels.
    pub fn schematic_height(&self) -> f64 {
        match self.itinerary.layout() {
            Layout::Circuit => 160.0,
            Layout::Shuttle => (self.rows.saturating_sub(1) as f64 * 32.0).max(48.0),
        }
    }

    /// Discovery since this itinerary's origin; excursion state is separate.
    pub fn discovered(&self, ordinal: usize, total_distance: f64) -> bool {
        ordinal < self.stations.len()
            && ordinal as u64 <= self.itinerary.locate(total_distance).stations_visited
    }

    pub fn train(&self, total_distance: f64) -> MapTrain {
        let position = self.itinerary.locate(total_distance);
        let t = (position.distance_into_segment
            / self.itinerary.segment_length(position.segment).unwrap())
        .clamp(0.0, 1.0);
        let from = &self.stations[position.from];
        let to = &self.stations[position.to];
        // A circuit's closing profile extends to x=1 instead of drawing a false
        // backwards slope across all preceding stations. It wraps at arrival.
        let profile = if position.at_station.is_some() {
            from.profile
        } else {
            from.profile.zip(to.profile).map(|(a, mut b)| {
                if self.itinerary.layout() == Layout::Circuit && position.to == 0 {
                    b.x = 1.0;
                }
                a.between(b, t)
            })
        };
        MapTrain {
            point: if position.at_station.is_some() {
                from.point
            } else {
                from.point.between(to.point, t)
            },
            profile,
            from: position.from,
            to: position.to,
            forward: position.forward,
            at_station: position.at_station,
        }
    }
}
