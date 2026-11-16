//! Route-to-renderer adapter. Unknown routes and excursions clear old hints.
use super::*;
use crate::rendering::{
    PassengerOptions, SceneryContext, SceneryLifeOptions, SegmentScenery, StationStyle,
    StationStyles,
};

fn route_art(
    line: &str,
    segment: usize,
    forward: bool,
    excursion: bool,
) -> (SceneryContext, StationStyles) {
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
    use crate::simulation::catalog::Layout;
    let next_index = match (line.layout, forward) {
        (Layout::Circuit, true) => (segment + 1) % line.segments.len(),
        (Layout::Circuit, false) => return Default::default(),
        (Layout::Shuttle, true) => (segment + 1).min(line.segments.len() - 1),
        (Layout::Shuttle, false) => segment.saturating_sub(1),
    };
    let next = &line.segments[next_index];
    let style = |id: &str| {
        line.stations
            .iter()
            .find(|s| s.id == id && !s.retired)
            .and_then(|s| s.architecture.as_ref())
            .map(|a| station_style(a.style))
    };
    let (from, to) = if forward {
        (&current.from, &current.to)
    } else {
        (&current.to, &current.from)
    };
    (
        SceneryContext {
            current: SegmentScenery::from_tags(&current.scenery),
            next: SegmentScenery::from_tags(&next.scenery),
        },
        StationStyles {
            departing: style(from),
            arriving: style(to),
        },
    )
}

pub(super) fn station_style(style: crate::simulation::catalog::ArchitectureStyle) -> StationStyle {
    use crate::simulation::catalog::ArchitectureStyle as A;
    match style {
        A::Plateau => StationStyle::Plateau,
        A::TimberHalt => StationStyle::TimberHalt,
        A::MountainStone => StationStyle::MountainStone,
        A::PortTerminal => StationStyle::PortTerminal,
        A::UrbanTerminal => StationStyle::UrbanTerminal,
        A::ServicePost => StationStyle::ServicePost,
    }
}

impl CpuExpressApp {
    pub(super) fn update_scenery_adapter(&mut self) {
        let (scenery, styles) = route_art(
            self.world.active_line_id(),
            self.world.travel_view().segment,
            self.world.travel_view().forward,
            self.world.excursion_snapshot().current.is_some(),
        );
        self.renderer.set_scenery_context(scenery);
        let snapshot = self.world.travel_view();
        let inhabited = |style| {
            matches!(
                style,
                Some(
                    StationStyle::Plateau
                        | StationStyle::UrbanTerminal
                        | StationStyle::PortTerminal
                )
            )
        };
        let (departure, arrival) = (inhabited(styles.departing), inhabited(styles.arriving));
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
    fn real_line_reverses_platforms_and_selects_adjacent_scenery_at_terminals() {
        let catalog = crate::simulation::catalog::Catalog::builtin()
            .as_ref()
            .unwrap();
        let line = catalog.line("curitiba-paranagua").unwrap();
        for index in 0..line.segments.len() {
            let (outbound, styles) = route_art(&line.id, index, true, false);
            let (inbound, reverse) = route_art(&line.id, index, false, false);
            assert_eq!(styles.departing, reverse.arriving);
            assert_eq!(styles.arriving, reverse.departing);
            assert!(styles.departing.is_some() && styles.arriving.is_some());
            assert_eq!(outbound.current, inbound.current);
            assert_eq!(
                inbound.next,
                SegmentScenery::from_tags(&line.segments[index.saturating_sub(1)].scenery)
            );
        }
        let (at_end, styles) = route_art(&line.id, 12, true, false);
        assert_eq!(at_end.current, at_end.next);
        assert_eq!(styles.arriving, Some(StationStyle::UrbanTerminal));
        assert_eq!(route_art(&line.id, 12, false, true), Default::default());
    }
    #[test]
    fn route_context_tracks_segments_and_clears_for_excursions() {
        let (first, styles) = route_art("aurora-v1", 0, true, false);
        assert!(!first.current.is_empty());
        assert_eq!(styles.departing, Some(StationStyle::Plateau));
        assert_eq!(styles.arriving, Some(StationStyle::TimberHalt));
        let (last, styles) = route_art("aurora-v1", 3, true, false);
        assert_eq!(last.next, first.current);
        assert_eq!(styles.arriving, Some(StationStyle::Plateau));
        for (line, index, excursion) in [
            ("missing", 0, false),
            ("aurora-v1", 50, false),
            ("aurora-v1", 0, true),
        ] {
            assert_eq!(route_art(line, index, true, excursion), Default::default());
        }
    }
}
