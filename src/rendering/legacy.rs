//! Baseline pastoral renderer, preserved pixel for pixel for rollback.
use super::engines::{self, EngineStyle};
use super::sprites::{Canvas, cloud, coach, locomotive, pine, rgb};
use crate::simulation::{Train, World};
use eframe::egui::Color32;

pub(super) fn draw(pixels: &mut [Color32], train: &Train, world: &World, engine: EngineStyle) {
    let mut canvas = Canvas::new(pixels);
    draw_sky(&mut canvas, world.elapsed_seconds());

    // The ratios and tile widths divide the world's wrapping period. That
    // makes scenery continuous even after a very long running session.
    draw_far_mountains(&mut canvas, world.scroll_px() * 0.0625);
    draw_mountains(&mut canvas, world.scroll_px() * 0.125);
    draw_hills(&mut canvas, world.scroll_px() * 0.25);
    draw_trees_and_fence(&mut canvas, world.scroll_px() * 0.5);
    draw_track(&mut canvas, world.scroll_px());

    canvas.origin = canvas.train_origin();
    for puff in world.smoke().iter().filter(|_| engine.emits_smoke()) {
        let life = (1.0 - puff.age / puff.lifetime).clamp(0.0, 1.0);
        let x = 220 + (puff.x * 0.5).round() as i32;
        let y = 32 + (puff.y * 0.5).round() as i32;
        let radius = (puff.size * 0.5).round().clamp(1.0, 6.0) as i32;
        canvas.smoke_circle(x, y, radius, (life * 205.0) as u8);
        canvas.smoke_circle(x - 1, y - 1, (radius - 1).max(1), (life * 70.0) as u8);
    }

    let phase = train.wheel_phase();
    for (index, x) in [58, 100, 142].into_iter().enumerate() {
        coach(&mut canvas, x, phase, index);
    }
    if engine == EngineStyle::Steam {
        locomotive(&mut canvas, phase);
    } else {
        engines::draw_modern(&mut canvas, phase, engine, false);
    }
    canvas.origin = 0;
    draw_foreground(&mut canvas, world.scroll_px());
}

fn draw_sky(canvas: &mut Canvas<'_>, seconds: f64) {
    canvas.rect(0, 0, canvas.width(), 70, rgb(239, 208, 169));
    canvas.rect(0, 10, canvas.width(), 11, rgb(244, 218, 179));
    canvas.rect(0, 21, canvas.width(), 17, rgb(247, 226, 187));
    canvas.rect(0, 38, canvas.width(), 15, rgb(241, 226, 186));
    canvas.circle(266, 17, 10, rgb(243, 219, 165));
    canvas.circle(266, 17, 8, rgb(255, 238, 187));
    canvas.rect(257, 17, 18, 1, rgb(252, 232, 177));
    let shift = (seconds * 0.8).rem_euclid(320.0).floor() as i32;
    for tile in canvas.tiles(320) {
        cloud(canvas, tile + 24 - shift, 16, false);
        cloud(canvas, tile + 146 - shift, 9, true);
        cloud(canvas, tile + 282 - shift, 30, true);
    }
}

fn draw_far_mountains(canvas: &mut Canvas<'_>, scroll: f64) {
    let shift = (scroll * 0.5).rem_euclid(320.0).floor() as i32;
    for tile in canvas.tiles(320) {
        let x = tile - shift;
        canvas.polygon(
            &[
                (x, 43),
                (x, 34),
                (x + 26, 27),
                (x + 47, 31),
                (x + 68, 18),
                (x + 91, 31),
                (x + 117, 24),
                (x + 151, 38),
                (x + 174, 31),
                (x + 194, 18),
                (x + 220, 33),
                (x + 248, 26),
                (x + 280, 35),
                (x + 302, 29),
                (x + 320, 34),
                (x + 320, 47),
            ],
            rgb(150, 174, 165),
        );
        canvas.polygon(
            &[(x + 68, 18), (x + 92, 32), (x + 78, 32)],
            rgb(130, 157, 154),
        );
        canvas.polygon(
            &[(x + 194, 18), (x + 219, 33), (x + 205, 34)],
            rgb(133, 158, 155),
        );
        canvas.polygon(
            &[
                (x + 68, 18),
                (x + 61, 22),
                (x + 66, 22),
                (x + 70, 21),
                (x + 74, 23),
            ],
            rgb(217, 215, 182),
        );
    }
}

fn draw_mountains(canvas: &mut Canvas<'_>, scroll: f64) {
    let shift = (scroll * 0.5).rem_euclid(320.0).floor() as i32;
    for tile in canvas.tiles(320) {
        let x = tile - shift;
        canvas.polygon(
            &[
                (x, 49),
                (x, 40),
                (x + 26, 34),
                (x + 47, 38),
                (x + 82, 27),
                (x + 111, 39),
                (x + 137, 32),
                (x + 166, 41),
                (x + 196, 29),
                (x + 229, 41),
                (x + 255, 33),
                (x + 281, 38),
                (x + 301, 34),
                (x + 320, 40),
                (x + 320, 50),
            ],
            rgb(106, 149, 137),
        );
        canvas.polygon(
            &[(x + 82, 27), (x + 111, 39), (x + 96, 41)],
            rgb(83, 130, 126),
        );
        canvas.polygon(
            &[(x + 196, 29), (x + 229, 41), (x + 208, 42)],
            rgb(80, 127, 123),
        );
        canvas.line((x + 57, 35), (x + 82, 27), rgb(148, 174, 142));
    }
}

fn draw_hills(canvas: &mut Canvas<'_>, scroll: f64) {
    let shift = (scroll * 0.5).rem_euclid(320.0).floor() as i32;
    for tile in canvas.tiles(320) {
        let x = tile - shift;
        canvas.polygon(
            &[
                (x, 53),
                (x, 47),
                (x + 35, 42),
                (x + 66, 43),
                (x + 95, 48),
                (x + 136, 42),
                (x + 163, 40),
                (x + 196, 43),
                (x + 222, 47),
                (x + 252, 42),
                (x + 285, 44),
                (x + 320, 47),
                (x + 320, 54),
            ],
            rgb(154, 174, 113),
        );
        canvas.polygon(
            &[(x + 5, 49), (x + 56, 44), (x + 75, 46), (x + 31, 51)],
            rgb(185, 190, 129),
        );
        canvas.rect(x + 264, 43, 10, 7, rgb(223, 199, 143));
        canvas.polygon(
            &[(x + 262, 43), (x + 269, 38), (x + 276, 43)],
            rgb(160, 112, 76),
        );
        canvas.rect(x + 266, 45, 2, 4, rgb(95, 120, 94));
        canvas.rect(x + 271, 45, 2, 2, rgb(247, 224, 163));
    }
    canvas.rect(0, 50, canvas.width(), 5, rgb(160, 177, 111));
    canvas.rect(0, 53, canvas.width(), 3, rgb(187, 182, 119));
}

fn draw_trees_and_fence(canvas: &mut Canvas<'_>, scroll: f64) {
    let shift = (scroll * 0.5).rem_euclid(320.0).floor() as i32;
    for tile in canvas.tiles(320) {
        for (x, height) in [
            (12, 14),
            (26, 11),
            (87, 12),
            (156, 10),
            (245, 17),
            (257, 13),
            (307, 11),
        ] {
            pine(canvas, tile + x - shift, 52, height, false);
        }
    }
    let fence_shift = (scroll * 0.5).rem_euclid(24.0).floor() as i32;
    canvas.rect(0, 49, canvas.width(), 1, rgb(181, 158, 105));
    canvas.rect(0, 52, canvas.width(), 1, rgb(142, 138, 87));
    for index in -1..(canvas.width() / 24 + 2) {
        let x = index * 24 - fence_shift;
        canvas.rect(x, 47, 2, 7, rgb(142, 126, 83));
        canvas.rect(x, 47, 1, 6, rgb(221, 199, 138));
    }
}

fn draw_track(canvas: &mut Canvas<'_>, scroll: f64) {
    canvas.rect(0, 55, canvas.width(), 8, rgb(107, 116, 84));
    canvas.rect(0, 56, canvas.width(), 5, rgb(144, 134, 100));
    let shift = (scroll * 0.5).rem_euclid(10.0).floor() as i32;
    for index in -1..(canvas.width() / 10 + 2) {
        let x = index * 10 - shift;
        canvas.rect(x, 56, 5, 5, rgb(89, 92, 69));
        canvas.rect(x, 58, 5, 1, rgb(178, 146, 96));
        canvas.pixel(x + 7, 61, rgb(178, 167, 125));
    }
    canvas.rect(0, 57, canvas.width(), 1, rgb(226, 215, 165));
    canvas.rect(0, 58, canvas.width(), 1, rgb(76, 95, 77));
    canvas.rect(0, 60, canvas.width(), 1, rgb(189, 184, 135));
}

fn draw_foreground(canvas: &mut Canvas<'_>, scroll: f64) {
    canvas.rect(0, 63, canvas.width(), 7, rgb(103, 133, 78));
    canvas.rect(0, 65, canvas.width(), 5, rgb(78, 116, 74));
    let shift = (scroll * 0.5).rem_euclid(160.0).floor() as i32;
    for tile in canvas.tiles(160) {
        let x = tile - shift;
        for offset in [7, 27, 58, 83, 119, 142] {
            canvas.line((x + offset, 65), (x + offset - 2, 61), rgb(88, 125, 76));
            canvas.line((x + offset, 65), (x + offset + 2, 62), rgb(154, 168, 99));
        }
        for (offset, y) in [(14, 64), (63, 67), (128, 63)] {
            canvas.pixel(x + offset, y, rgb(244, 208, 115));
            canvas.pixel(x + offset + 2, y + 1, rgb(204, 179, 99));
        }
        canvas.rect(x + 96, 62, 5, 2, rgb(148, 147, 105));
        canvas.rect(x + 97, 61, 3, 1, rgb(183, 176, 126));
    }
}
