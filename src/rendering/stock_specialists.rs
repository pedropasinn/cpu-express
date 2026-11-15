//! Original fictional specialist wagons. The caller owns wheels and couplers.
//! All bodies fit the existing 42 x 28 thumbnail and leave the rail untouched.
use super::sprites::{Canvas, rgb};
use eframe::egui::Color32;

const INK: Color32 = rgb(38, 55, 56);
const GOLD: Color32 = rgb(230, 195, 125);
const CREAM: Color32 = rgb(237, 222, 175);
const GREEN: Color32 = rgb(58, 113, 98);
const RUST: Color32 = rgb(141, 79, 57);
const BLUE: Color32 = rgb(62, 91, 123);
const STEEL: Color32 = rgb(139, 152, 131);
const GLASS: Color32 = rgb(143, 185, 181);

pub(super) fn livestock(c: &mut Canvas<'_>, x: i32) {
    // Open ventilation slots, two animal profiles and a central sliding gate.
    c.rect(x + 2, 35, 34, 2, INK);
    c.rect(x, 37, 38, 2, INK);
    c.rect(x + 3, 35, 32, 1, STEEL);
    c.rect(x + 1, 39, 36, 11, RUST);
    for dx in [3, 23] {
        c.rect(x + dx, 39, 12, 8, INK);
        c.rect(x + dx + 2, 42, 7, 3, CREAM);
        c.rect(x + dx + 8, 40, 3, 3, CREAM);
        c.pixel(x + dx + 10, 40, GOLD);
        c.pixel(x + dx + 9, 41, INK);
        for leg in [2, 7] {
            c.rect(x + dx + leg, 45, 1, 2, CREAM);
        }
    }
    for y in [39, 44, 48] {
        c.rect(x + 1, y, 36, 1, GOLD);
    }
    c.rect(x + 16, 39, 6, 11, INK);
    c.rect(x + 17, 40, 4, 9, RUST);
    c.line((x + 17, 41), (x + 20, 47), GOLD);
    c.rect(x, 49, 38, 2, INK);
}

pub(super) fn container(c: &mut Canvas<'_>, x: i32) {
    // Two detachable ribbed boxes with different heights and an open gap.
    c.rect(x, 48, 38, 3, INK);
    for (dx, y, w, color) in [(1, 34, 21, BLUE), (24, 38, 13, RUST)] {
        c.rect(x + dx, y, w, 14 - (y - 34), INK);
        c.rect(x + dx + 1, y + 1, w - 2, 12 - (y - 34), color);
        c.rect(x + dx + 1, y + 1, w - 2, 1, GOLD);
        for rib in (3..w - 1).step_by(4) {
            c.rect(x + dx + rib, y + 3, 1, 45 - y, STEEL);
        }
        c.pixel(x + dx + 1, 47, GOLD);
        c.pixel(x + dx + w - 2, 47, GOLD);
    }
    c.rect(x + 2, 49, 34, 1, STEEL);
}

pub(super) fn autorack(c: &mut Canvas<'_>, x: i32) {
    // Two decks and four tiny cars. Open sides keep the structure readable.
    for y in [32, 41, 49] {
        c.rect(x, y, 38, 2, INK);
        c.rect(x + 1, y, 36, 1, GOLD);
    }
    for y in [35, 44] {
        for (dx, color) in [(4, GREEN), (21, RUST)] {
            c.rect(x + dx + 2, y, 7, 2, INK);
            c.rect(x + dx + 3, y, 5, 1, GLASS);
            c.rect(x + dx, y + 2, 13, 3, color);
            c.pixel(x + dx + 12, y + 2, CREAM);
            for wheel in [2, 10] {
                c.rect(x + dx + wheel, y + 4, 2, 2, INK);
            }
        }
    }
    for dx in [1, 18, 36] {
        c.rect(x + dx, 33, 1, 17, STEEL);
    }
}

pub(super) fn pressure_tank(c: &mut Canvas<'_>, x: i32) {
    // Narrow pressure vessel, rounded ends, saddles and a protected valve dome.
    for dx in [9, 27] {
        c.rect(x + dx - 2, 45, 5, 6, INK);
        c.rect(x + dx - 1, 46, 3, 4, RUST);
    }
    c.circle(x + 8, 42, 6, INK);
    c.circle(x + 29, 42, 6, INK);
    c.rect(x + 8, 36, 22, 13, INK);
    c.circle(x + 8, 42, 5, GLASS);
    c.circle(x + 29, 42, 5, GLASS);
    c.rect(x + 8, 37, 22, 11, GLASS);
    c.rect(x + 7, 38, 24, 2, CREAM);
    c.rect(x + 7, 45, 25, 2, STEEL);
    c.rect(x + 16, 32, 7, 4, INK);
    c.rect(x + 17, 33, 5, 2, GOLD);
    c.rect(x + 19, 34, 1, 3, RUST);
    for dx in [12, 25] {
        c.rect(x + dx, 37, 1, 11, INK);
    }
    // Neutral diamond marking; no real operator or hazardous cargo branding.
    c.polygon(
        &[(x + 19, 40), (x + 22, 43), (x + 19, 46), (x + 16, 43)],
        BLUE,
    );
    c.pixel(x + 19, 43, CREAM);
}

pub(super) fn crane(c: &mut Canvas<'_>, x: i32) {
    // Folded lattice boom stays inside the wagon; no animated rescue implied.
    c.rect(x, 48, 38, 3, INK);
    c.rect(x + 1, 49, 36, 1, GOLD);
    c.rect(x + 23, 42, 12, 6, RUST);
    c.rect(x + 25, 39, 9, 3, INK);
    c.rect(x + 26, 40, 7, 2, STEEL);
    c.rect(x + 5, 39, 13, 9, INK);
    c.rect(x + 6, 40, 11, 7, GOLD);
    c.rect(x + 6, 38, 11, 2, INK);
    c.rect(x + 7, 40, 5, 4, GLASS);
    c.rect(x + 14, 41, 2, 5, RUST);
    c.line((x + 12, 39), (x + 32, 32), INK);
    c.line((x + 12, 43), (x + 32, 36), INK);
    c.line((x + 12, 40), (x + 32, 33), GOLD);
    c.line((x + 12, 42), (x + 32, 35), GOLD);
    for (dx, y) in [(16, 38), (22, 36), (28, 34)] {
        c.line((x + dx, y), (x + dx + 3, y + 2), RUST);
    }
    c.rect(x + 32, 33, 1, 10, INK);
    c.rect(x + 31, 43, 3, 2, GOLD);
    c.pixel(x + 30, 42, GOLD);
    c.rect(x + 3, 45, 2, 4, STEEL);
    c.rect(x + 35, 45, 2, 4, STEEL);
}

pub(super) fn bicycle(c: &mut Canvas<'_>, x: i32) {
    // A short baggage cabin plus an open rack; bikes are cargo, not passengers.
    c.rect(x, 48, 38, 3, INK);
    c.rect(x, 37, 16, 13, INK);
    c.rect(x + 2, 35, 12, 2, INK);
    c.rect(x + 3, 35, 10, 1, CREAM);
    c.rect(x + 1, 38, 14, 11, GREEN);
    c.rect(x + 3, 39, 9, 10, INK);
    c.rect(x + 4, 40, 7, 8, STEEL);
    c.rect(x + 5, 40, 5, 3, GLASS);
    c.pixel(x + 10, 45, GOLD);
    for dx in [20, 32] {
        c.circle(x + dx, 45, 3, INK);
        c.circle(x + dx, 45, 2, STEEL);
        c.pixel(x + dx, 45, INK);
    }
    c.line((x + 20, 45), (x + 24, 39), GOLD);
    c.line((x + 24, 39), (x + 28, 45), GOLD);
    c.line((x + 20, 45), (x + 28, 45), GOLD);
    c.line((x + 24, 39), (x + 30, 39), GOLD);
    c.line((x + 30, 39), (x + 28, 45), GOLD);
    c.line((x + 29, 37), (x + 32, 45), RUST);
    c.rect(x + 28, 36, 4, 1, INK);
    c.rect(x + 22, 38, 4, 1, INK);
    c.rect(x + 36, 40, 1, 10, STEEL);
}

pub(super) fn coil(c: &mut Canvas<'_>, x: i32) {
    // Three exposed steel coils sit in individual low V-shaped cradles.
    c.rect(x, 48, 38, 3, INK);
    c.rect(x + 1, 49, 36, 1, RUST);
    for (dx, y, radius) in [(7, 43, 5), (19, 41, 7), (31, 43, 5)] {
        c.circle(x + dx, y, radius, INK);
        c.circle(x + dx, y, radius - 1, STEEL);
        c.circle(x + dx, y, radius - 2, CREAM);
        c.circle(x + dx, y, 2, INK);
        c.pixel(x + dx, y, GLASS);
        c.line((x + dx - 5, 46), (x + dx - 2, 49), GOLD);
        c.line((x + dx + 5, 46), (x + dx + 2, 49), GOLD);
    }
    c.rect(x + 1, 45, 1, 5, INK);
    c.rect(x + 36, 45, 1, 5, INK);
}

pub(super) fn fair(c: &mut Canvas<'_>, x: i32) {
    // Folded stalls and a striped roof identify a travelling equipment wagon.
    c.rect(x + 2, 39, 34, 12, INK);
    c.rect(x + 3, 40, 32, 10, GREEN);
    c.polygon(&[(x, 39), (x + 6, 34), (x + 31, 34), (x + 37, 39)], INK);
    for dx in [6, 14, 22, 30] {
        c.rect(x + dx, 35, 3, 4, CREAM);
    }
    c.rect(x + 1, 39, 36, 2, GOLD);
    c.rect(x + 19, 32, 1, 3, INK);
    c.rect(x + 20, 32, 5, 2, RUST);
    for dx in [5, 25] {
        c.rect(x + dx, 42, 8, 7, RUST);
        c.line((x + dx, 42), (x + dx + 7, 48), GOLD);
        c.line((x + dx + 7, 42), (x + dx, 48), GOLD);
    }
    c.rect(x + 15, 41, 7, 9, INK);
    c.rect(x + 16, 42, 5, 7, CREAM);
    c.rect(x + 19, 44, 1, 3, RUST);
}
