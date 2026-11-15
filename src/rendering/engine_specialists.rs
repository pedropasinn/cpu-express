//! Original fictional engines; all dimensions are authored on the native grid.
use super::sprites::{Canvas, rgb, wheel};
use eframe::egui::Color32;

const INK: Color32 = rgb(37, 56, 52);
const BODY: Color32 = rgb(42, 105, 88);
const LIGHT: Color32 = rgb(83, 143, 106);
const SHADE: Color32 = rgb(28, 79, 70);
const CREAM: Color32 = rgb(230, 218, 175);
const BRASS: Color32 = rgb(239, 194, 112);
const NAVY: Color32 = rgb(48, 71, 85);

fn window(c: &mut Canvas<'_>, x: i32, y: i32, lit: bool) {
    c.rect(x, y, 6, 6, INK);
    c.rect(
        x + 1,
        y + 1,
        4,
        4,
        if lit { BRASS } else { rgb(130, 181, 188) },
    );
    c.rect(x + 1, y + 1, 4, 1, CREAM);
}

fn chimney(c: &mut Canvas<'_>) {
    // The smoke emitter is fixed at (220, 32), including in Legacy.
    c.rect(217, 33, 5, 5, INK);
    c.rect(218, 34, 3, 3, SHADE);
    c.rect(216, 31, 8, 3, INK);
    c.rect(217, 31, 6, 1, BRASS);
}

fn lamp(c: &mut Canvas<'_>, x: i32, y: i32, lit: bool) {
    c.rect(x, y, 3, 4, INK);
    c.rect(
        x + 1,
        y + 1,
        2,
        2,
        if lit { rgb(255, 244, 190) } else { BRASS },
    );
}

fn steam_frame(c: &mut Canvas<'_>) {
    c.rect(180, 51, 54, 2, INK);
    c.rect(183, 53, 4, 1, BRASS);
    c.rect(184, 55, 3, 1, BRASS);
    c.polygon(&[(232, 50), (239, 56), (230, 56)], NAVY);
    c.line((232, 52), (236, 55), BRASS);
}

pub(super) fn express(c: &mut Canvas<'_>, phase: f64, lit: bool) {
    // Low streamlined boiler, cut-away skirts and three larger driving wheels.
    c.rect(183, 38, 15, 13, INK);
    c.rect(184, 39, 13, 11, BODY);
    c.rect(182, 36, 18, 2, NAVY);
    c.rect(184, 36, 14, 1, CREAM);
    window(c, 186, 39, lit);
    c.polygon(
        &[
            (198, 40),
            (204, 37),
            (226, 37),
            (234, 42),
            (234, 48),
            (198, 48),
        ],
        INK,
    );
    c.polygon(
        &[
            (199, 41),
            (205, 38),
            (225, 38),
            (232, 42),
            (232, 47),
            (199, 47),
        ],
        BODY,
    );
    c.rect(204, 38, 21, 2, LIGHT);
    c.rect(198, 46, 34, 2, SHADE);
    c.line((199, 44), (229, 44), BRASS);
    c.line((229, 44), (232, 42), BRASS);
    c.rect(204, 35, 5, 3, BRASS);
    for x in [209, 226] {
        c.rect(x, 40, 1, 5, CREAM);
    }
    chimney(c);
    lamp(c, 232, 40, lit);
    steam_frame(c);
    wheel(c, 189, 55, 2, phase);
    for x in [201, 213, 225] {
        wheel(c, x, 53, 4, phase);
    }
    wheel(c, 233, 55, 2, phase);
    let dx = (phase.cos() * 3.0).round() as i32;
    let dy = (phase.sin() * 3.0).round() as i32;
    c.line((201 + dx, 53 + dy), (225 + dx, 53 + dy), BRASS);
    c.line((225 + dx, 53 + dy), (230, 48), CREAM);
    for x in [201, 213, 225] {
        c.pixel(x + dx, 53 + dy, CREAM);
    }
}

pub(super) fn articulated(c: &mut Canvas<'_>, phase: f64, lit: bool) {
    // Two separate rod groups and cylinders below a narrow high boiler.
    c.rect(182, 33, 14, 18, INK);
    c.rect(183, 34, 12, 16, BODY);
    c.rect(181, 31, 17, 2, NAVY);
    c.rect(183, 31, 13, 1, CREAM);
    window(c, 185, 35, lit);
    c.rect(183, 43, 12, 2, BRASS);
    c.rect(196, 36, 33, 10, INK);
    c.rect(197, 37, 30, 8, BODY);
    c.rect(197, 37, 30, 2, LIGHT);
    c.rect(197, 43, 30, 2, SHADE);
    for x in [201, 211, 225] {
        c.rect(x, 38, 1, 6, BRASS);
    }
    c.rect(202, 33, 6, 3, BRASS);
    chimney(c);
    lamp(c, 230, 37, lit);
    c.rect(184, 47, 18, 3, SHADE);
    c.rect(214, 47, 16, 3, SHADE);
    steam_frame(c);
    for x in [189, 201, 213, 225] {
        wheel(c, x, 54, 3, phase + if x >= 213 { 1.2 } else { 0.0 });
    }
    wheel(c, 233, 55, 2, phase);
    for (left, right, offset) in [(189, 201, 0.0), (213, 225, 1.2)] {
        let dx = ((phase + offset).cos() * 2.0).round() as i32;
        let dy = ((phase + offset).sin() * 2.0).round() as i32;
        c.line((left + dx, 54 + dy), (right + dx, 54 + dy), BRASS);
        c.line((right + dx, 54 + dy), (right + 4, 49), CREAM);
        c.pixel(left + dx, 54 + dy, CREAM);
        c.pixel(right + dx, 54 + dy, CREAM);
    }
}

pub(super) fn twin_diesel(c: &mut Canvas<'_>, lit: bool) {
    // Two tall end cabs flank a recessed machinery deck and separate fans.
    for x in [184, 222] {
        c.rect(x, 33, 12, 18, INK);
        c.rect(x + 1, 34, 10, 16, BODY);
        c.rect(x - 1, 32, 14, 1, NAVY);
        c.rect(x + 1, 34, 10, 1, LIGHT);
        window(c, x + 3, 36, lit);
        c.rect(x + 1, 44, 10, 2, CREAM);
        c.rect(x + 1, 48, 10, 2, SHADE);
    }
    c.rect(196, 41, 26, 10, INK);
    c.rect(197, 42, 24, 8, BODY);
    c.rect(197, 42, 24, 1, LIGHT);
    c.rect(197, 48, 24, 2, SHADE);
    for x in [199, 212] {
        c.rect(x, 39, 7, 2, NAVY);
        c.rect(x + 1, 39, 5, 1, BRASS);
        c.rect(x, 44, 7, 3, NAVY);
        for offset in [1, 3, 5] {
            c.rect(x + offset, 44, 1, 3, CREAM);
        }
    }
    lamp(c, 233, 42, lit);
    c.pixel(183, 43, BRASS);
}

pub(super) fn rack_electric(c: &mut Canvas<'_>, lit: bool) {
    // Compact mountain cab, steep low nose, offset collector and visible cog.
    // This is visual equipment: no grade advantage or track mutation.
    c.rect(193, 37, 17, 14, INK);
    c.rect(194, 38, 15, 12, BODY);
    c.rect(191, 35, 21, 2, NAVY);
    c.rect(194, 36, 15, 1, CREAM);
    window(c, 196, 39, lit);
    c.rect(204, 39, 3, 10, SHADE);
    c.pixel(206, 46, BRASS);
    c.polygon(
        &[(210, 42), (217, 42), (230, 49), (230, 50), (210, 50)],
        INK,
    );
    c.polygon(&[(211, 43), (216, 43), (226, 48), (211, 48)], BODY);
    c.line((211, 44), (224, 48), LIGHT);
    c.rect(184, 46, 9, 5, INK);
    c.rect(185, 47, 7, 3, SHADE);
    for x in [185, 189, 230] {
        c.rect(x, 42, 1, 9, BRASS);
    }
    c.rect(184, 42, 8, 1, CREAM);
    c.line((197, 35), (204, 31), INK);
    c.line((204, 31), (208, 34), INK);
    c.rect(200, 30, 9, 1, BRASS);
    c.rect(197, 34, 11, 1, BRASS);
    c.circle(215, 49, 3, NAVY);
    for (x, y) in [(212, 49), (218, 49), (215, 46), (215, 50)] {
        c.pixel(x, y, BRASS);
    }
    c.pixel(215, 49, CREAM);
    lamp(c, 228, 45, lit);
}
