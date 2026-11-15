//! Original bodies for the content round. Shared chassis are drawn by the caller.
use super::sprites::{Canvas, rgb};
use eframe::egui::Color32;

const INK: Color32 = rgb(38, 55, 56);
const GOLD: Color32 = rgb(230, 195, 125);
const CREAM: Color32 = rgb(237, 222, 175);
const GREEN: Color32 = rgb(58, 113, 98);
const RUST: Color32 = rgb(141, 79, 57);

pub(super) fn dining(c: &mut Canvas<'_>, x: i32, lit: bool) {
    // Kitchen chimney and raised vent distinguish the restaurant roof.
    c.rect(x, 38, 38, 13, INK);
    c.rect(x + 1, 39, 36, 11, RUST);
    c.rect(x - 1, 36, 40, 2, INK);
    c.rect(x + 2, 35, 34, 1, CREAM);
    c.rect(x + 5, 33, 3, 3, INK);
    c.rect(x + 4, 32, 5, 1, GOLD);
    c.rect(x + 14, 34, 16, 1, INK);
    c.rect(x + 2, 48, 34, 1, GOLD);
    c.rect(x + 3, 40, 6, 7, INK);
    c.rect(x + 4, 41, 4, 5, rgb(143, 185, 181));
    for dx in [12, 23] {
        c.rect(x + dx, 40, 9, 7, INK);
        c.rect(
            x + dx + 1,
            41,
            7,
            5,
            if lit { GOLD } else { rgb(143, 185, 181) },
        );
        // White tablecloth, pedestal and a tiny bottle are legible at 1x.
        c.rect(x + dx + 2, 44, 5, 1, CREAM);
        c.rect(x + dx + 4, 45, 1, 2, CREAM);
        c.pixel(x + dx + 4, 43, GREEN);
    }
    c.rect(x + 34, 40, 1, 8, GOLD);
}

pub(super) fn mail(c: &mut Canvas<'_>, x: i32) {
    c.rect(x, 38, 38, 13, INK);
    c.rect(x + 1, 39, 36, 11, rgb(62, 91, 123));
    c.rect(x - 1, 37, 40, 2, INK);
    c.rect(x + 2, 36, 34, 1, rgb(139, 158, 142));
    c.rect(x + 1, 48, 36, 1, GOLD);
    // Wide sliding baggage door and sorting-room windows.
    c.rect(x + 14, 39, 13, 11, INK);
    c.rect(x + 15, 40, 11, 10, rgb(92, 121, 149));
    c.rect(x + 15, 39, 11, 1, GOLD);
    c.rect(x + 24, 43, 1, 4, GOLD);
    for dx in [3, 29] {
        c.rect(x + dx, 40, 6, 5, INK);
        c.rect(x + dx + 1, 41, 4, 3, rgb(219, 231, 204));
    }
    // Envelope emblem, not a real postal mark.
    c.rect(x + 16, 43, 6, 4, CREAM);
    c.line((x + 16, 43), (x + 19, 45), RUST);
    c.line((x + 21, 43), (x + 19, 45), RUST);
}

pub(super) fn gondola(c: &mut Canvas<'_>, x: i32) {
    // Angular ore heaps rise above a low, open steel body.
    c.polygon(
        &[
            (x + 2, 44),
            (x + 7, 38),
            (x + 13, 41),
            (x + 21, 36),
            (x + 29, 40),
            (x + 34, 38),
            (x + 36, 45),
        ],
        INK,
    );
    c.polygon(
        &[
            (x + 4, 43),
            (x + 8, 39),
            (x + 14, 42),
            (x + 21, 38),
            (x + 29, 42),
            (x + 34, 40),
            (x + 35, 44),
        ],
        rgb(112, 124, 115),
    );
    for (dx, y) in [(8, 40), (19, 40), (25, 41), (32, 42)] {
        c.rect(x + dx, y, 2, 1, rgb(182, 180, 153));
    }
    c.rect(x, 44, 38, 7, INK);
    c.rect(x + 1, 45, 36, 5, RUST);
    c.rect(x, 44, 38, 1, GOLD);
    for dx in [3, 11, 19, 27, 35] {
        c.rect(x + dx, 45, 1, 5, INK);
    }
}

pub(super) fn hopper(c: &mut Canvas<'_>, x: i32) {
    // Covered grain hopper: two sloping discharge pockets above the frame.
    c.polygon(
        &[
            (x + 1, 37),
            (x + 5, 34),
            (x + 32, 34),
            (x + 36, 37),
            (x + 35, 44),
            (x + 29, 49),
            (x + 24, 49),
            (x + 19, 44),
            (x + 14, 49),
            (x + 9, 49),
            (x + 2, 44),
        ],
        INK,
    );
    c.rect(x + 3, 37, 31, 6, rgb(191, 188, 151));
    c.rect(x + 6, 35, 25, 2, CREAM);
    for dx in [4, 20] {
        c.polygon(
            &[
                (x + dx, 43),
                (x + dx + 12, 43),
                (x + dx + 9, 47),
                (x + dx + 6, 47),
            ],
            rgb(139, 152, 131),
        );
        c.rect(x + dx + 5, 48, 6, 2, GOLD);
    }
    for dx in [7, 18, 29] {
        c.rect(x + dx, 37, 1, 7, INK);
    }
    for dx in [9, 24] {
        c.rect(x + dx, 33, 5, 1, INK);
    }
    c.rect(x + 35, 38, 1, 12, INK);
    for y in [40, 43, 46] {
        c.rect(x + 33, y, 4, 1, GOLD);
    }
}

pub(super) fn timber(c: &mut Canvas<'_>, x: i32) {
    // Low platform, stacked logs and tall stakes leave open sky between loads.
    c.rect(x, 48, 38, 3, INK);
    c.rect(x + 1, 49, 36, 1, GOLD);
    for (left, y, length) in [(2, 44, 34), (4, 40, 30), (8, 36, 25)] {
        c.rect(x + left, y, length, 4, INK);
        c.rect(x + left + 1, y + 1, length - 2, 2, rgb(174, 113, 70));
        c.rect(x + left + 2, y + 1, length - 6, 1, rgb(218, 162, 97));
        c.rect(x + left + length - 4, y + 1, 3, 2, CREAM);
        c.pixel(x + left + length - 3, y + 2, RUST);
    }
    for dx in [5, 18, 32] {
        c.rect(x + dx, 35, 2, 15, INK);
        c.rect(x + dx, 36, 1, 12, rgb(139, 152, 131));
    }
}

pub(super) fn refrigerator(c: &mut Canvas<'_>, x: i32) {
    c.rect(x, 36, 38, 15, INK);
    c.rect(x + 1, 37, 36, 13, CREAM);
    c.rect(x + 2, 35, 32, 1, rgb(139, 158, 142));
    c.rect(x + 1, 47, 36, 3, rgb(143, 185, 181));
    // Insulated door with hinges; external cooling unit at the right end.
    c.rect(x + 12, 38, 13, 12, INK);
    c.rect(x + 13, 39, 11, 10, rgb(219, 231, 204));
    c.rect(x + 21, 42, 1, 5, INK);
    for y in [40, 46] {
        c.rect(x + 11, y, 4, 1, GOLD);
    }
    c.rect(x + 28, 39, 9, 10, INK);
    c.rect(x + 29, 40, 7, 8, rgb(139, 152, 131));
    for y in [41, 43, 45] {
        c.rect(x + 30, y, 5, 1, INK);
    }
    // Simple snowflake identifies chilled supplies without lettering.
    c.line((x + 5, 40), (x + 9, 44), rgb(62, 91, 123));
    c.line((x + 5, 44), (x + 9, 40), rgb(62, 91, 123));
    c.rect(x + 7, 39, 1, 7, rgb(62, 91, 123));
}
