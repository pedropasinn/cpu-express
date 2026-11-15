//! Fictional generic locomotives, authored directly on the native pixel grid.
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

pub(super) fn freight_steam(c: &mut Canvas<'_>, phase: f64, lit: bool) {
    // A saddle-tank freight engine: tall water tank, rear bunker and four
    // coupled axles. The original chimney mouth and coupling sockets stay put.
    c.rect(182, 39, 8, 12, INK);
    c.rect(183, 40, 6, 10, SHADE);
    c.rect(183, 38, 5, 2, NAVY);
    c.rect(188, 34, 17, 17, INK);
    c.rect(189, 35, 15, 15, BODY);
    c.rect(187, 32, 19, 2, NAVY);
    c.rect(189, 32, 15, 1, CREAM);
    window(c, 190, 37, lit);
    window(c, 197, 37, lit);
    c.rect(189, 46, 15, 2, BRASS);
    c.rect(204, 37, 24, 14, INK);
    c.rect(205, 38, 21, 12, BODY);
    c.rect(205, 38, 21, 2, LIGHT);
    c.rect(205, 48, 21, 2, SHADE);
    c.rect(205, 44, 21, 1, BRASS);
    for x in [207, 224] {
        c.rect(x, 40, 1, 8, BRASS);
    }
    c.rect(211, 41, 9, 3, NAVY);
    c.rect(213, 42, 5, 1, BRASS);
    c.rect(208, 35, 4, 2, BRASS);
    c.rect(217, 33, 5, 5, INK);
    c.rect(218, 34, 3, 3, rgb(174, 91, 57));
    c.rect(216, 31, 8, 3, INK);
    c.rect(217, 31, 6, 1, BRASS);
    c.rect(227, 40, 3, 11, INK);
    c.rect(228, 42, 1, 5, CREAM);
    c.rect(231, 41, 3, 4, INK);
    c.rect(232, 42, 2, 2, if lit { rgb(255, 244, 190) } else { BRASS });
    c.rect(180, 51, 54, 2, INK);
    c.polygon(&[(232, 50), (239, 56), (230, 56)], NAVY);
    c.line((232, 52), (236, 55), BRASS);
    c.rect(183, 53, 4, 1, BRASS);
    c.rect(184, 55, 3, 1, BRASS);
    for x in [189, 201, 213, 225] {
        wheel(c, x, 54, 3, phase);
    }
    wheel(c, 233, 55, 2, phase);
    let dx = (phase.cos() * 2.0).round() as i32;
    let dy = (phase.sin() * 2.0).round() as i32;
    c.line((189 + dx, 54 + dy), (225 + dx, 54 + dy), BRASS);
    c.line((225 + dx, 54 + dy), (230, 50), CREAM);
    for x in [189, 201, 213, 225] {
        c.pixel(x + dx, 54 + dy, CREAM);
    }
}

pub(super) fn shunter(c: &mut Canvas<'_>, lit: bool) {
    // Central cab, two short low hoods and open walkways.
    c.rect(199, 34, 16, 17, INK);
    c.rect(200, 35, 14, 15, BODY);
    c.rect(198, 32, 18, 2, NAVY);
    c.rect(200, 32, 14, 1, CREAM);
    window(c, 201, 36, lit);
    window(c, 208, 36, lit);
    c.rect(201, 43, 12, 1, CREAM);
    c.rect(205, 45, 5, 5, SHADE);
    c.pixel(209, 46, BRASS);
    for (x, width) in [(187, 12), (215, 14)] {
        c.rect(x, 43, width, 8, INK);
        c.rect(x + 1, 44, width - 2, 6, BODY);
        c.rect(x + 1, 44, width - 2, 1, LIGHT);
        c.rect(x + 1, 49, width - 2, 1, SHADE);
        for offset in [3, 6, 9] {
            c.rect(x + offset, 46, 1, 2, NAVY);
        }
    }
    for x in [184, 232] {
        c.rect(x, 43, 1, 8, BRASS);
        c.rect(x - 1, 43, 3, 1, CREAM);
    }
    c.rect(229, 49, 6, 2, NAVY);
    c.pixel(231, 49, BRASS);
    c.pixel(233, 50, BRASS);
    c.pixel(232, 45, if lit { rgb(255, 244, 190) } else { BRASS });
}

pub(super) fn railcar(c: &mut Canvas<'_>, lit: bool) {
    // Rounded passenger railmotor with a clerestory and a sloping driving cab.
    c.polygon(
        &[
            (183, 38),
            (187, 34),
            (224, 34),
            (230, 37),
            (235, 44),
            (235, 50),
            (183, 50),
        ],
        INK,
    );
    c.polygon(
        &[
            (184, 39),
            (188, 35),
            (223, 35),
            (229, 38),
            (233, 44),
            (233, 49),
            (184, 49),
        ],
        BODY,
    );
    c.rect(189, 33, 32, 1, NAVY);
    c.rect(192, 32, 26, 1, CREAM);
    c.rect(186, 45, 45, 2, CREAM);
    c.rect(184, 47, 49, 2, LIGHT);
    c.rect(184, 49, 49, 1, SHADE);
    for x in [187, 195, 203, 211, 222] {
        window(c, x, 38, lit);
    }
    c.line((229, 39), (232, 43), rgb(130, 181, 188));
    c.rect(219, 37, 1, 13, CREAM);
    c.pixel(218, 45, BRASS);
    c.rect(231, 45, 3, 1, if lit { rgb(255, 244, 190) } else { BRASS });
    c.rect(204, 48, 8, 1, BRASS);
}
