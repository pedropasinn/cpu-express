//! Original scenery in independent, world-anchored parallax layers.
//!
//! All spatial periods divide SCROLL_PERIOD_PX. A biome never owns an offset:
//! both sides of a transition use exactly the same World and layer coordinates.

use eframe::egui::Color32;

use super::{
    palette::{Palette, dusk_amount, mix},
    sprites::{Canvas, rgb},
    theme::{Biome, RenderQuality, SceneOptions, Weather, finite_nonnegative},
};

#[derive(Clone, Copy)]
pub(super) struct View {
    pub scenery_life: super::SceneryLifeOptions,
    pub wind_speed_kmh: Option<f32>,
    pub precipitation_mm: Option<f32>,
    pub distance: f64,
    pub seconds: f64,
    pub light_seconds: f64,
    pub options: SceneOptions,
    pub quality: RenderQuality,
}

impl View {
    fn detailed(self) -> bool {
        self.quality == RenderQuality::Full && self.options.details
    }
}

pub(super) fn background(
    pixels: &mut [Color32],
    biome: Biome,
    view: View,
    tags: super::SegmentScenery,
) {
    let mut palette = Palette::at_time(biome, view.light_seconds, view.options.day_cycle);
    // A forest-tagged highland is a humid slope, not Aurora's arid ridge.
    // Reuse existing forest terrain colors; retain the biome's sky and lighting.
    if biome == Biome::HighDesert && tags.contains(crate::simulation::catalog::SceneryTag::Forest) {
        let forest = Palette::at_time(Biome::Forest, view.light_seconds, view.options.day_cycle);
        palette.far = forest.far;
        palette.far_shadow = forest.far_shadow;
        palette.ridge = forest.ridge;
        palette.ridge_shadow = forest.ridge_shadow;
        palette.ground = forest.ground;
        palette.ground_light = forest.ground_light;
        palette.foliage = forest.foliage;
        palette.foliage_light = forest.foliage_light;
        palette.foreground = forest.foreground;
        palette.foreground_shadow = forest.foreground_shadow;
        palette.ink = forest.ink;
    }
    let mut canvas = Canvas::new(pixels);
    sky(&mut canvas, &palette, view);
    if tags.is_empty() {
        match biome {
            Biome::GreenValley => valley(&mut canvas, &palette, view),
            Biome::Forest => forest(&mut canvas, &palette, view),
            Biome::Coast => coast(&mut canvas, &palette, view),
            Biome::HighDesert => desert(&mut canvas, &palette, view),
        }
        foreground(&mut canvas, &palette, biome, view);
    } else {
        super::landmarks::draw(&mut canvas, &palette, view, tags);
    }
    track(&mut canvas, view.distance, view.detailed());
    // Veil distant scenery only; the train and track remain readable.
    let strength = match view.options.weather {
        Weather::Clear => 0.0,
        Weather::Cloudy => 0.22,
        Weather::Fog => 0.53,
        Weather::Rain | Weather::Snow => 0.30,
        Weather::Thunderstorm => 0.45,
    };
    if strength > 0.0 {
        let tint = mix(
            rgb(132, 150, 156),
            rgb(45, 57, 78),
            dusk_amount(view.light_seconds, view.options.day_cycle),
        );
        let width = pixels.len() / 70;
        for pixel in pixels.iter_mut().take(width * 54) {
            *pixel = mix(*pixel, tint, strength);
        }
    }
}

fn shift(distance: f64, ratio: f64, period: f64) -> i32 {
    // Reduce BEFORE multiplying so even exceptional finite inputs are safe.
    (finite_nonnegative(distance).rem_euclid(period / ratio) * ratio).floor() as i32
}

fn translated_polygon<const N: usize>(
    canvas: &mut Canvas<'_>,
    x: i32,
    points: [(i32, i32); N],
    color: Color32,
) {
    let translated = points.map(|(px, py)| (px + x, py));
    canvas.polygon(&translated, color);
}

fn sky(canvas: &mut Canvas<'_>, p: &Palette, view: View) {
    canvas.rect(0, 0, canvas.width(), 70, p.sky[0]);
    canvas.rect(0, 12, canvas.width(), 12, p.sky[1]);
    canvas.rect(0, 24, canvas.width(), 14, p.sky[2]);
    canvas.rect(0, 38, canvas.width(), 32, p.sky[3]);

    let dusk = dusk_amount(view.light_seconds, view.options.day_cycle);
    let sun_color = mix(p.sun, rgb(223, 226, 208), dusk);
    if view.options.weather == Weather::Clear {
        canvas.circle(273, 17, 8, mix(p.sky[1], sun_color, 0.34));
        canvas.circle(273, 17, 6, sun_color);
        canvas.rect(269, 19, 9, 1, mix(sun_color, p.sky[1], 0.18));
    }
    if view.detailed() && view.options.weather == Weather::Clear {
        for (x, y) in [
            (18, 6),
            (55, 18),
            (92, 8),
            (132, 4),
            (178, 14),
            (226, 5),
            (305, 8),
        ] {
            let base = p.sky[usize::from(y >= 12)];
            canvas.pixel(x, y, mix(base, rgb(249, 224, 174), dusk * 0.88));
        }
    }

    if view.options.clouds && view.quality == RenderQuality::Full {
        let drift = shift(view.seconds, 0.8, 320.0);
        for tile in canvas.tiles(320) {
            cloud(canvas, tile + 28 - drift, 12, false, p);
            cloud(canvas, tile + 158 - drift, 7, true, p);
            if view.options.details {
                cloud(canvas, tile + 292 - drift, 25, true, p);
            }
        }
    }
}

fn cloud(canvas: &mut Canvas<'_>, x: i32, y: i32, small: bool, p: &Palette) {
    let width = if small { 24 } else { 39 };
    canvas.rect(x, y + 4, width, 2, p.cloud_shadow);
    canvas.rect(x + 2, y + 1, width - 5, 4, p.cloud);
    canvas.rect(x + 7, y - 1, 10, 3, p.cloud);
    if !small {
        canvas.rect(x + 19, y - 3, 10, 6, p.cloud);
        canvas.rect(x + 29, y + 2, 8, 2, p.cloud);
    }
}

fn valley(canvas: &mut Canvas<'_>, p: &Palette, view: View) {
    let far = shift(view.distance, 0.03125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - far;
        translated_polygon(
            canvas,
            x,
            [
                (0, 46),
                (0, 37),
                (22, 32),
                (44, 35),
                (74, 17),
                (98, 32),
                (127, 24),
                (158, 39),
                (184, 31),
                (208, 21),
                (238, 35),
                (264, 29),
                (287, 36),
                (307, 33),
                (320, 37),
                (320, 46),
            ],
            p.far,
        );
        if view.detailed() {
            translated_polygon(canvas, x, [(74, 17), (98, 32), (85, 31)], p.far_shadow);
            translated_polygon(canvas, x, [(208, 21), (238, 35), (220, 34)], p.far_shadow);
            translated_polygon(
                canvas,
                x,
                [(74, 17), (65, 23), (72, 22), (75, 23), (78, 21), (83, 24)],
                mix(p.cloud, p.far, 0.16),
            );
        }
    }

    if view.quality == RenderQuality::Full {
        let near = shift(view.distance, 0.0625, 320.0);
        for tile in canvas.tiles(320) {
            let x = tile - near;
            translated_polygon(
                canvas,
                x,
                [
                    (0, 50),
                    (0, 42),
                    (30, 34),
                    (50, 39),
                    (85, 29),
                    (113, 39),
                    (138, 34),
                    (168, 42),
                    (197, 32),
                    (230, 41),
                    (257, 35),
                    (283, 41),
                    (305, 36),
                    (320, 42),
                    (320, 50),
                ],
                p.ridge,
            );
            translated_polygon(canvas, x, [(85, 29), (113, 39), (98, 40)], p.ridge_shadow);
        }
    }

    let hill = shift(view.distance, 0.125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - hill;
        translated_polygon(
            canvas,
            x,
            [
                (0, 56),
                (0, 48),
                (35, 43),
                (70, 44),
                (100, 48),
                (138, 43),
                (165, 41),
                (197, 45),
                (227, 47),
                (256, 43),
                (288, 45),
                (320, 48),
                (320, 56),
            ],
            p.ground,
        );
        if view.detailed() {
            translated_polygon(
                canvas,
                x,
                [(8, 50), (48, 45), (76, 47), (33, 52)],
                p.ground_light,
            );
            canvas.line((x + 91, 50), (x + 122, 46), p.ground_light);
            canvas.line((x + 105, 52), (x + 139, 47), p.ground_light);
            cottage(canvas, x + 265, 45, p);
        }
    }
    canvas.rect(0, 52, canvas.width(), 5, p.ground);

    let trees = shift(view.distance, 0.25, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - trees;
        for (offset, height) in [
            (15, 15),
            (30, 11),
            (97, 12),
            (249, 17),
            (263, 13),
            (310, 10),
        ] {
            fir(canvas, x + offset, 53, height, p);
        }
        if view.detailed() {
            water_tower(canvas, x + 41, 53, p);
            signal(canvas, x + 283, 54, p);
        }
    }
    if view.detailed() {
        fence(canvas, p, view.distance, false);
    }
}

fn forest(canvas: &mut Canvas<'_>, p: &Palette, view: View) {
    // Reuse the original fir primitive, with three world-anchored depths and
    // a small stream clearing. Every period matches an existing parallax layer.
    let far = shift(view.distance, 0.03125, 320.0);
    let distant_palette = Palette {
        foliage: p.far,
        foliage_light: p.far_shadow,
        ink: p.far_shadow,
        ..*p
    };
    for tile in canvas.tiles(320) {
        let x = tile - far;
        for (offset, height) in [
            (8, 22),
            (35, 26),
            (62, 20),
            (89, 29),
            (122, 24),
            (156, 28),
            (191, 22),
            (223, 27),
            (258, 23),
            (290, 30),
            (318, 20),
        ] {
            fir(canvas, x + offset, 48, height, &distant_palette);
        }
    }
    canvas.rect(0, 45, canvas.width(), 12, p.ground);
    if view.quality == RenderQuality::Full {
        let middle = shift(view.distance, 0.0625, 320.0);
        let middle_palette = Palette {
            foliage: p.ridge,
            foliage_light: p.ridge_shadow,
            ink: p.ridge_shadow,
            ..*p
        };
        for tile in canvas.tiles(320) {
            let x = tile - middle;
            for (offset, height) in [
                (17, 25),
                (39, 30),
                (70, 27),
                (101, 32),
                (141, 23),
                (181, 27),
                (211, 31),
                (244, 24),
                (280, 29),
                (309, 26),
            ] {
                fir(canvas, x + offset, 52, height, &middle_palette);
            }
        }
    }
    let ground = shift(view.distance, 0.125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - ground;
        translated_polygon(
            canvas,
            x,
            [
                (112, 46),
                (121, 46),
                (99, 50),
                (125, 54),
                (163, 57),
                (131, 57),
                (85, 51),
            ],
            p.water,
        );
        if view.detailed() {
            canvas.line((x + 107, 50), (x + 125, 53), p.water_light);
            canvas.line((x + 128, 55), (x + 148, 56), p.water_light);
            cottage(canvas, x + 246, 51, p);
        }
    }
    let close = shift(view.distance, 0.25, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - close;
        for (offset, height) in [
            (12, 25),
            (33, 20),
            (60, 28),
            (87, 21),
            (178, 24),
            (202, 31),
            (228, 23),
            (290, 27),
            (312, 19),
        ] {
            fir(canvas, x + offset, 54, height, p);
        }
        if view.detailed() {
            signal(canvas, x + 144, 54, p);
            canvas.rect(x + 253, 51, 13, 3, p.roof);
            canvas.pixel(x + 253, 52, p.trim);
            canvas.pixel(x + 264, 52, p.trim);
        }
    }
}

fn coast(canvas: &mut Canvas<'_>, p: &Palette, view: View) {
    let far = shift(view.distance, 0.03125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - far;
        translated_polygon(
            canvas,
            x,
            [
                (0, 37),
                (0, 34),
                (18, 33),
                (40, 26),
                (54, 26),
                (67, 30),
                (84, 34),
                (116, 35),
                (144, 31),
                (159, 23),
                (175, 21),
                (192, 27),
                (217, 31),
                (244, 33),
                (270, 29),
                (288, 31),
                (310, 34),
                (320, 34),
                (320, 37),
            ],
            p.far,
        );
        if view.detailed() {
            translated_polygon(
                canvas,
                x,
                [(175, 21), (192, 27), (218, 32), (186, 29)],
                p.far_shadow,
            );
        }
    }
    canvas.rect(0, 36, canvas.width(), 17, p.water);
    canvas.rect(0, 36, canvas.width(), 1, p.water_light);
    canvas.rect(0, 40, canvas.width(), 1, mix(p.water, p.water_light, 0.30));
    if view.detailed() {
        let ripples = shift(view.distance, 0.125, 160.0);
        let shimmer = shift(view.seconds, 2.0, 160.0);
        for tile in canvas.tiles(160) {
            let x = tile - ripples;
            for (dx, y, width) in [(9, 38, 19), (35, 43, 12), (88, 40, 25), (136, 45, 13)] {
                canvas.rect(x + dx, y, width, 1, mix(p.water, p.water_light, 0.62));
            }
            canvas.rect(tile + 70 - shimmer, 39, 7, 1, p.water_light);
            canvas.rect(tile + 120 - shimmer, 44, 11, 1, p.water_light);
        }
        let boats = shift(view.distance, 0.0625, 320.0);
        for tile in canvas.tiles(320) {
            sailboat(canvas, tile + 47 - boats, 39, p);
            sailboat(canvas, tile + 220 - boats, 43, p);
        }
    }
    let dunes = shift(view.distance, 0.125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - dunes;
        translated_polygon(
            canvas,
            x,
            [
                (0, 57),
                (0, 51),
                (32, 49),
                (61, 50),
                (89, 47),
                (122, 46),
                (150, 48),
                (183, 51),
                (219, 48),
                (253, 49),
                (282, 47),
                (320, 51),
                (320, 57),
            ],
            p.ground_light,
        );
        translated_polygon(
            canvas,
            x,
            [(89, 48), (122, 47), (150, 49), (183, 52), (116, 52)],
            p.ground,
        );
    }
    canvas.rect(0, 53, canvas.width(), 4, p.ground);

    let objects = shift(view.distance, 0.25, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - objects;
        lighthouse(canvas, x + 273, 51, p, view);
        for (offset, height) in [(23, 7), (41, 5), (113, 4), (302, 6)] {
            beach_grass(canvas, x + offset, 54, height, p);
        }
        if view.detailed() {
            canvas.rect(x + 57, 46, 26, 2, p.ink);
            canvas.rect(x + 58, 46, 24, 1, p.wall);
            for post in [60, 70, 80] {
                canvas.rect(x + post, 48, 2, 6, p.roof);
            }
            canvas.rect(x + 97, 51, 8, 3, p.wall);
            canvas.line((x + 96, 51), (x + 106, 51), p.roof);
        }
    }
}

fn desert(canvas: &mut Canvas<'_>, p: &Palette, view: View) {
    let far = shift(view.distance, 0.03125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - far;
        translated_polygon(
            canvas,
            x,
            [
                (0, 49),
                (0, 36),
                (21, 36),
                (31, 31),
                (41, 30),
                (47, 21),
                (79, 21),
                (84, 30),
                (102, 33),
                (115, 39),
                (152, 36),
                (175, 36),
                (182, 27),
                (189, 26),
                (194, 17),
                (224, 17),
                (229, 27),
                (244, 30),
                (259, 37),
                (288, 34),
                (307, 36),
                (320, 36),
                (320, 49),
            ],
            p.far,
        );
        if view.detailed() {
            translated_polygon(
                canvas,
                x,
                [(71, 22), (79, 22), (84, 30), (101, 34), (84, 34)],
                p.far_shadow,
            );
            translated_polygon(
                canvas,
                x,
                [(214, 18), (224, 18), (229, 27), (241, 31), (220, 30)],
                p.far_shadow,
            );
            canvas.rect(x + 49, 22, 28, 1, mix(p.far, p.ground_light, 0.35));
            canvas.rect(x + 195, 18, 28, 1, mix(p.far, p.ground_light, 0.35));
        }
    }
    if view.quality == RenderQuality::Full {
        let near = shift(view.distance, 0.0625, 320.0);
        for tile in canvas.tiles(320) {
            let x = tile - near;
            translated_polygon(
                canvas,
                x,
                [
                    (0, 53),
                    (0, 43),
                    (30, 38),
                    (48, 39),
                    (56, 33),
                    (72, 33),
                    (78, 39),
                    (109, 44),
                    (141, 42),
                    (169, 36),
                    (182, 36),
                    (191, 42),
                    (225, 44),
                    (251, 39),
                    (266, 37),
                    (288, 39),
                    (320, 43),
                    (320, 53),
                ],
                p.ridge,
            );
            translated_polygon(
                canvas,
                x,
                [(68, 34), (72, 34), (78, 40), (103, 44), (79, 43)],
                p.ridge_shadow,
            );
            canvas.line((x + 158, 41), (x + 185, 41), p.ground_light);
        }
    }
    let sand = shift(view.distance, 0.125, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - sand;
        translated_polygon(
            canvas,
            x,
            [
                (0, 57),
                (0, 50),
                (34, 47),
                (77, 49),
                (112, 46),
                (147, 49),
                (184, 51),
                (225, 47),
                (268, 46),
                (299, 49),
                (320, 50),
                (320, 57),
            ],
            p.ground,
        );
        if view.detailed() {
            canvas.line((x + 34, 50), (x + 65, 51), p.ground_light);
            canvas.line((x + 122, 48), (x + 156, 51), p.ground_light);
            canvas.line((x + 209, 51), (x + 242, 49), p.ground_light);
        }
    }
    canvas.rect(0, 53, canvas.width(), 4, p.ground);
    let objects = shift(view.distance, 0.25, 320.0);
    for tile in canvas.tiles(320) {
        let x = tile - objects;
        cactus(canvas, x + 17, 54, 16, p);
        cactus(canvas, x + 103, 54, 9, p);
        cactus(canvas, x + 249, 54, 13, p);
        depot(canvas, x + 276, 52, p);
        if view.detailed() {
            telegraph(canvas, x + 49, 54, p);
            canvas.rect(x + 135, 51, 7, 3, p.ridge_shadow);
            canvas.rect(x + 136, 50, 5, 1, p.ground_light);
            canvas.rect(x + 230, 52, 4, 2, p.ridge_shadow);
        }
    }
}

fn fir(canvas: &mut Canvas<'_>, x: i32, base: i32, height: i32, p: &Palette) {
    canvas.rect(x, base - height + 3, 2, height - 1, p.ink);
    // Asymmetric branch clusters: broad lit planes instead of noisy edge pixels.
    for layer in 0..3 {
        let top = base - height + layer * (height / 5);
        let half = 2 + layer * 2;
        let bottom = top + height / 2;
        canvas.polygon(
            &[
                (x, top),
                (x - half, bottom - 1),
                (x - 1, bottom),
                (x + half + 2, bottom - 1),
            ],
            p.foliage,
        );
        canvas.polygon(
            &[
                (x, top + 1),
                (x - half + 1, bottom - 2),
                (x + 1, bottom - 2),
            ],
            p.foliage_light,
        );
        canvas.rect(x + 1, bottom - 1, half, 1, p.foliage);
    }
}

fn cottage(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette) {
    canvas.rect(x, base - 7, 13, 8, p.wall);
    canvas.polygon(
        &[(x - 2, base - 7), (x + 6, base - 13), (x + 15, base - 7)],
        p.roof,
    );
    canvas.line((x - 1, base - 8), (x + 6, base - 13), p.trim);
    canvas.rect(x + 10, base - 6, 3, 7, p.trim);
    canvas.rect(x, base, 13, 1, p.ink);
    canvas.rect(x + 3, base - 4, 3, 5, p.ink);
    canvas.rect(x + 8, base - 4, 3, 3, p.sun);
    canvas.rect(x + 10, base - 13, 2, 5, p.roof);
}

fn water_tower(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette) {
    canvas.rect(x + 2, base - 9, 2, 9, p.ink);
    canvas.rect(x + 11, base - 9, 2, 9, p.ink);
    canvas.line((x + 3, base - 8), (x + 11, base - 1), p.trim);
    canvas.line((x + 11, base - 8), (x + 3, base - 1), p.ink);
    canvas.rect(x, base - 17, 15, 9, p.roof);
    canvas.rect(x + 1, base - 16, 13, 2, p.trim);
    canvas.rect(x + 1, base - 10, 13, 1, p.ink);
    canvas.rect(x + 11, base - 15, 3, 5, p.ink);
    canvas.rect(x + 1, base - 13, 13, 1, p.trim);
    canvas.rect(x + 5, base - 15, 1, 5, p.wall);
    canvas.polygon(
        &[(x - 1, base - 17), (x + 7, base - 21), (x + 16, base - 17)],
        p.ink,
    );
    canvas.rect(x + 14, base - 12, 5, 2, p.ink);
    canvas.rect(x + 17, base - 10, 2, 4, p.ink);
}

fn signal(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette) {
    canvas.rect(x, base - 15, 2, 15, p.ink);
    canvas.rect(x - 2, base - 16, 6, 6, p.ink);
    canvas.pixel(x, base - 14, p.sun);
    canvas.pixel(x, base - 12, p.foliage_light);
    canvas.rect(x - 2, base - 1, 6, 1, p.trim);
}

fn fence(canvas: &mut Canvas<'_>, p: &Palette, distance: f64, wire: bool) {
    let offset = shift(distance, 0.25, 24.0);
    canvas.rect(0, 50, canvas.width(), 1, p.trim);
    if !wire {
        canvas.rect(0, 53, canvas.width(), 1, p.ink);
    }
    for index in -1..15 {
        let x = index * 24 - offset;
        canvas.rect(x, 48, 2, 7, p.ink);
        canvas.rect(x, 48, 1, 6, p.wall);
    }
}

fn sailboat(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette) {
    canvas.rect(x, base, 13, 1, p.ink);
    canvas.rect(x + 2, base + 1, 9, 1, p.roof);
    canvas.rect(x + 6, base - 11, 1, 11, p.ink);
    canvas.polygon(
        &[(x + 5, base - 10), (x - 1, base - 2), (x + 5, base - 2)],
        p.wall,
    );
    canvas.polygon(
        &[(x + 7, base - 8), (x + 11, base - 2), (x + 7, base - 2)],
        p.trim,
    );
}

fn lighthouse(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette, view: View) {
    canvas.polygon(
        &[
            (x, base),
            (x + 3, base - 24),
            (x + 9, base - 24),
            (x + 12, base),
        ],
        p.wall,
    );
    canvas.line((x + 9, base - 23), (x + 11, base - 1), p.trim);
    canvas.rect(x + 2, base - 13, 8, 4, p.roof);
    canvas.rect(x + 4, base - 21, 3, 3, p.ink);
    canvas.rect(x + 5, base - 5, 3, 5, p.ink);
    canvas.rect(x + 1, base - 26, 10, 3, p.ink);
    canvas.rect(x + 3, base - 25, 6, 2, p.sun);
    canvas.polygon(
        &[(x, base - 26), (x + 6, base - 31), (x + 12, base - 26)],
        p.roof,
    );
    canvas.rect(x - 2, base - 1, 16, 2, p.ink);
    if view.detailed() {
        let glow = dusk_amount(view.light_seconds, view.options.day_cycle);
        let color = mix(p.sky[1], p.sun, glow * 0.65);
        if glow > 0.0 {
            canvas.rect(x - 9, base - 25, 9, 1, color);
            canvas.rect(x + 12, base - 25, 9, 1, color);
        }
    }
}

fn beach_grass(canvas: &mut Canvas<'_>, x: i32, base: i32, height: i32, p: &Palette) {
    canvas.line((x, base), (x - 3, base - height), p.foliage);
    canvas.line((x, base), (x + 2, base - height - 1), p.foliage_light);
    canvas.line((x, base), (x + 5, base - height / 2), p.foliage);
}

fn cactus(canvas: &mut Canvas<'_>, x: i32, base: i32, height: i32, p: &Palette) {
    canvas.rect(x, base - height, 4, height, p.foliage);
    canvas.rect(x + 1, base - height + 1, 1, height - 1, p.foliage_light);
    canvas.rect(x - 4, base - height + 4, 3, height / 2, p.foliage);
    canvas.rect(x - 3, base - height + 3, 1, 4, p.foliage_light);
    canvas.rect(x - 3, base - height / 2, 6, 3, p.foliage);
    canvas.rect(x + 6, base - height + 6, 3, height / 3, p.foliage);
    canvas.rect(x + 3, base - height / 2 + 2, 5, 2, p.foliage);
    canvas.pixel(x + 6, base - height + 5, p.foliage_light);
}

fn depot(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette) {
    canvas.rect(x, base - 14, 25, 14, p.wall);
    canvas.rect(x - 2, base - 15, 29, 3, p.roof);
    canvas.rect(x + 9, base - 18, 7, 4, p.wall);
    canvas.rect(x + 8, base - 18, 9, 1, p.trim);
    canvas.rect(x + 11, base - 8, 5, 8, p.ink);
    canvas.rect(x + 3, base - 9, 5, 4, p.ink);
    canvas.rect(x + 18, base - 9, 4, 4, p.ink);
    canvas.rect(x + 3, base - 9, 5, 1, p.sun);
    canvas.rect(x + 18, base - 9, 4, 1, p.sun);
    canvas.rect(x - 3, base - 1, 31, 2, p.roof);
    canvas.rect(x - 1, base - 11, 27, 1, p.trim);
}

fn telegraph(canvas: &mut Canvas<'_>, x: i32, base: i32, p: &Palette) {
    canvas.rect(x, base - 26, 2, 26, p.ink);
    canvas.rect(x - 6, base - 23, 14, 2, p.roof);
    for dx in [-5, 0, 5] {
        canvas.rect(x + dx, base - 25, 2, 2, p.wall);
    }
    canvas.line((x - 6, base - 23), (x - 20, base - 21), p.far_shadow);
    canvas.line((x + 8, base - 23), (x + 22, base - 21), p.far_shadow);
}

fn foreground(canvas: &mut Canvas<'_>, p: &Palette, biome: Biome, view: View) {
    canvas.rect(0, 61, canvas.width(), 9, p.foreground);
    canvas.rect(0, 66, canvas.width(), 4, p.foreground_shadow);
    if !view.detailed() {
        return;
    }
    let offset = shift(view.distance, 0.5, 160.0);
    for tile in canvas.tiles(160) {
        let x = tile - offset;
        for dx in [9, 35, 65, 119, 143] {
            if biome == Biome::HighDesert {
                canvas.rect(x + dx, 65, 3, 1, p.ground_light);
                canvas.pixel(x + dx + 5, 68, p.ink);
            } else {
                canvas.line((x + dx, 66), (x + dx - 2, 62), p.foliage);
                canvas.line((x + dx, 66), (x + dx + 2, 63), p.foliage_light);
            }
        }
        canvas.rect(x + 89, 63, 6, 2, p.ink);
        canvas.rect(x + 90, 62, 4, 1, p.ground_light);
        if biome == Biome::GreenValley {
            canvas.pixel(x + 18, 64, p.sun);
            canvas.pixel(x + 55, 67, p.wall);
        } else if biome == Biome::Coast {
            canvas.rect(x + 23, 64, 2, 1, p.wall);
            canvas.pixel(x + 101, 67, p.wall);
        }
    }
}

fn track(canvas: &mut Canvas<'_>, distance: f64, details: bool) {
    canvas.rect(0, 55, canvas.width(), 8, rgb(104, 109, 88));
    canvas.rect(0, 56, canvas.width(), 5, rgb(140, 129, 103));
    let offset = shift(distance, 0.5, 10.0);
    for index in -1..(canvas.width() / 10 + 2) {
        let x = index * 10 - offset;
        canvas.rect(x, 56, 5, 5, rgb(83, 87, 73));
        if details {
            canvas.rect(x, 58, 5, 1, rgb(173, 143, 98));
            canvas.pixel(x + 7, 61, rgb(176, 164, 126));
        }
    }
    // Wheel centers (54 + radius 3, and 55 + radius 2) contact this exact row.
    canvas.rect(0, 57, canvas.width(), 1, rgb(226, 215, 165));
    canvas.rect(0, 58, canvas.width(), 1, rgb(76, 95, 77));
    canvas.rect(0, 60, canvas.width(), 1, rgb(189, 184, 135));
}

pub(super) fn weather(canvas: &mut Canvas<'_>, view: View) {
    if !matches!(
        view.options.weather,
        Weather::Rain | Weather::Thunderstorm | Weather::Snow
    ) || view.quality == RenderQuality::Low
    {
        return;
    }
    let t = finite_nonnegative(view.seconds).rem_euclid(20.0);
    // Default movements loop exactly after 20 s; neither uses frame counts.
    let snow = view.options.weather == Weather::Snow;
    let default_count = if view.options.weather == Weather::Thunderstorm {
        40
    } else {
        24
    };
    // Bounded artistic density from an interval amount, not inferred mm/hour.
    // The condition above remains authoritative, including zero observed amount.
    let count = view.precipitation_mm.map_or(default_count, |mm| {
        if mm == 0.0 {
            0
        } else {
            (8.0 + (mm / 100.0).sqrt() * 56.0).round() as i32
        }
    });
    let drift = view.wind_speed_kmh.map_or(t * 16.0, |wind| {
        (finite_nonnegative(view.seconds) * f64::from(wind) * (48.0 / 250.0)).rem_euclid(320.0)
    });
    let slant = view
        .wind_speed_kmh
        .map_or(1, |wind| (wind * (4.0 / 250.0)).round() as i32);
    for tile in (0..canvas.width()).step_by(320) {
        for index in 0..count {
            let x = tile + (f64::from(index * 47) - drift).rem_euclid(320.0).floor() as i32;
            let y = (f64::from(index * 23) + t * if snow { 7.0 } else { 35.0 })
                .rem_euclid(70.0)
                .floor() as i32;
            if snow {
                canvas.pixel(x, y, rgb(225, 235, 235));
                if index % 3 == 0 {
                    canvas.pixel(x + 1, y, rgb(196, 215, 222));
                }
            } else {
                canvas.line((x, y), (x - slant, y + 3), rgb(176, 202, 201));
            }
        }
    }
}
