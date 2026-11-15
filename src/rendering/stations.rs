//! Original station architecture and active-time platform animation.

use super::{
    palette::{Palette, dusk_amount},
    scenery::View,
    sprites::Canvas,
    theme::{Biome, RenderQuality},
};

/// Presentation of one real station service, supplied by the domain owner.
/// Counts only control bounded visual representatives (at most three each),
/// never a passenger balance. Keep this value through departure and advance
/// progress only with active simulation time. Explicit zeros draw no passengers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StationService {
    pub progress: f32,
    pub boarding: u32,
    pub alighting: u32,
}

impl StationService {
    pub(super) fn normalized(self) -> Self {
        Self {
            progress: if self.progress.is_finite() {
                self.progress.clamp(0.0, 1.0)
            } else {
                0.0
            },
            ..self
        }
    }
}

// Shared scale and dock position make arrival/departure meet on the same pixel.
// Bounds include all authored decorations and platform passengers. Cull only
// when the entire station has left the framebuffer, never at a timer boundary.
fn station_x(distance: f64, arriving: bool, width: i32) -> Option<i32> {
    if !distance.is_finite() || distance < 0.0 {
        return None;
    }
    let x = 246.0 + distance * if arriving { 0.4 } else { -0.4 };
    (-80.0..f64::from(width + 32))
        .contains(&x)
        .then(|| x.round() as i32)
}

#[cfg(test)]
fn draw(
    canvas: &mut Canvas<'_>,
    world: &crate::simulation::World,
    service: Option<StationService>,
    view: View,
) {
    draw_styled(
        canvas,
        world,
        service,
        view,
        super::StationStyles::default(),
        (super::PassengerOptions::default(), view.options.weather),
    );
}

pub(super) fn draw_styled(
    canvas: &mut Canvas<'_>,
    world: &crate::simulation::World,
    service: Option<StationService>,
    view: View,
    styles: super::StationStyles,
    (options, weather): (super::PassengerOptions, super::Weather),
) {
    if let Some(branch) = world.excursion_snapshot().current {
        draw_excursion(canvas, branch, view);
        return;
    }
    let journey = world.journey_snapshot();
    // Only real-service presentation opts into the new cast. None retains
    // pixel-exact illustrative galleries, and excursions never borrow counts.
    let cast = service.map(|_| super::passengers::PlatformCast {
        seed: journey.seed,
        arrival: journey.stations_visited,
        options,
        weather,
    });
    let traveled = journey.segment_progress * journey.route_segment.length_units;
    let waiting = world.station_wait() > 0.0;
    if let Some(x) = station_x(if waiting { 0.0 } else { traveled }, false, canvas.width()) {
        let service = service
            .unwrap_or(StationService {
                progress: if waiting {
                    ((8.0 - world.station_wait()) / 8.0) as f32
                } else {
                    1.0
                },
                boarding: 2,
                alighting: 1,
            })
            .normalized();
        draw_station_content(
            canvas,
            x,
            (journey.route_segment.from.index(), styles.departing, cast),
            service,
            Biome::from(journey.route_segment.biome_hint),
            view,
            !waiting && traveled < 100.0 && (cast.is_none() || traveled >= 1.0),
        );
    }
    if !waiting
        && let Some(x) = station_x(journey.distance_to_next_station_units, true, canvas.width())
    {
        // Actual service counts describe the last arrival, never a future one.
        let approaching = if service.is_some() {
            StationService::default()
        } else {
            StationService {
                progress: 0.0,
                boarding: 2,
                alighting: 1,
            }
        };
        draw_station_content(
            canvas,
            x,
            (journey.route_segment.to.index(), styles.arriving, cast),
            approaching,
            Biome::from(journey.next_biome_hint),
            view,
            false,
        );
    }
}

const MIRANTE: usize = 4;
const CONCHAS: usize = 5;

fn draw_excursion(
    canvas: &mut Canvas<'_>,
    branch: crate::simulation::BranchExpedition,
    view: View,
) {
    use crate::simulation::{BranchKind, BranchPhase};
    let destination = match branch.kind {
        BranchKind::Serra => MIRANTE,
        BranchKind::Litoral => CONCHAS,
    };
    let biome = Biome::from(branch.kind.biome());
    // Match the domain's active 90/8/90-second trip. The parked main journey
    // cannot supply this distance. No service transactions exist at branch stops,
    // so never recycle Aurora's passenger counts or the legacy illustration.
    let seconds =
        super::theme::finite_nonnegative(branch.phase_seconds).min(branch.phase.duration());
    let traveled = seconds * 24.0;
    let remaining = (branch.phase.duration() - seconds) * 24.0;
    let empty = StationService::default();
    match branch.phase {
        BranchPhase::Outbound => {
            if let Some(x) = station_x(traveled, false, canvas.width()) {
                draw_station(
                    canvas,
                    x,
                    0,
                    empty,
                    Biome::GreenValley,
                    view,
                    traveled < 100.0,
                );
            }
            if let Some(x) = station_x(remaining, true, canvas.width()) {
                draw_station(canvas, x, destination, empty, biome, view, false);
            }
        }
        BranchPhase::Service => draw_station(canvas, 246, destination, empty, biome, view, false),
        BranchPhase::Return => {
            if let Some(x) = station_x(traveled, false, canvas.width()) {
                draw_station(canvas, x, destination, empty, biome, view, traveled < 100.0);
            }
            if let Some(x) = station_x(remaining, true, canvas.width()) {
                draw_station(canvas, x, 0, empty, Biome::GreenValley, view, false);
            }
        }
    }
}

fn draw_station(
    canvas: &mut Canvas<'_>,
    x: i32,
    station: usize,
    service: StationService,
    biome: super::Biome,
    view: View,
    departing: bool,
) {
    draw_station_with_style(canvas, x, (station, None), service, biome, view, departing);
}

fn draw_station_with_style(
    canvas: &mut Canvas<'_>,
    x: i32,
    (station, style): (usize, Option<super::StationStyle>),
    service: StationService,
    biome: super::Biome,
    view: View,
    departing: bool,
) {
    draw_station_content(
        canvas,
        x,
        (station, style, None),
        service,
        biome,
        view,
        departing,
    );
}

fn draw_station_content(
    canvas: &mut Canvas<'_>,
    x: i32,
    (station, style, cast): (
        usize,
        Option<super::StationStyle>,
        Option<super::passengers::PlatformCast>,
    ),
    service: StationService,
    biome: super::Biome,
    view: View,
    departing: bool,
) {
    use super::Biome;
    let p = Palette::at_time(biome, view.light_seconds, view.options.day_cycle);
    let ink = p.ink;
    let cream = p.wall;
    let roof = p.roof;
    let detailed = view.options.details && view.quality == RenderQuality::Full;
    if let Some(style) = style {
        super::station_architecture::draw(
            canvas,
            x,
            style,
            &p,
            dusk_amount(view.light_seconds, view.options.day_cycle) >= 0.35,
            detailed,
        );
    } else {
        // Platform remains on the established rail row; roof silhouettes vary by place.
        canvas.rect(x - 7, 54, 79, 3, ink);
        canvas.rect(x - 7, 54, 79, 1, p.trim);
        canvas.rect(x, 32, 48, 22, ink);
        canvas.rect(x + 1, 33, 46, 20, cream);
        canvas.rect(x + 39, 33, 8, 20, p.trim);
        canvas.rect(x + 1, 50, 46, 3, p.roof);
        match biome {
            Biome::HighDesert => {
                // Adobe parapet and a raised bell gable.
                canvas.rect(x - 2, 29, 52, 4, roof);
                canvas.rect(x - 1, 29, 50, 1, p.trim);
                canvas.rect(x + 16, 24, 16, 6, cream);
                canvas.rect(x + 15, 24, 18, 2, roof);
                canvas.rect(x + 22, 26, 4, 4, ink);
                canvas.pixel(x + 23, 28, p.sun);
            }
            _ => {
                let peak = if biome == Biome::Forest { 20 } else { 23 };
                canvas.polygon(
                    &[(x - 4, 33), (x + 10, peak), (x + 36, peak), (x + 52, 33)],
                    ink,
                );
                canvas.polygon(
                    &[
                        (x - 2, 32),
                        (x + 10, peak + 1),
                        (x + 36, peak + 1),
                        (x + 49, 32),
                    ],
                    roof,
                );
                canvas.line((x + 10, peak + 1), (x + 36, peak + 1), p.trim);
                if detailed {
                    canvas.line((x + 5, 28), (x + 42, 28), p.trim);
                    canvas.rect(x + 38, peak - 2, 4, 7, ink);
                    canvas.rect(x + 38, peak - 2, 3, 5, roof);
                }
            }
        }
        // Recessed windows and doorway share a consistent light direction.
        let glass = if dusk_amount(view.light_seconds, view.options.day_cycle) >= 0.35 {
            p.sun
        } else {
            p.water_light
        };
        for dx in [4, 32] {
            canvas.rect(x + dx, 37, 10, 10, ink);
            canvas.rect(x + dx + 1, 38, 8, 7, glass);
            canvas.rect(x + dx + 5, 38, 1, 7, p.trim);
            canvas.rect(x + dx, 42, 10, 1, p.trim);
            canvas.rect(x + dx - 1, 47, 12, 1, p.trim);
        }
        canvas.rect(x + 19, 39, 9, 15, ink);
        canvas.rect(x + 20, 40, 7, 12, roof);
        canvas.rect(x + 21, 41, 5, 5, glass);
        canvas.pixel(x + 25, 49, p.trim);
        canvas.rect(x + 17, 53, 13, 1, cream);
        // Canopy, braces and sign stay readable at native scale.
        canvas.rect(x - 4, 35, 56, 2, ink);
        canvas.rect(x - 4, 35, 56, 1, roof);
        for dx in [-2, 49] {
            canvas.rect(x + dx, 37, 1, 17, ink);
            canvas.line(
                (x + dx, 41),
                (x + dx + if dx < 0 { 4 } else { -4 }, 37),
                ink,
            );
        }
        canvas.rect(x + 16, 32, 15, 4, ink);
        canvas.rect(x + 18, 33, 11, 1, p.trim);
        if detailed {
            // A tiny station ordinal plaque, no illegible fake text.
            if station < MIRANTE {
                for dx in 0..=station % 4 {
                    canvas.pixel(x + 20 + dx as i32 * 2, 34, cream);
                }
            } else if station == MIRANTE {
                canvas.line((x + 20, 34), (x + 23, 32), cream);
                canvas.line((x + 23, 32), (x + 26, 34), cream);
            } else {
                canvas.rect(x + 21, 34, 5, 1, cream);
                for dx in [21, 23, 25] {
                    canvas.pixel(x + dx, 32, cream);
                }
            }
            match biome {
                Biome::Forest => {
                    // Timber truss, a hanging lantern and stacked cut logs.
                    canvas.line((x + 11, 23), (x + 23, 31), p.trim);
                    canvas.line((x + 35, 23), (x + 23, 31), p.trim);
                    canvas.rect(x + 22, 24, 2, 7, ink);
                    canvas.rect(x + 7, 37, 1, 3, ink);
                    canvas.rect(x + 6, 40, 3, 3, p.sun);
                    for dx in [2, 16, 30, 44] {
                        canvas.rect(x + dx, 37, 1, 16, roof);
                    }
                    canvas.rect(x + 53, 50, 12, 3, roof);
                    for dx in [55, 59, 63] {
                        canvas.pixel(x + dx, 51, p.trim);
                    }
                    for (dx, y) in [(56, 49), (60, 49), (58, 46)] {
                        canvas.circle(x + dx, y, 2, ink);
                        canvas.circle(x + dx, y, 1, p.trim);
                    }
                }
                Biome::Coast => {
                    // White harbor cupola and alternating canvas awning panels.
                    canvas.rect(x + 18, 19, 11, 10, ink);
                    canvas.rect(x + 19, 20, 9, 8, cream);
                    canvas.rect(x + 21, 21, 5, 4, p.water);
                    canvas.rect(x + 17, 18, 13, 2, roof);
                    for dx in (0..48).step_by(6) {
                        canvas.rect(x + dx, 35, 3, 2, cream);
                    }
                    canvas.rect(x + 54, 37, 1, 17, ink);
                    let flutter = ((view.seconds * 2.0).floor() as u64 % 2) as i32;
                    canvas.polygon(
                        &[(x + 55, 37), (x + 64, 39 + flutter), (x + 55, 42)],
                        p.wall,
                    );
                    canvas.rect(x + 53, 51, 8, 3, roof);
                    canvas.rect(x + 55, 50, 4, 1, p.trim);
                }
                Biome::HighDesert => {
                    // Stepped stone coping, bell rope and sparse masonry joints.
                    canvas.rect(x + 18, 22, 12, 2, roof);
                    canvas.rect(x + 23, 27, 2, 3, p.trim);
                    canvas.rect(x + 24, 30, 1, 2, ink);
                    for (dx, y) in [(2, 34), (35, 34), (2, 51), (31, 51)] {
                        canvas.rect(x + dx, y, 5, 1, p.trim);
                    }
                    canvas.rect(x + 53, 48, 6, 6, roof);
                    canvas.rect(x + 54, 47, 4, 1, p.trim);
                    canvas.rect(x + 56, 43, 1, 5, p.foliage);
                    canvas.rect(x + 54, 44, 4, 1, p.foliage);
                }
                Biome::GreenValley => {
                    // A little clock dormer makes the village station recognizable.
                    canvas.rect(x + 18, 22, 12, 10, ink);
                    canvas.rect(x + 19, 23, 10, 8, cream);
                    canvas.circle(x + 24, 27, 3, roof);
                    canvas.circle(x + 24, 27, 2, p.trim);
                    canvas.line((x + 24, 25), (x + 24, 27), ink);
                    canvas.pixel(x + 25, 27, ink);
                    for dx in [4, 33] {
                        canvas.rect(x + dx, 48, 9, 2, roof);
                        canvas.rect(x + dx + 1, 47, 7, 1, p.foliage);
                        canvas.pixel(x + dx + 2, 47, p.sun);
                    }
                    canvas.rect(x + 54, 48, 11, 2, roof);
                    for dx in [55, 63] {
                        canvas.rect(x + dx, 50, 1, 4, ink);
                    }
                }
            }
        }
        if station >= MIRANTE {
            draw_landmark(canvas, x, station, &p);
        }
    }
    if detailed {
        // Platform edge joints remain present across all phases.
        for dx in (0..70).step_by(8) {
            canvas.pixel(x + dx, 55, p.trim);
        }
    }
    if let Some(cast) = cast {
        let crowd_style = style.or(Some(match biome {
            Biome::GreenValley => super::StationStyle::Plateau,
            Biome::Forest => super::StationStyle::TimberHalt,
            Biome::HighDesert => super::StationStyle::MountainStone,
            Biome::Coast => super::StationStyle::PortTerminal,
        }));
        super::passengers::platform(
            canvas,
            x,
            cast,
            service,
            crowd_style,
            &p,
            (view.seconds, detailed, departing),
        );
        return;
    }
    if detailed {
        // Original illustrative porter for callers without real service data.
        canvas.rect(x + 66, 46, 2, 2, cream);
        canvas.rect(x + 65, 48, 4, 4, p.water);
        canvas.rect(x + 66, 52, 1, 2, ink);
        canvas.rect(x + 68, 52, 1, 2, ink);
        let wave = departing && ((view.seconds * 3.0).floor() as u64).is_multiple_of(2);
        canvas.line((x + 69, 49), (x + 71, if wave { 46 } else { 50 }), p.trim);
        canvas.pixel(x + 71, if wave { 45 } else { 49 }, p.foliage);
    }
    let progress = f64::from(service.progress);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    for (boarding, count) in [(true, service.boarding), (false, service.alighting)] {
        for index in 0..count.min(3) {
            let start = if boarding { 12 + index as i32 * 6 } else { -28 };
            let end = if boarding { -28 } else { 32 + index as i32 * 6 };
            let px = x + (f64::from(start) + f64::from(end - start) * eased).round() as i32;
            // Boarders end behind the train; retain that occlusion on departure
            // instead of showing the same passenger again as the platform moves.
            if boarding && progress >= 1.0 || !boarding && progress <= 0.0 {
                continue;
            }
            canvas.rect(px, 46, 2, 2, cream);
            canvas.rect(px - 1, 48, 4, 4, if boarding { roof } else { p.water });
            let walking = progress > 0.0 && progress < 1.0;
            let stride = i32::from(
                walking && !((view.seconds * 4.0).rem_euclid(2.0) as u32 + index).is_multiple_of(2),
            );
            canvas.rect(px - stride, 52, 1, 2, ink);
            canvas.rect(px + 2 + stride, 52, 1, 2, ink);
            canvas.pixel(px + 3, 51, p.trim);
        }
    }
}

fn draw_landmark(canvas: &mut Canvas<'_>, x: i32, station: usize, p: &Palette) {
    if station == MIRANTE {
        // Tall stone lookout, open viewing balcony and a small brass telescope.
        // Everything remains inside the platform's existing horizontal bounds.
        canvas.rect(x + 53, 30, 14, 24, p.ink);
        canvas.rect(x + 54, 31, 12, 22, p.wall);
        for y in [34, 40, 46, 52] {
            canvas.rect(x + 54, y, 12, 1, p.trim);
        }
        canvas.rect(x + 57, 39, 6, 14, p.roof);
        canvas.rect(x + 58, 40, 4, 12, p.ink);
        canvas.rect(x + 51, 29, 18, 2, p.roof);
        canvas.rect(x + 51, 24, 18, 1, p.ink);
        for dx in [51, 55, 63, 68] {
            canvas.rect(x + dx, 24, 1, 5, p.ink);
        }
        canvas.line((x + 60, 28), (x + 60, 21), p.ink);
        canvas.line((x + 56, 22), (x + 64, 18), p.trim);
        canvas.line((x + 57, 23), (x + 65, 19), p.roof);
        canvas.line((x + 59, 27), (x + 57, 29), p.ink);
        canvas.line((x + 61, 27), (x + 63, 29), p.ink);
    } else {
        // Dockside derrick, suspended hook, quay bollards and shell mosaic.
        canvas.rect(x + 60, 26, 3, 28, p.roof);
        canvas.rect(x + 61, 27, 1, 26, p.trim);
        canvas.line((x + 61, 26), (x + 45, 22), p.ink);
        canvas.line((x + 61, 31), (x + 45, 22), p.roof);
        canvas.line((x + 46, 23), (x + 46, 29), p.ink);
        canvas.rect(x + 45, 29, 3, 1, p.trim);
        for dx in [54, 67] {
            canvas.rect(x + dx, 51, 3, 3, p.ink);
            canvas.rect(x + dx - 1, 50, 5, 1, p.trim);
        }
        canvas.rect(x + 19, 20, 9, 8, p.water);
        for (dx, y) in [(20, 22), (22, 21), (24, 21), (26, 22)] {
            canvas.line((x + dx, y), (x + 23, 26), p.trim);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        rendering::{HEIGHT, SceneOptions, SceneRenderer, WIDTH},
        simulation::{Journey, Train, World},
    };
    use eframe::egui::Color32;

    fn view() -> View {
        View {
            scenery_life: super::super::SceneryLifeOptions::default(),
            wind_speed_kmh: None,
            precipitation_mm: None,
            distance: 0.0,
            seconds: 0.5,
            light_seconds: 180.0,
            options: SceneOptions::default(),
            quality: RenderQuality::Full,
        }
    }

    #[test]
    fn authored_styles_keep_bounds_and_meet_at_the_platform_in_every_quality() {
        use super::super::{StationStyle, StationStyles};
        for style in StationStyle::ALL {
            for quality in [RenderQuality::Full, RenderQuality::Low] {
                for light_seconds in [180.0, 348.0, 480.0] {
                    let view = View {
                        quality,
                        light_seconds,
                        ..view()
                    };
                    for x in [-80, 352] {
                        let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
                        draw_station_with_style(
                            &mut Canvas::new(&mut pixels),
                            x,
                            (0, Some(style)),
                            StationService::default(),
                            Biome::Forest,
                            view,
                            false,
                        );
                        assert!(pixels.iter().all(|p| *p == Color32::TRANSPARENT));
                    }
                    let styles = StationStyles {
                        departing: Some(style),
                        arriving: Some(style),
                    };
                    let frame = |distance| {
                        let mut world = World::default();
                        world.restore_journey(
                            Journey::restore_checkpoint(7, distance, 1000.0).unwrap(),
                        );
                        let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
                        draw_styled(
                            &mut Canvas::new(&mut pixels),
                            &world,
                            Some(StationService::default()),
                            view,
                            styles,
                            (
                                super::super::PassengerOptions::default(),
                                view.options.weather,
                            ),
                        );
                        pixels
                    };
                    let boundary = Journey::from_seed(7).route().segments()[0].length_units;
                    assert_eq!(frame(boundary - 0.001), frame(boundary));
                    assert_eq!(frame(boundary), frame(boundary + 0.001));
                    let pixels = frame(boundary);
                    assert!(
                        pixels[57 * WIDTH..]
                            .iter()
                            .all(|p| *p == Color32::TRANSPARENT)
                    );
                }
            }
        }
    }

    fn platform(world: &World, service: StationService) -> Vec<Color32> {
        let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
        draw(&mut Canvas::new(&mut pixels), world, Some(service), view());
        pixels
    }

    #[test]
    fn excursions_move_off_aurora_and_keep_landmarks_across_phase_boundaries() {
        use crate::simulation::{BranchExpedition, BranchKind, BranchPhase};
        for kind in [BranchKind::Serra, BranchKind::Litoral] {
            let frame = |phase, phase_seconds| {
                let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
                draw_excursion(
                    &mut Canvas::new(&mut pixels),
                    BranchExpedition {
                        kind,
                        phase,
                        phase_seconds,
                    },
                    view(),
                );
                pixels
            };
            let arriving = frame(BranchPhase::Outbound, 90.0);
            assert_eq!(arriving, frame(BranchPhase::Service, 0.0));
            assert_eq!(arriving, frame(BranchPhase::Return, 0.0));
            assert_eq!(
                frame(BranchPhase::Outbound, 0.0),
                frame(BranchPhase::Return, 90.0)
            );
            assert!(
                frame(BranchPhase::Outbound, 45.0)
                    .iter()
                    .all(|p| *p == Color32::TRANSPARENT)
            );
            let mut main = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
            draw_station(
                &mut Canvas::new(&mut main),
                246,
                2,
                StationService::default(),
                Biome::from(kind.biome()),
                view(),
                false,
            );
            assert_ne!(arriving, main, "branch must have its own authored landmark");
        }
    }

    #[test]
    fn all_station_identities_and_positions_survive_arrival_and_departure() {
        let journey = Journey::default();
        let seed = journey.snapshot().seed;
        let mut distance = 0.0;
        for segment in journey.route().segments() {
            distance += segment.length_units;
            let mut world = World::default();
            world.restore_journey(
                Journey::restore_checkpoint(seed, distance - 0.001, 10.0).unwrap(),
            );
            let before = platform(&world, StationService::default());
            world.restore_journey(Journey::restore_checkpoint(seed, distance, 10.0).unwrap());
            world.set_station_wait(8.0);
            assert_eq!(before, platform(&world, StationService::default()));
            world.set_station_wait(0.0);
            assert_eq!(before, platform(&world, StationService::default()));
            world.restore_journey(
                Journey::restore_checkpoint(seed, distance + 100.0, 10.0).unwrap(),
            );
            let departed = platform(&world, StationService::default());
            assert_ne!(before, departed);
            assert!(departed.iter().any(|p| *p != Color32::TRANSPARENT));
        }
    }

    #[test]
    fn culling_only_removes_fully_offscreen_art() {
        for x in [-80, 352] {
            for biome in [
                Biome::GreenValley,
                Biome::Forest,
                Biome::Coast,
                Biome::HighDesert,
            ] {
                let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
                draw_station(
                    &mut Canvas::new(&mut pixels),
                    x,
                    0,
                    StationService {
                        progress: 0.5,
                        boarding: 3,
                        alighting: 3,
                    },
                    biome,
                    view(),
                    true,
                );
                assert!(pixels.iter().all(|p| *p == Color32::TRANSPARENT));
            }
        }
        assert_eq!(station_x(0.0, true, 320), station_x(0.0, false, 320));
        assert_eq!(station_x(100.0, false, 320), Some(206));
        assert_eq!(station_x(1000.0, false, 320), None);
        assert_eq!(station_x(f64::NAN, true, 320), None);
    }

    #[test]
    fn explicit_zero_service_never_invents_passengers_and_counts_are_bounded() {
        let mut world = World::default();
        world.set_station_wait(4.0);
        let empty = platform(&world, StationService::default());
        for progress in [0.0, 0.5, 1.0] {
            assert_eq!(
                empty,
                platform(
                    &world,
                    StationService {
                        progress,
                        ..StationService::default()
                    }
                )
            );
        }
        let crowded = platform(
            &world,
            StationService {
                progress: 0.5,
                boarding: 3,
                alighting: 3,
            },
        );
        assert_ne!(empty, crowded);
        assert_eq!(
            crowded,
            platform(
                &world,
                StationService {
                    progress: 0.5,
                    boarding: u32::MAX,
                    alighting: u32::MAX
                }
            )
        );
        assert_eq!(
            StationService {
                progress: f32::NAN,
                ..StationService::default()
            }
            .normalized()
            .progress,
            0.0
        );
    }

    #[test]
    fn active_platform_animation_freezes_during_pause() {
        let config = Config::default();
        let mut train = Train::new(&config);
        let mut world = World::default();
        world.set_station_wait(4.0);
        let mut renderer = SceneRenderer::new();
        renderer.set_station_service(Some(StationService {
            progress: 0.5,
            boarding: 2,
            alighting: 2,
        }));
        let before = renderer.render(&train, &world).to_vec();
        world.advance_expedition(&mut train, 0.0, 0.3, &config);
        let moving = renderer.render(&train, &world).to_vec();
        assert_ne!(before, moving);
        world.set_paused(true);
        for _ in 0..3 {
            world.advance_expedition(&mut train, 100.0, 1.0, &config);
            assert_eq!(moving, renderer.render(&train, &world));
        }
    }
}
