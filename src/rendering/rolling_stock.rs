//! Original rolling stock sharing the baseline's wheel, rail and smoke geometry.
//!
//! Cars are drawn directly into the native 320 × 70 framebuffer. Their silhouettes
//! and details use whole cells, with no decoded textures, heap work or randomness.

use eframe::egui::Color32;

use super::engines::{self, EngineStyle};
use super::sprites::{Canvas, coach, locomotive, rgb, wheel};

/// A purely visual coach choice; it has no effect on speed or simulation state.
/// The array passed to [`super::SceneRenderer::set_coaches`] is ordered tail first.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachStyle {
    /// The familiar four-window coach from the original scene.
    Passenger,
    /// A blue sleeping car with small, curtained compartments.
    Sleeper,
    /// An open wooden wagon carrying crates and a covered load.
    Freight,
    /// A cream cylindrical tank with copper straps and a filling hatch.
    Tank,
    /// A green viewing car with a raised panoramic glass canopy.
    Observation,
    Dining,
    Mail,
    Gondola,
    Hopper,
    Timber,
    Refrigerator,
    Livestock,
    Container,
    Autorack,
    PressureTank,
    Crane,
    Bicycle,
    Coil,
    Fair,
}

impl CoachStyle {
    pub const ALL: [Self; 19] = [
        Self::Passenger,
        Self::Sleeper,
        Self::Freight,
        Self::Tank,
        Self::Observation,
        Self::Dining,
        Self::Mail,
        Self::Gondola,
        Self::Hopper,
        Self::Timber,
        Self::Refrigerator,
        Self::Livestock,
        Self::Container,
        Self::Autorack,
        Self::PressureTank,
        Self::Crane,
        Self::Bicycle,
        Self::Coil,
        Self::Fair,
    ];

    pub fn name(self) -> &'static str {
        crate::simulation::ServiceWagon::from(self).name()
    }
}

macro_rules! wagon_adapters {
    ($($variant:ident),+ $(,)?) => {
        impl From<CoachStyle> for crate::simulation::ServiceWagon {
            fn from(style: CoachStyle) -> Self {
                match style { $(CoachStyle::$variant => Self::$variant),+ }
            }
        }
        impl From<crate::simulation::ServiceWagon> for CoachStyle {
            fn from(wagon: crate::simulation::ServiceWagon) -> Self {
                match wagon { $(crate::simulation::ServiceWagon::$variant => Self::$variant),+ }
            }
        }
    };
}
wagon_adapters!(
    Passenger,
    Sleeper,
    Freight,
    Tank,
    Observation,
    Dining,
    Mail,
    Gondola,
    Hopper,
    Timber,
    Refrigerator,
    Livestock,
    Container,
    Autorack,
    PressureTank,
    Crane,
    Bicycle,
    Coil,
    Fair,
);

const INK: Color32 = rgb(38, 55, 56);
const GOLD: Color32 = rgb(230, 195, 125);
const GLASS: Color32 = rgb(143, 185, 181);
const GLASS_LIGHT: Color32 = rgb(219, 231, 204);
const LAMP: Color32 = rgb(255, 222, 145);
const LAMP_SHADOW: Color32 = rgb(204, 151, 94);

pub(super) fn draw_train(
    canvas: &mut Canvas<'_>,
    phase: f64,
    coaches: &[CoachStyle; 3],
    engine: EngineStyle,
    illuminated: bool,
    details: bool,
) {
    // Train already guarantees a valid phase. Keep this drawing boundary robust
    // for future preview adapters without touching that simulation contract.
    let phase = if phase.is_finite() { phase } else { 0.0 };
    for (index, (&style, x)) in coaches.iter().zip([58, 100, 142]).enumerate() {
        draw_coach(canvas, x, phase, style, index, illuminated, details);
    }

    if engine != EngineStyle::Steam {
        engines::draw_modern(canvas, phase, engine, illuminated);
        return;
    }

    // Preserve the original running gear and sockets: chimney mouth (220, 32), driving
    // wheels (201/213/225, 54), leading wheel (233, 55), and crank remain fixed.
    locomotive(canvas, phase);
    if details {
        engine_body(canvas);
        canvas.rect(186, 47, 5, 2, INK);
        canvas.rect(187, 47, 3, 1, GOLD);
        canvas.pixel(204, 44, rgb(221, 222, 170));
        canvas.pixel(221, 48, rgb(221, 222, 170));
        canvas.rect(207, 50, 13, 1, rgb(181, 122, 70));
        canvas.pixel(201, 48, rgb(203, 158, 91));
    }
    if illuminated {
        // Discrete highlights, never blurred halos or extra smoke particles.
        canvas.rect(189, 41, 3, 4, LAMP_SHADOW);
        canvas.rect(189, 41, 3, 2, LAMP);
        canvas.pixel(195, 42, LAMP);
        canvas.pixel(232, 42, LAMP);
        canvas.rect(232, 43, 3, 1, rgb(255, 244, 190));
        canvas.pixel(233, 44, LAMP);
    }
}

/// Sculpt the upper body without touching the legacy sprite or running gear.
/// Three broad enamel bands remain the livery replacement keys in scene.rs.
fn engine_body(canvas: &mut Canvas<'_>) {
    let copper = rgb(174, 91, 57);
    let brass = rgb(239, 194, 112);
    canvas.rect(184, 35, 17, 1, rgb(97, 119, 105));
    canvas.rect(185, 36, 14, 1, INK);
    canvas.rect(185, 38, 2, 11, copper);
    canvas.rect(187, 47, 11, 3, rgb(105, 61, 48));
    canvas.rect(188, 48, 8, 1, copper);
    canvas.rect(196, 40, 1, 6, rgb(85, 127, 115));
    canvas.rect(200, 42, 25, 2, rgb(83, 143, 106));
    canvas.rect(200, 44, 26, 4, rgb(42, 105, 88));
    canvas.rect(200, 48, 26, 3, rgb(28, 79, 70));
    // Boiler hoops wrap the curved volume, with dark lower edges.
    for x in [204, 222] {
        canvas.rect(x, 42, 2, 8, copper);
        canvas.rect(x, 42, 1, 6, brass);
        canvas.pixel(x + 1, 49, INK);
    }
    canvas.rect(208, 44, 11, 1, rgb(153, 185, 132));
    canvas.rect(211, 46, 6, 3, INK);
    canvas.rect(212, 47, 4, 1, brass);
    canvas.rect(201, 50, 25, 1, copper);
    canvas.rect(227, 42, 2, 8, INK);
    canvas.rect(228, 44, 1, 3, rgb(114, 137, 119));
    canvas.rect(230, 41, 4, 1, INK);
    canvas.pixel(231, 42, brass);
    // Steam dome, whistle and handrail are deliberately separated silhouettes.
    canvas.rect(207, 38, 3, 1, brass);
    canvas.rect(210, 39, 2, 2, rgb(121, 68, 49));
    canvas.rect(213, 37, 1, 4, brass);
    canvas.pixel(212, 37, INK);
}

fn draw_coach(
    canvas: &mut Canvas<'_>,
    x: i32,
    phase: f64,
    style: CoachStyle,
    index: usize,
    illuminated: bool,
    details: bool,
) {
    match style {
        CoachStyle::Passenger => {
            coach(canvas, x, phase, index);
            if illuminated {
                for column in 0..4 {
                    let wx = x + 5 + column * 8;
                    canvas.rect(wx, 42, 4, 4, LAMP_SHADOW);
                    canvas.rect(wx, 42, 4, 2, LAMP);
                    canvas.rect(wx + 2, 42, 1, 4, GOLD);
                }
            }
            if details {
                for dx in [8, 27] {
                    canvas.rect(x + dx, 34, 3, 1, INK);
                }
                canvas.pixel(x + 35, 46, GOLD);
            }
        }
        CoachStyle::Sleeper => sleeper(canvas, x, illuminated, details),
        CoachStyle::Freight => freight(canvas, x, details),
        CoachStyle::Tank => tank(canvas, x, details),
        CoachStyle::Observation => observation(canvas, x, illuminated, details),
        CoachStyle::Dining => super::stock_details::dining(canvas, x, illuminated),
        CoachStyle::Mail => super::stock_details::mail(canvas, x),
        CoachStyle::Gondola => super::stock_details::gondola(canvas, x),
        CoachStyle::Hopper => super::stock_details::hopper(canvas, x),
        CoachStyle::Timber => super::stock_details::timber(canvas, x),
        CoachStyle::Refrigerator => super::stock_details::refrigerator(canvas, x),
        CoachStyle::Livestock => super::stock_specialists::livestock(canvas, x),
        CoachStyle::Container => super::stock_specialists::container(canvas, x),
        CoachStyle::Autorack => super::stock_specialists::autorack(canvas, x),
        CoachStyle::PressureTank => super::stock_specialists::pressure_tank(canvas, x),
        CoachStyle::Crane => super::stock_specialists::crane(canvas, x),
        CoachStyle::Bicycle => super::stock_specialists::bicycle(canvas, x),
        CoachStyle::Coil => super::stock_specialists::coil(canvas, x),
        CoachStyle::Fair => super::stock_specialists::fair(canvas, x),
    }

    if details {
        coach_finish(canvas, x, style);
    }

    // Baseline's wheels and couplers are shared by every silhouette. Redrawing
    // these for Passenger is avoided so that its no-detail path stays exact.
    if style != CoachStyle::Passenger {
        chassis(canvas, x, phase, details);
    }
}

fn coach_finish(canvas: &mut Canvas<'_>, x: i32, style: CoachStyle) {
    match style {
        CoachStyle::Dining
        | CoachStyle::Mail
        | CoachStyle::Gondola
        | CoachStyle::Hopper
        | CoachStyle::Timber
        | CoachStyle::Refrigerator
        | CoachStyle::Livestock
        | CoachStyle::Container
        | CoachStyle::Autorack
        | CoachStyle::PressureTank
        | CoachStyle::Crane
        | CoachStyle::Bicycle
        | CoachStyle::Coil
        | CoachStyle::Fair => {}
        CoachStyle::Passenger | CoachStyle::Sleeper => {
            // Rounded clerestory roof and a shaded eave, not a flat gold slab.
            canvas.rect(x + 3, 35, 32, 1, rgb(139, 158, 142));
            canvas.rect(x + 1, 36, 36, 1, rgb(92, 114, 108));
            canvas.rect(x, 38, 38, 1, INK);
            canvas.rect(x + 3, 50, 32, 1, INK);
            for dx in [3, 33] {
                canvas.rect(x + dx, 47, 1, 3, rgb(112, 92, 70));
            }
            canvas.rect(x + 16, 48, 6, 1, GOLD);
        }
        CoachStyle::Freight => {
            // Recessed plank seams, rope and folded canvas make the load read.
            canvas.rect(x + 4, 48, 7, 1, rgb(104, 66, 51));
            canvas.rect(x + 27, 48, 7, 1, rgb(104, 66, 51));
            canvas.line((x + 19, 37), (x + 17, 41), rgb(136, 166, 132));
            canvas.rect(x + 28, 38, 2, 4, rgb(41, 79, 79));
            canvas.rect(x + 21, 37, 1, 6, GOLD);
            canvas.rect(x + 5, 40, 7, 1, rgb(122, 87, 58));
        }
        CoachStyle::Tank => {
            // End cap has a stepped ellipse; the body retains clean broad bands.
            canvas.line((x + 6, 41), (x + 5, 45), rgb(237, 222, 175));
            canvas.rect(x + 30, 42, 2, 5, rgb(139, 152, 131));
            canvas.rect(x + 7, 49, 23, 1, rgb(103, 120, 106));
            canvas.rect(x + 17, 38, 6, 1, rgb(237, 222, 175));
            canvas.rect(x + 19, 34, 2, 1, INK);
        }
        CoachStyle::Observation => {
            // Interior seat silhouettes below the glass, with broad reflections.
            for dx in [7, 17, 27] {
                canvas.rect(x + dx, 41, 3, 3, rgb(53, 86, 80));
                canvas.line((x + dx, 38), (x + dx + 2, 40), GLASS_LIGHT);
            }
            canvas.rect(x + 7, 32, 24, 1, rgb(156, 177, 152));
            canvas.rect(x + 5, 45, 28, 1, INK);
            canvas.rect(x + 5, 48, 28, 1, rgb(40, 84, 76));
        }
    }
}

fn chassis(canvas: &mut Canvas<'_>, x: i32, phase: f64, details: bool) {
    canvas.rect(x - 2, 51, 42, 2, INK);
    canvas.rect(x + 1, 50, 36, 1, rgb(170, 143, 98));
    for offset in [7, 30] {
        wheel(canvas, x + offset, 54, 3, phase);
        canvas.rect(x + offset - 3, 51, 7, 1, INK);
        if details {
            canvas.rect(x + offset - 2, 52, 5, 1, rgb(108, 124, 111));
        }
    }
}

fn sleeper(canvas: &mut Canvas<'_>, x: i32, illuminated: bool, details: bool) {
    canvas.rect(x, 38, 38, 13, INK);
    canvas.rect(x + 1, 39, 36, 11, rgb(62, 91, 123));
    canvas.rect(x + 2, 39, 34, 1, GOLD);
    canvas.rect(x + 2, 47, 34, 2, rgb(92, 121, 149));
    canvas.rect(x + 1, 49, 36, 1, GOLD);
    canvas.rect(x - 1, 36, 40, 3, INK);
    canvas.rect(x + 1, 35, 36, 1, rgb(94, 109, 118));
    canvas.rect(x + 2, 36, 34, 1, rgb(182, 180, 153));
    for column in 0..5 {
        let wx = x + 3 + column * 6;
        canvas.rect(wx, 41, 4, 6, INK);
        canvas.rect(
            wx + 1,
            42,
            2,
            4,
            if illuminated { LAMP_SHADOW } else { GLASS },
        );
        canvas.rect(
            wx + 1,
            42,
            2,
            2,
            if illuminated { LAMP } else { GLASS_LIGHT },
        );
        if details {
            canvas.pixel(wx + 2, 44, rgb(150, 96, 80));
        }
    }
    canvas.rect(x + 34, 40, 1, 8, GOLD);
    canvas.pixel(x + 35, 45, GOLD);
    if details {
        canvas.rect(x + 6, 34, 4, 1, INK);
        canvas.rect(x + 27, 34, 4, 1, INK);
        canvas.rect(x + 16, 48, 5, 1, rgb(231, 205, 157));
    }
}

fn freight(canvas: &mut Canvas<'_>, x: i32, details: bool) {
    // Load first, then the enclosing plank sides. Its lower silhouette is
    // deliberately different from a passenger coach at native resolution.
    canvas.rect(x + 3, 37, 11, 9, INK);
    canvas.rect(x + 4, 38, 9, 7, rgb(184, 135, 82));
    canvas.rect(x + 5, 38, 7, 1, rgb(229, 188, 120));
    canvas.rect(x + 15, 39, 17, 7, rgb(41, 79, 79));
    canvas.polygon(
        &[(x + 15, 40), (x + 19, 35), (x + 29, 36), (x + 33, 42)],
        rgb(63, 110, 103),
    );
    canvas.line((x + 19, 36), (x + 28, 37), rgb(118, 155, 125));
    canvas.rect(x, 43, 38, 8, INK);
    canvas.rect(x + 1, 44, 36, 6, rgb(141, 79, 57));
    canvas.rect(x + 1, 44, 36, 1, rgb(218, 162, 97));
    canvas.rect(x + 1, 47, 36, 1, rgb(189, 115, 72));
    for offset in [2, 12, 25, 35] {
        canvas.rect(x + offset, 43, 2, 7, rgb(73, 76, 64));
        if details {
            canvas.pixel(x + offset, 45, GOLD);
            canvas.pixel(x + offset, 49, GOLD);
        }
    }
    if details {
        canvas.line((x + 5, 39), (x + 11, 43), rgb(122, 87, 58));
        canvas.line((x + 11, 39), (x + 5, 43), rgb(122, 87, 58));
        canvas.line((x + 21, 36), (x + 23, 42), rgb(202, 173, 120));
        canvas.rect(x + 16, 46, 6, 3, rgb(222, 193, 135));
        canvas.rect(x + 17, 47, 4, 1, rgb(125, 85, 60));
    }
}

fn tank(canvas: &mut Canvas<'_>, x: i32, details: bool) {
    let body = rgb(191, 188, 151);
    let light = rgb(237, 222, 175);
    let shade = rgb(139, 152, 131);
    canvas.circle(x + 8, 44, 6, INK);
    canvas.circle(x + 29, 44, 6, INK);
    canvas.rect(x + 8, 38, 22, 13, INK);
    canvas.circle(x + 8, 44, 5, body);
    canvas.circle(x + 29, 44, 5, body);
    canvas.rect(x + 8, 39, 22, 11, body);
    canvas.rect(x + 6, 40, 25, 2, light);
    canvas.rect(x + 6, 47, 26, 2, shade);
    canvas.rect(x + 17, 35, 6, 3, INK);
    canvas.rect(x + 18, 35, 4, 1, GOLD);
    canvas.rect(x + 18, 37, 4, 1, rgb(163, 121, 80));
    for offset in [12, 26] {
        canvas.rect(x + offset, 39, 2, 11, rgb(146, 100, 70));
        canvas.rect(x + offset, 40, 1, 2, GOLD);
    }
    if details {
        canvas.rect(x + 17, 43, 6, 3, rgb(57, 102, 94));
        canvas.pixel(x + 19, 44, GOLD);
        canvas.rect(x + 34, 41, 1, 9, INK);
        for y in [42, 45, 48] {
            canvas.rect(x + 32, y, 3, 1, GOLD);
        }
    }
}

fn observation(canvas: &mut Canvas<'_>, x: i32, illuminated: bool, details: bool) {
    canvas.rect(x, 38, 38, 13, INK);
    canvas.rect(x + 1, 45, 36, 5, rgb(58, 113, 98));
    canvas.rect(x + 1, 46, 36, 1, rgb(111, 159, 126));
    canvas.rect(x + 1, 49, 36, 1, GOLD);
    // Raised glass gallery with a stepped cap, bounded by the same carriage.
    canvas.rect(x + 4, 33, 30, 3, INK);
    canvas.rect(x + 7, 32, 24, 1, rgb(126, 147, 134));
    canvas.rect(x + 5, 33, 28, 1, GOLD);
    canvas.rect(x + 3, 36, 32, 9, INK);
    canvas.rect(
        x + 4,
        36,
        30,
        8,
        if illuminated { LAMP_SHADOW } else { GLASS },
    );
    canvas.rect(
        x + 4,
        36,
        30,
        2,
        if illuminated { LAMP } else { GLASS_LIGHT },
    );
    canvas.rect(x + 4, 42, 30, 2, rgb(70, 116, 111));
    for offset in [4, 13, 23, 33] {
        canvas.rect(x + offset, 35, 1, 10, GOLD);
        canvas.rect(x + offset + 1, 38, 1, 7, INK);
    }
    canvas.rect(x - 1, 38, 5, 2, INK);
    canvas.rect(x + 34, 38, 5, 2, INK);
    canvas.rect(x + 1, 41, 1, 8, GOLD);
    canvas.rect(x + 36, 41, 1, 8, GOLD);
    if details {
        canvas.pixel(x + 7, 39, GLASS_LIGHT);
        canvas.pixel(x + 17, 40, GLASS_LIGHT);
        canvas.pixel(x + 27, 39, GLASS_LIGHT);
        canvas.rect(x + 14, 47, 10, 1, GOLD);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{HEIGHT, WIDTH};

    const STYLES: [CoachStyle; CoachStyle::ALL.len()] = CoachStyle::ALL;

    #[test]
    fn every_car_has_two_wheels_on_the_baseline_rail() {
        for style in STYLES {
            for phase in [0.0, 1.25, std::f64::consts::PI] {
                let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
                draw_coach(
                    &mut Canvas::new(&mut pixels),
                    58,
                    phase,
                    style,
                    0,
                    true,
                    true,
                );
                for x in [65, 88] {
                    assert_eq!(pixels[57 * WIDTH + x], rgb(39, 55, 53), "{style:?}");
                    assert_eq!(pixels[58 * WIDTH + x], Color32::TRANSPARENT, "{style:?}");
                }
                for (index, pixel) in pixels.iter().enumerate() {
                    if *pixel != Color32::TRANSPARENT {
                        assert!((56..98).contains(&(index % WIDTH)), "{style:?}");
                        assert!((32..=57).contains(&(index / WIDTH)), "{style:?}");
                        assert_eq!(pixel.a(), 255);
                    }
                }
            }
        }
    }

    #[test]
    fn carriage_styles_are_distinct_at_native_resolution() {
        let mut frames = Vec::new();
        for style in STYLES {
            let mut frame = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
            draw_coach(&mut Canvas::new(&mut frame), 58, 0.7, style, 0, false, true);
            for existing in &frames {
                assert_ne!(&frame, existing, "{style:?} must have its own silhouette");
            }
            frames.push(frame);
        }
    }

    #[test]
    fn new_cars_have_distinct_opaque_silhouettes_and_keep_couplers() {
        let mut masks = Vec::new();
        for style in CoachStyle::ALL {
            let mut frame = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
            draw_coach(&mut Canvas::new(&mut frame), 58, 0.7, style, 0, false, true);
            for x in [56, 57, 96, 97] {
                assert_ne!(frame[51 * WIDTH + x].a(), 0, "{style:?}");
            }
            let mask: Vec<bool> = frame.iter().map(|p| p.a() != 0).collect();
            // Passenger and Sleeper intentionally retain their historical outline.
            if !matches!(style, CoachStyle::Passenger | CoachStyle::Sleeper) {
                assert!(!masks.contains(&mask), "{style:?} needs a distinct outline");
            }
            masks.push(mask);
        }
    }

    #[test]
    fn engine_details_preserve_chimney_wheels_and_crank() {
        for phase in [0.0, 0.8, 2.5, 5.9] {
            let mut baseline = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
            locomotive(&mut Canvas::new(&mut baseline), phase);
            let mut detailed = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
            draw_train(
                &mut Canvas::new(&mut detailed),
                phase,
                &[CoachStyle::Freight; 3],
                EngineStyle::Steam,
                true,
                true,
            );
            for y in 53..=57 {
                assert_eq!(
                    &baseline[y * WIDTH + 180..y * WIDTH + 240],
                    &detailed[y * WIDTH + 180..y * WIDTH + 240],
                );
            }
            for y in 31..=38 {
                assert_eq!(
                    &baseline[y * WIDTH + 216..y * WIDTH + 224],
                    &detailed[y * WIDTH + 216..y * WIDTH + 224],
                );
            }
        }
    }

    #[test]
    fn passenger_without_new_effects_matches_original_sprite() {
        let mut baseline = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
        let mut current = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
        coach(&mut Canvas::new(&mut baseline), 100, 0.6, 1);
        draw_coach(
            &mut Canvas::new(&mut current),
            100,
            0.6,
            CoachStyle::Passenger,
            1,
            false,
            false,
        );
        assert_eq!(baseline, current);
    }

    #[test]
    fn all_coaches_clip_exactly_at_both_edges_in_both_detail_modes() {
        for style in CoachStyle::ALL {
            for details in [false, true] {
                for lit in [false, true] {
                    let mut reference = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
                    draw_coach(
                        &mut Canvas::new(&mut reference),
                        58,
                        0.7,
                        style,
                        0,
                        lit,
                        details,
                    );
                    for width in [160, 320, 641, 1920] {
                        for x in [-20, 58, width as i32 - 20] {
                            let mut frame = vec![Color32::TRANSPARENT; width * HEIGHT];
                            draw_coach(
                                &mut Canvas::new(&mut frame),
                                x,
                                0.7,
                                style,
                                0,
                                lit,
                                details,
                            );
                            for y in 0..HEIGHT {
                                for column in 0..width {
                                    let source = column as i32 - x + 58;
                                    let expected = if (0..WIDTH as i32).contains(&source) {
                                        reference[y * WIDTH + source as usize]
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    assert_eq!(
                                        frame[y * width + column],
                                        expected,
                                        "{style:?}, width={width}, x={x}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
