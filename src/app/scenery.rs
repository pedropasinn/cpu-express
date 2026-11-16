//! Route-to-renderer adapter. Unknown routes and excursions clear old hints.
use super::*;
use crate::rendering::{
    PassengerOptions, SceneryContext, SceneryLifeOptions, SegmentScenery, StationStyle,
    StationStyles,
};

fn route_art(line: &str, segment: usize, excursion: bool) -> (SceneryContext, StationStyles) {
    if excursion {
        return Default::default();
    }
    let Ok(catalog) = crate::simulation::catalog::Catalog::builtin() else {
        return Default::default();
    };
    let Some(line) = catalog.line(line) else {
        return Default::default();
    };
    let Some(current) = line.segments.get(segment) else {
        return Default::default();
    };
    let next = &line.segments[(segment + 1) % line.segments.len()];
    // Authored Aurora mapping until E3 adds architecture to generic stations.
    let style = |id: &str| match (line.id.as_str(), id) {
        ("aurora-v1", "aurora") => Some(StationStyle::Plateau),
        ("aurora-v1", "pines") => Some(StationStyle::TimberHalt),
        ("aurora-v1", "ridge") => Some(StationStyle::MountainStone),
        ("aurora-v1", "harbor") => Some(StationStyle::PortTerminal),
        _ => None,
    };
    (
        SceneryContext {
            current: SegmentScenery::from_tags(&current.scenery),
            next: SegmentScenery::from_tags(&next.scenery),
        },
        StationStyles {
            departing: style(&current.from),
            arriving: style(&current.to),
        },
    )
}

impl CpuExpressApp {
    pub(super) fn update_scenery_adapter(&mut self) {
        let (scenery, styles) = route_art(
            crate::simulation::route::CIRCUIT_LINE_ID,
            self.world.journey_snapshot().route_segment.index,
            self.world.excursion_snapshot().current.is_some(),
        );
        self.renderer.set_scenery_context(scenery);
        let snapshot = self.world.journey_snapshot();
        // Aurora's inhabited endpoints; E3 must supply the active itinerary's
        // endpoints (including reverse travel) when it enables other lines.
        let (departure, arrival) = match snapshot.route_segment.index {
            0 => (true, false),
            2 => (false, true),
            3 => (true, true),
            _ => (false, false),
        };
        self.renderer
            .set_settlement_context(crate::rendering::SettlementContext::along_segment(
                snapshot.seed,
                snapshot.segment_progress,
                departure,
                arrival,
            ));
        self.renderer.set_station_styles(styles);
        self.renderer.set_scenery_life_options(SceneryLifeOptions {
            reduce_motion: self.config.reduce_menu_motion || self.machine_effects.low_power,
        });
        self.renderer.set_passenger_options(PassengerOptions {
            reduce_motion: self.config.reduce_menu_motion || self.machine_effects.low_power,
            cold: self.local_weather.last_good.as_ref().is_some_and(|s| {
                !s.stale() && s.temperature_c.is_finite() && s.temperature_c < 12.0
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_context_tracks_segments_and_clears_for_excursions() {
        let (first, styles) = route_art("aurora-v1", 0, false);
        assert!(!first.current.is_empty());
        assert_eq!(styles.departing, Some(StationStyle::Plateau));
        assert_eq!(styles.arriving, Some(StationStyle::TimberHalt));
        let (last, styles) = route_art("aurora-v1", 3, false);
        assert_eq!(last.next, first.current);
        assert_eq!(styles.arriving, Some(StationStyle::Plateau));
        for (line, index, excursion) in [
            ("missing", 0, false),
            ("aurora-v1", 50, false),
            ("aurora-v1", 0, true),
        ] {
            assert_eq!(route_art(line, index, excursion), Default::default());
        }
    }
}
