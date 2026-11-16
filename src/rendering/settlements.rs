//! Original fictional settlements, selected by route hints. No persistent RNG.
use super::{
    SegmentScenery,
    palette::{Palette, mix},
    scenery::View,
    sprites::Canvas,
    theme::{RenderQuality, Weather, finite_nonnegative},
};
use crate::simulation::catalog::SceneryTag as T;
use eframe::egui::Color32;

/// Route-owned seed and proximity. No settlement is inferred from CPU or time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SettlementContext {
    pub seed: u64,
    pub density: f32,
}

impl Default for SettlementContext {
    fn default() -> Self {
        Self {
            seed: 0,
            density: 1.0,
        }
    }
}

impl SettlementContext {
    /// Smooth arrival/departure envelope within the outer quarter of a segment.
    #[must_use]
    pub fn along_segment(seed: u64, progress: f64, departure: bool, arrival: bool) -> Self {
        let progress = finite_nonnegative(progress).min(1.0);
        let left = if departure {
            (1.0 - progress * 4.0).max(0.0)
        } else {
            0.0
        };
        let right = if arrival {
            ((progress - 0.75) * 4.0).max(0.0)
        } else {
            0.0
        };
        let t = left.max(right);
        Self {
            seed,
            density: (t * t * (3.0 - 2.0 * t)) as f32,
        }
    }
}

/// Each entry has its own silhouette/details; all are kind: fictional.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementElement {
    TimberHouse,
    MasonryHouse,
    Townhouse,
    BalconyHouse,
    HippedHouse,
    Shop,
    Bakery,
    Greenhouse,
    Church,
    Market,
    WaterTank,
    Bandstand,
    Fountain,
    Office,
    Apartments,
    Tower,
    Cinema,
    School,
    Library,
    Factory,
    SawtoothMill,
    Workshop,
    Warehouse,
    Silos,
    OilTank,
    BrickKiln,
    Gantry,
    HarborShed,
    Containers,
    Freighter,
    Beacon,
    Pier,
    Street,
    Wall,
    Football,
    Playground,
    Orchard,
    Laundry,
}

use SettlementElement as E;
impl SettlementElement {
    pub const ALL: [Self; 38] = [
        E::TimberHouse,
        E::MasonryHouse,
        E::Townhouse,
        E::BalconyHouse,
        E::HippedHouse,
        E::Shop,
        E::Bakery,
        E::Greenhouse,
        E::Church,
        E::Market,
        E::WaterTank,
        E::Bandstand,
        E::Fountain,
        E::Office,
        E::Apartments,
        E::Tower,
        E::Cinema,
        E::School,
        E::Library,
        E::Factory,
        E::SawtoothMill,
        E::Workshop,
        E::Warehouse,
        E::Silos,
        E::OilTank,
        E::BrickKiln,
        E::Gantry,
        E::HarborShed,
        E::Containers,
        E::Freighter,
        E::Beacon,
        E::Pier,
        E::Street,
        E::Wall,
        E::Football,
        E::Playground,
        E::Orchard,
        E::Laundry,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            E::TimberHouse => "Casa de madeira",
            E::MasonryHouse => "Casa de alvenaria",
            E::Townhouse => "Sobrado geminado",
            E::BalconyHouse => "Casa com varanda",
            E::HippedHouse => "Casa de quatro águas",
            E::Shop => "Armazém de esquina",
            E::Bakery => "Padaria",
            E::Greenhouse => "Estufa de quintal",
            E::Church => "Igreja com campanário",
            E::Market => "Mercado de toldos",
            E::WaterTank => "Caixa-d'água elevada",
            E::Bandstand => "Coreto",
            E::Fountain => "Praça com fonte",
            E::Office => "Edifício de três andares",
            E::Apartments => "Apartamentos de cinco andares",
            E::Tower => "Torre de oito andares",
            E::Cinema => "Cinema",
            E::School => "Escola",
            E::Library => "Biblioteca",
            E::Factory => "Fábrica de chaminé",
            E::SawtoothMill => "Fábrica de sheds",
            E::Workshop => "Oficina de bairro",
            E::Warehouse => "Armazém de carga",
            E::Silos => "Silos geminados",
            E::OilTank => "Reservatório industrial",
            E::BrickKiln => "Olaria",
            E::Gantry => "Pórtico de contêineres",
            E::HarborShed => "Galpão do cais",
            E::Containers => "Pilha de contêineres",
            E::Freighter => "Navio de dois mastros",
            E::Beacon => "Farol de cais",
            E::Pier => "Píer com cabeços",
            E::Street => "Rua com iluminação",
            E::Wall => "Muro com portão",
            E::Football => "Campinho",
            E::Playground => "Praça de brinquedos",
            E::Orchard => "Pomar cercado",
            E::Laundry => "Quintal com varal",
        }
    }
}

const VILLAGE: &[E] = &[
    E::TimberHouse,
    E::MasonryHouse,
    E::BalconyHouse,
    E::HippedHouse,
    E::Greenhouse,
    E::Church,
    E::Orchard,
    E::Laundry,
];
const TOWN: &[E] = &[
    E::Townhouse,
    E::Shop,
    E::Bakery,
    E::Church,
    E::Market,
    E::Bandstand,
    E::Fountain,
    E::Library,
];
const CITY: &[E] = &[
    E::Office,
    E::Apartments,
    E::Tower,
    E::Cinema,
    E::School,
    E::Library,
    E::Market,
    E::Fountain,
];
const INDUSTRY: &[E] = &[
    E::Factory,
    E::SawtoothMill,
    E::Workshop,
    E::Warehouse,
    E::Silos,
    E::OilTank,
    E::BrickKiln,
    E::WaterTank,
];
const PORT: &[E] = &[
    E::Gantry,
    E::HarborShed,
    E::Containers,
    E::Freighter,
    E::Beacon,
    E::Pier,
];
const SUBURB: &[E] = &[
    E::MasonryHouse,
    E::HippedHouse,
    E::School,
    E::Street,
    E::Wall,
    E::Football,
    E::Playground,
    E::Laundry,
];

fn hash(mut n: u64) -> u64 {
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}

pub(super) fn draw(c: &mut Canvas<'_>, p: &Palette, v: View, tags: SegmentScenery) {
    let density = if v.settlement.density.is_finite() {
        v.settlement.density.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if density == 0.0 {
        return;
    }
    let pools = [
        (T::Village, VILLAGE),
        (T::Town, TOWN),
        (T::City, CITY),
        (T::Industrial, INDUSTRY),
        (T::Port, PORT),
        (T::Suburb, SUBURB),
    ];
    let count = pools.iter().filter(|(tag, _)| tags.contains(*tag)).count();
    if count == 0 {
        return;
    }
    // 960 / each ratio divides 184320. Seed identities do not depend on tiles,
    // viewport width, render history or floating point distance beyond the wrap.
    for (layer, ratio, base) in [(0, 0.03125, 41), (1, 0.125, 47), (2, 0.25, 53)] {
        let shift =
            (finite_nonnegative(v.distance).rem_euclid(960.0 / ratio) * ratio).floor() as i32;
        let last = ((c.width() + 959) / 960 + 1) * 960;
        let far_palette = Palette {
            wall: mix(p.wall, p.far, 0.6),
            roof: mix(p.roof, p.far, 0.6),
            ink: p.far_shadow,
            trim: p.ridge,
            ..*p
        };
        for tile in (-960..=last).step_by(960) {
            for slot in 0..16 {
                let n = hash(v.settlement.seed ^ (slot as u64 + layer * 73));
                // Each lot has a stable threshold. Buildings grow from the
                // ground over a small proximity interval instead of flashing on.
                let threshold = (n % 65) as f32 / 100.0;
                let reveal = ((density - threshold) / 0.35).clamp(0.0, 1.0);
                if reveal == 0.0 {
                    continue;
                }
                let (_, pool) = pools
                    .iter()
                    .filter(|(tag, _)| tags.contains(*tag))
                    .nth((slot as usize + layer as usize) % count)
                    .expect("nonempty pools");
                let element = pool[((n >> 8) as usize) % pool.len()];
                // Tall skyline silhouettes live behind the smaller street lots.
                let element = if layer == 0 && tags.contains(T::City) {
                    CITY[slot as usize % 3]
                } else {
                    element
                };
                if layer == 2
                    && matches!(element, E::Tower | E::Apartments | E::Freighter | E::Gantry)
                {
                    continue;
                }
                let x = tile + slot * 60 + (n % 9) as i32 - shift;
                // Include the farthest chimney puff at the maximum wind hint.
                if x + 64 < 0 || x - 4 >= c.width() {
                    continue;
                }
                let palette = if layer == 0 { &far_palette } else { p };
                // Clip the reveal to its own lot, keeping rail rows untouched.
                draw_element(c, palette, v, element, x, base, n, reveal);
            }
        }
    }
}

fn phase(v: View) -> i32 {
    if v.scenery_life.reduce_motion || v.quality == RenderQuality::Low || !v.options.details {
        0
    } else {
        (finite_nonnegative(v.seconds).rem_euclid(12.0)) as i32
    }
}

fn smoke(c: &mut Canvas<'_>, p: &Palette, v: View, x: i32, y: i32) {
    if v.quality == RenderQuality::Low || !v.options.details {
        return;
    }
    let wind = v.wind_speed_kmh.unwrap_or(12.0).clamp(0.0, 250.0) / 50.0;
    for i in 0..3 {
        let rise = (phase(v) + i * 4) % 12;
        c.rect(
            x + ((rise as f32) * wind / 3.0) as i32,
            y - rise,
            2 + rise / 5,
            2,
            mix(p.cloud_shadow, p.sky[2], 0.4),
        );
    }
}

fn roof(c: &mut Canvas<'_>, p: &Palette, x: i32, y: i32, w: i32, hip: bool) {
    let points = if hip {
        [
            (x - 2, y),
            (x + 4, y - 5),
            (x + w - 5, y - 5),
            (x + w + 2, y),
        ]
    } else {
        [
            (x - 2, y),
            (x + w / 2, y - 7),
            (x + w / 2 + 1, y - 7),
            (x + w + 2, y),
        ]
    };
    c.polygon(&points, p.roof);
    c.line((x - 1, y), (x + w + 1, y), p.ink);
}

fn windows(c: &mut Canvas<'_>, p: &Palette, v: View, area: (i32, i32, i32, i32), seed: u64) {
    let (x, y, w, h) = area;
    let lit = super::landscape_life::window_light(p, v);
    for row in (3..h - 2).step_by(4) {
        for col in (3..w - 2).step_by(5) {
            let color = if hash(seed ^ (row * 31 + col) as u64).is_multiple_of(5) {
                p.ink
            } else {
                lit
            };
            c.rect(x + col, y + row, 2, 2, color);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_element(
    c: &mut Canvas<'_>,
    p: &Palette,
    v: View,
    e: E,
    x: i32,
    base: i32,
    seed: u64,
    reveal: f32,
) {
    // Clip only this lot's reveal; never erase previously painted scenery.
    let top = base + 2 - (54.0 * reveal) as i32;
    let old = c.clip_rows(top, (base + 2).min(54));
    paint(c, p, v, e, x, base, seed);
    c.clip_rows(old.0, old.1);
}

fn paint(c: &mut Canvas<'_>, p: &Palette, v: View, e: E, x: i32, b: i32, seed: u64) {
    let wall = mix(p.wall, p.trim, ((seed >> 16) % 4) as f32 * 0.13);
    let glass = super::landscape_life::window_light(p, v);
    let wet = matches!(v.options.weather, Weather::Rain | Weather::Thunderstorm);
    match e {
        E::TimberHouse
        | E::MasonryHouse
        | E::Townhouse
        | E::BalconyHouse
        | E::HippedHouse
        | E::Shop
        | E::Bakery => {
            let h = if e == E::Townhouse { 23 } else { 13 };
            let w = if e == E::BalconyHouse { 30 } else { 24 };
            c.rect(x, b - h, w, h, wall);
            roof(c, p, x, b - h, w, e == E::HippedHouse);
            windows(c, p, v, (x, b - h, w, h), seed);
            c.rect(x + 10, b - 7, 4, 7, p.ink);
            if e == E::TimberHouse {
                for y in (b - h + 2..b).step_by(3) {
                    c.rect(x, y, w, 1, p.trim);
                }
                c.rect(x + 3, b - 9, 4, 4, glass);
            }
            if e == E::MasonryHouse {
                for y in (b - h + 2..b).step_by(4) {
                    for dx in (1..w).step_by(6) {
                        c.pixel(x + dx + (y % 2), y, p.roof);
                    }
                }
            }
            if e == E::Townhouse {
                c.rect(x + 12, b - h, 1, h, p.roof);
                c.rect(x - 1, b - 12, w + 2, 2, p.trim);
            }
            if e == E::BalconyHouse {
                c.rect(x - 2, b - 7, w + 4, 2, p.roof);
                for dx in [0, 8, 21, 29] {
                    c.rect(x + dx, b - 6, 1, 6, p.trim);
                }
            }
            if matches!(e, E::Shop | E::Bakery) {
                c.rect(x - 2, b - 10, w + 4, 3, p.roof);
                for dx in (0..w).step_by(6) {
                    c.rect(x + dx, b - 10, 3, 3, p.wall);
                }
                c.rect(x + 2, b - 6, 6, 4, glass);
            }
            if e == E::Bakery {
                c.rect(x + 18, b - h - 9, 3, 8, p.roof);
                smoke(c, p, v, x + 19, b - h - 10);
                c.rect(x + 3, b - 4, 4, 1, p.sun);
            }
        }
        E::Office | E::Apartments | E::Tower => {
            let (floors, w) = match e {
                E::Office => (3, 30),
                E::Apartments => (5, 25),
                _ => (8, 21),
            };
            let h = floors * 4 + 4 + (seed % 2) as i32;
            c.rect(x, b - h, w, h, wall);
            c.rect(x + w - 4, b - h, 4, h, p.trim);
            c.rect(x - 1, b - h - 2, w + 2, 2, p.ink);
            windows(c, p, v, (x, b - h, w, h), seed);
            c.rect(x + w / 2, b - 5, 4, 5, p.ink);
            if e == E::Apartments {
                for y in (b - h + 6..b - 3).step_by(4) {
                    c.rect(x - 2, y, w + 3, 1, p.roof);
                }
            }
            if e == E::Tower {
                c.rect(x + 5, b - h - 5, 10, 3, p.roof);
                c.line((x + 10, b - h - 9), (x + 10, b - h - 5), p.ink);
            }
        }
        E::Church => {
            c.rect(x + 10, b - 15, 25, 15, wall);
            roof(c, p, x + 10, b - 15, 25, false);
            c.rect(x, b - 30, 11, 30, p.wall);
            roof(c, p, x, b - 30, 11, false);
            c.rect(x + 4, b - 38, 1, 6, p.ink);
            c.rect(x + 2, b - 36, 5, 1, p.ink);
            c.rect(x + 3, b - 26, 5, 7, p.ink);
            c.rect(x + 4, b - 23, 3, 2, p.sun);
            c.circle(x + 23, b - 13, 3, glass);
            c.rect(x + 20, b - 7, 6, 7, p.roof);
        }
        E::Market => {
            c.rect(x, b - 12, 39, 12, wall);
            for dx in (0..39).step_by(13) {
                roof(c, p, x + dx, b - 12, 11, false);
                c.rect(x + dx + 2, b - 7, 8, 6, p.ink);
                for j in 0..3 {
                    c.rect(
                        x + dx + 2 + j * 3,
                        b - 4,
                        2,
                        2,
                        if j == 1 { p.sun } else { p.foliage_light },
                    );
                }
            }
        }
        E::WaterTank => {
            for dx in [5, 20] {
                c.rect(x + dx, b - 20, 2, 20, p.ink);
            }
            c.line((x + 6, b - 17), (x + 21, b - 2), p.trim);
            c.line((x + 21, b - 17), (x + 6, b - 2), p.trim);
            c.rect(x, b - 30, 28, 12, wall);
            c.rect(x - 1, b - 31, 30, 2, p.roof);
            for y in [b - 27, b - 21] {
                c.rect(x, y, 28, 1, p.ink);
            }
        }
        E::Bandstand => {
            c.rect(x, b - 2, 34, 2, p.trim);
            roof(c, p, x, b - 18, 34, true);
            for dx in [2, 10, 23, 31] {
                c.rect(x + dx, b - 18, 1, 16, p.wall);
            }
            c.rect(x + 1, b - 6, 32, 1, p.roof);
            c.rect(x + 8, b - 1, 18, 2, p.wall);
        }
        E::Fountain => {
            c.rect(x, b - 3, 35, 3, p.trim);
            c.rect(x + 2, b - 4, 31, 1, p.water);
            c.rect(x + 16, b - 14, 3, 11, p.wall);
            c.rect(x + 8, b - 13, 19, 2, p.wall);
            for dx in [7, 26] {
                c.line((x + 17, b - 18), (x + dx, b - 7), p.water_light);
            }
            c.rect(x - 2, b - 1, 3, 2, p.foliage);
        }
        E::Cinema | E::School | E::Library => {
            c.rect(x, b - 19, 35, 19, wall);
            c.rect(x - 2, b - 20, 39, 2, p.roof);
            windows(c, p, v, (x, b - 19, 35, 19), seed);
            c.rect(x + 14, b - 8, 7, 8, p.ink);
            match e {
                E::Cinema => {
                    c.rect(x + 4, b - 25, 27, 7, p.roof);
                    c.rect(x + 7, b - 23, 21, 2, p.sun);
                    c.rect(x - 2, b - 10, 39, 3, p.trim);
                }
                E::School => {
                    roof(c, p, x + 10, b - 20, 15, false);
                    c.circle(x + 17, b - 19, 2, p.wall);
                    c.line((x + 37, b - 25), (x + 37, b), p.ink);
                    c.rect(x + 38, b - 25, 6, 4, p.roof);
                }
                _ => {
                    roof(c, p, x, b - 20, 35, false);
                    for dx in [3, 10, 24, 31] {
                        c.rect(x + dx, b - 15, 2, 15, p.trim);
                    }
                }
            }
        }
        E::Factory | E::SawtoothMill | E::Workshop | E::Warehouse | E::HarborShed => {
            c.rect(x, b - 16, 39, 16, wall);
            windows(c, p, v, (x, b - 16, 39, 10), seed);
            c.rect(x + 13, b - 9, 12, 9, p.ink);
            if e == E::SawtoothMill {
                for dx in (0..39).step_by(13) {
                    c.polygon(
                        &[
                            (x + dx, b - 16),
                            (x + dx + 12, b - 24),
                            (x + dx + 12, b - 16),
                        ],
                        p.roof,
                    );
                    c.line((x + dx + 11, b - 22), (x + dx + 11, b - 17), glass);
                }
            } else {
                roof(c, p, x, b - 16, 39, e == E::Workshop);
            }
            if e == E::Factory {
                c.rect(x + 32, b - 38, 5, 23, p.roof);
                c.rect(x + 31, b - 39, 7, 2, p.trim);
                smoke(c, p, v, x + 33, b - 40);
            }
            if e == E::Workshop {
                c.circle(x + 6, b - 8, 3, p.roof);
                c.circle(x + 6, b - 8, 1, p.wall);
            }
            if e == E::Warehouse {
                for dx in [2, 29] {
                    c.rect(x + dx, b - 5, 7, 5, p.roof);
                    c.line((x + dx, b - 5), (x + dx + 6, b - 1), p.trim);
                }
            }
            if e == E::HarborShed {
                c.rect(x - 1, b - 1, 43, 2, p.ink);
                c.line((x + 1, b - 13), (x + 38, b - 13), p.trim);
                c.rect(x + 15, b - 9, 1, 9, p.trim);
            }
        }
        E::Silos => {
            for dx in [0, 14, 28] {
                c.rect(x + dx, b - 25, 12, 25, wall);
                roof(c, p, x + dx, b - 25, 12, false);
                for y in (b - 22..b).step_by(5) {
                    c.rect(x + dx, y, 12, 1, p.trim);
                }
            }
            c.line((x + 5, b - 34), (x + 35, b - 34), p.ink);
            c.line((x + 20, b - 34), (x + 20, b), p.ink);
        }
        E::OilTank => {
            c.circle(x + 18, b - 16, 16, wall);
            c.rect(x + 2, b - 16, 33, 16, wall);
            c.rect(x + 2, b - 3, 33, 2, p.trim);
            c.rect(x + 31, b - 22, 1, 22, p.ink);
            for y in (b - 20..b).step_by(3) {
                c.rect(x + 29, y, 4, 1, p.ink);
            }
        }
        E::BrickKiln => {
            c.circle(x + 16, b - 9, 13, p.roof);
            c.rect(x + 3, b - 9, 27, 9, p.roof);
            c.rect(x + 11, b - 8, 9, 8, p.ink);
            c.rect(x + 13, b - 5, 5, 5, p.sun);
            c.rect(x + 28, b - 30, 4, 30, p.trim);
            smoke(c, p, v, x + 29, b - 31);
        }
        E::Gantry => {
            for dx in [1, 34] {
                c.rect(x + dx, b - 32, 3, 32, p.roof);
                c.rect(x + dx - 2, b - 2, 7, 2, p.ink);
            }
            c.rect(x, b - 34, 39, 4, p.trim);
            c.line((x + 3, b - 31), (x + 18, b - 20), p.roof);
            let d = phase(v) / 4;
            c.rect(x + 20 + d, b - 31, 5, 3, p.ink);
            c.line((x + 22 + d, b - 28), (x + 22 + d, b - 14), p.ink);
            c.rect(x + 16 + d, b - 14, 14, 6, p.roof);
        }
        E::Containers => {
            for (dx, dy, color) in [(0, 0, p.roof), (22, 0, p.trim), (9, -8, p.foliage)] {
                c.rect(x + dx, b - 8 + dy, 21, 8, color);
                c.rect(x + dx, b - 8 + dy, 21, 1, p.ink);
                for rib in (3..20).step_by(4) {
                    c.rect(x + dx + rib, b - 6 + dy, 1, 5, p.wall);
                }
            }
        }
        E::Freighter => {
            c.polygon(
                &[
                    (x, b - 10),
                    (x + 45, b - 10),
                    (x + 39, b - 2),
                    (x + 6, b - 2),
                ],
                p.ink,
            );
            c.rect(x + 3, b - 12, 39, 3, p.roof);
            c.rect(x + 5, b - 23, 10, 11, wall);
            c.rect(x + 7, b - 21, 6, 2, glass);
            for dx in [20, 34] {
                c.line((x + dx, b - 29), (x + dx, b - 12), p.ink);
                c.line((x + dx, b - 28), (x + dx + 8, b - 18), p.trim);
            }
            c.rect(x + 19, b - 16, 18, 4, p.trim);
            c.rect(x + 5, b, 33, 1, p.water_light);
        }
        E::Beacon => {
            c.polygon(
                &[(x + 4, b), (x + 8, b - 32), (x + 17, b - 32), (x + 21, b)],
                p.wall,
            );
            for y in [b - 10, b - 22] {
                c.rect(x + 7, y, 12, 4, p.roof);
            }
            c.rect(x + 5, b - 36, 16, 4, p.ink);
            c.rect(x + 8, b - 35, 10, 2, glass);
            roof(c, p, x + 5, b - 36, 16, false);
            c.rect(x + 11, b - 6, 4, 6, p.ink);
        }
        E::Pier => {
            c.rect(x, b - 7, 43, 3, p.trim);
            for dx in [2, 14, 27, 39] {
                c.rect(x + dx, b - 4, 2, 5, p.ink);
                c.rect(x + dx - 1, b - 10, 4, 3, p.ink);
            }
            c.line((x + 3, b - 9), (x + 15, b - 8), p.roof);
        }
        E::Greenhouse => {
            c.rect(x, b - 15, 31, 15, p.water);
            c.polygon(
                &[(x - 1, b - 15), (x + 15, b - 24), (x + 32, b - 15)],
                p.water_light,
            );
            for dx in [0, 10, 20, 30] {
                c.line((x + dx, b - 15), (x + dx, b), p.wall);
            }
            c.line((x, b - 15), (x + 31, b - 15), p.wall);
            c.rect(x, b - 1, 31, 1, p.ink);
            for dx in [3, 13, 23] {
                c.rect(x + dx, b - 5, 5, 4, p.foliage);
            }
        }
        E::Street => {
            c.polygon(
                &[(x + 11, b - 17), (x + 23, b - 17), (x + 39, b), (x, b)],
                p.ridge_shadow,
            );
            c.line((x + 17, b - 13), (x + 17, b - 9), p.trim);
            c.line((x + 18, b - 5), (x + 20, b - 1), p.trim);
            for dx in [1, 35] {
                c.rect(x + dx, b - 24, 1, 24, p.ink);
                c.rect(x + dx - 2, b - 25, 5, 2, glass);
            }
            if wet {
                c.rect(x + 6, b - 3, 9, 1, p.water_light);
            }
        }
        E::Wall => {
            c.rect(x, b - 9, 42, 9, p.roof);
            for y in (b - 8..b).step_by(3) {
                c.rect(x, y, 42, 1, p.trim);
            }
            c.rect(x + 15, b - 13, 13, 13, p.ink);
            for dx in (17..28).step_by(3) {
                c.rect(x + dx, b - 12, 1, 11, p.wall);
            }
            c.rect(x - 1, b - 10, 44, 1, p.wall);
        }
        E::Football => {
            c.rect(x, b - 16, 43, 16, p.foliage);
            c.line((x + 1, b - 15), (x + 41, b - 15), p.wall);
            c.rect(x + 21, b - 15, 1, 14, p.wall);
            c.circle(x + 21, b - 7, 4, p.wall);
            c.circle(x + 21, b - 7, 3, p.foliage);
            for dx in [0, 37] {
                c.rect(x + dx, b - 12, 6, 1, p.wall);
                c.rect(x + dx, b - 12, 1, 9, p.wall);
                c.rect(x + dx + 5, b - 12, 1, 9, p.wall);
            }
        }
        E::Playground => {
            c.line((x, b), (x + 7, b - 20), p.roof);
            c.line((x + 14, b), (x + 7, b - 20), p.roof);
            c.rect(x + 7, b - 20, 20, 2, p.trim);
            c.line((x + 27, b - 20), (x + 33, b), p.roof);
            let d = phase(v) / 4 - 1;
            for dx in [14, 22] {
                c.line((x + dx, b - 18), (x + dx + d, b - 6), p.ink);
            }
            c.rect(x + 13 + d, b - 6, 11, 2, p.roof);
        }
        E::Orchard => {
            for dx in [4, 17, 31] {
                c.rect(x + dx, b - 16, 2, 16, p.roof);
                c.circle(x + dx, b - 18, 7, p.foliage);
                c.rect(x + dx - 4, b - 22, 6, 2, p.foliage_light);
                c.pixel(x + dx - 2, b - 18, p.sun);
            }
            c.rect(x - 1, b - 5, 42, 1, p.trim);
            for dx in (0..42).step_by(6) {
                c.rect(x + dx, b - 7, 1, 7, p.wall);
            }
        }
        E::Laundry => {
            for dx in [0, 37] {
                c.rect(x + dx, b - 18, 1, 18, p.roof);
            }
            c.line((x, b - 17), (x + 37, b - 17), p.ink);
            let d = if wet { 0 } else { phase(v) / 6 };
            for (dx, color) in [(4, p.wall), (14, p.roof), (25, p.water_light)] {
                c.rect(x + dx, b - 16, 7, 4, color);
                c.rect(x + dx + d, b - 12, 6, 4, color);
            }
            c.rect(x + 26, b - 4, 9, 4, p.trim);
        }
    }
    if v.options.weather == Weather::Snow
        && !matches!(
            e,
            E::Street | E::Football | E::Pier | E::Laundry | E::Fountain
        )
    {
        c.rect(x + 4, b - 2, 24, 1, p.cloud);
    }
    c.rect(x, b, 2, 1, p.ink);
}

/// Export-only isolated swatch using exactly the same element painter.
pub fn draw_settlement_preview(
    pixels: &mut [Color32],
    element: E,
    seed: u64,
    light: f64,
    weather: Weather,
) {
    let p = Palette::at_time(super::Biome::GreenValley, light, true);
    pixels.fill(p.sky[2]);
    let mut c = Canvas::new(pixels);
    let v = View {
        settlement: SettlementContext::default(),
        scenery_life: super::SceneryLifeOptions::default(),
        wind_speed_kmh: None,
        precipitation_mm: None,
        distance: 0.0,
        seconds: 8.0,
        light_seconds: light,
        options: super::SceneOptions {
            weather,
            ..Default::default()
        },
        quality: RenderQuality::Full,
    };
    c.rect(0, 54, c.width(), 16, p.ground);
    draw_element(&mut c, &p, v, element, 10, 53, seed, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoracao_congela_sem_perder_resposta_ao_clima() {
        let p = Palette::for_biome(super::super::Biome::GreenValley);
        let v = View {
            settlement: Default::default(),
            scenery_life: Default::default(),
            wind_speed_kmh: Some(250.0),
            precipitation_mm: None,
            distance: 0.0,
            seconds: 0.0,
            light_seconds: 180.0,
            options: super::super::SceneOptions::default(),
            quality: RenderQuality::Full,
        };
        let frame = |element, view| {
            let mut pixels = vec![Color32::TRANSPARENT; 64 * 70];
            paint(&mut Canvas::new(&mut pixels), &p, view, element, 4, 53, 42);
            pixels
        };
        for element in [
            E::Bakery,
            E::Factory,
            E::BrickKiln,
            E::Gantry,
            E::Playground,
            E::Laundry,
        ] {
            assert_ne!(
                frame(element, v),
                frame(element, View { seconds: 7.0, ..v }),
                "{element:?}"
            );
            let frozen = View {
                scenery_life: super::super::SceneryLifeOptions {
                    reduce_motion: true,
                },
                ..v
            };
            for seconds in [1.0, 7.0, 64.0] {
                assert_eq!(
                    frame(element, frozen),
                    frame(element, View { seconds, ..frozen })
                );
            }
            let low = View {
                quality: RenderQuality::Low,
                ..v
            };
            assert_eq!(
                frame(element, low),
                frame(
                    element,
                    View {
                        seconds: 7.0,
                        ..low
                    }
                )
            );
        }
        for (element, weather) in [(E::Street, Weather::Rain), (E::TimberHouse, Weather::Snow)] {
            assert_ne!(
                frame(element, v),
                frame(
                    element,
                    View {
                        options: super::super::SceneOptions {
                            weather,
                            ..v.options
                        },
                        ..v
                    }
                )
            );
        }
        assert_ne!(
            frame(E::Factory, v),
            frame(
                E::Factory,
                View {
                    wind_speed_kmh: Some(0.0),
                    ..v
                }
            )
        );
    }

    #[test]
    fn camadas_preservam_wrap_recorte_e_trilhos() {
        let tags = SegmentScenery::from_tags(&[
            T::Village,
            T::Town,
            T::City,
            T::Industrial,
            T::Port,
            T::Suburb,
        ]);
        let p = Palette::for_biome(super::super::Biome::GreenValley);
        let view = View {
            settlement: SettlementContext {
                seed: 2026,
                density: 1.0,
            },
            scenery_life: super::super::SceneryLifeOptions::default(),
            wind_speed_kmh: Some(250.0),
            precipitation_mm: None,
            distance: 0.0,
            seconds: 11.0,
            light_seconds: 180.0,
            options: super::super::SceneOptions::default(),
            quality: RenderQuality::Full,
        };
        for distance in [0.0, 3839.9, 7679.9, 30719.9, 184319.9] {
            let mut reference = vec![Color32::TRANSPARENT; 1920 * 70];
            draw(
                &mut Canvas::new(&mut reference),
                &p,
                View { distance, ..view },
                tags,
            );
            for width in [160, 320, 641, 1920] {
                let mut pixels = vec![Color32::TRANSPARENT; width * 70];
                draw(
                    &mut Canvas::new(&mut pixels),
                    &p,
                    View { distance, ..view },
                    tags,
                );
                for y in 0..70 {
                    assert_eq!(
                        &reference[y * 1920..y * 1920 + width],
                        &pixels[y * width..(y + 1) * width],
                        "distance={distance}, width={width}"
                    );
                }
                assert!(
                    pixels[54 * width..]
                        .iter()
                        .all(|pixel| *pixel == Color32::TRANSPARENT)
                );
                let before = pixels.clone();
                pixels.fill(Color32::TRANSPARENT);
                draw(
                    &mut Canvas::new(&mut pixels),
                    &p,
                    View {
                        distance: distance + crate::simulation::SCROLL_PERIOD_PX,
                        ..view
                    },
                    tags,
                );
                assert_eq!(before, pixels);
            }
        }
    }
}
