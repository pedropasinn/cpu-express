//! Original fictional architecture, independent of station ordinal and biome.
use super::{palette::Palette, sprites::Canvas};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationStyle {
    PortTerminal,
    MountainStone,
    TimberHalt,
    Plateau,
    UrbanTerminal,
    ServicePost,
}

impl StationStyle {
    pub const ALL: [Self; 6] = [
        Self::PortTerminal,
        Self::MountainStone,
        Self::TimberHalt,
        Self::Plateau,
        Self::UrbanTerminal,
        Self::ServicePost,
    ];
    pub const KIND: &'static str = "fictional";
    pub const fn id(self) -> &'static str {
        match self {
            Self::PortTerminal => "port_terminal",
            Self::MountainStone => "mountain_stone",
            Self::TimberHalt => "timber_halt",
            Self::Plateau => "plateau",
            Self::UrbanTerminal => "urban_terminal",
            Self::ServicePost => "service_post",
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::PortTerminal => "Terminal portuário",
            Self::MountainStone => "Estação de serra em pedra",
            Self::TimberHalt => "Parada de madeira",
            Self::Plateau => "Estação de planalto",
            Self::UrbanTerminal => "Terminal urbano",
            Self::ServicePost => "Posto de serviço",
        }
    }
}

/// Explicit route-adapter input. None preserves every legacy Aurora pixel.
/// Reset on line changes; branch landmarks deliberately ignore this override.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StationStyles {
    pub departing: Option<StationStyle>,
    pub arriving: Option<StationStyle>,
}

pub(super) fn draw(
    canvas: &mut Canvas<'_>,
    x: i32,
    style: StationStyle,
    p: &Palette,
    night: bool,
    detailed: bool,
) {
    let glass = if night { p.sun } else { p.water_light };
    canvas.rect(x - 7, 54, 79, 3, p.ink);
    canvas.rect(x - 7, 54, 79, 1, p.trim);
    match style {
        StationStyle::PortTerminal => {
            // Long goods hall, sawtooth roof and a compact customs tower.
            canvas.rect(x - 3, 33, 65, 21, p.ink);
            canvas.rect(x - 2, 34, 63, 19, p.wall);
            for dx in [-3, 17, 37] {
                canvas.polygon(&[(x + dx, 33), (x + dx + 15, 24), (x + dx + 20, 33)], p.ink);
                canvas.polygon(
                    &[(x + dx + 2, 32), (x + dx + 14, 26), (x + dx + 17, 32)],
                    p.roof,
                );
                canvas.rect(x + dx + 13, 29, 3, 3, glass);
                canvas.rect(x + dx + 5, 39, 12, 14, p.ink);
                canvas.rect(x + dx + 6, 40, 10, 12, p.roof);
                if detailed {
                    canvas.line((x + dx + 7, 41), (x + dx + 14, 51), p.trim);
                }
            }
            canvas.rect(x + 53, 19, 12, 35, p.ink);
            canvas.rect(x + 54, 20, 10, 33, p.wall);
            canvas.rect(x + 56, 24, 6, 6, glass);
            canvas.rect(x + 51, 17, 16, 3, p.roof);
            canvas.rect(x - 4, 35, 68, 2, p.trim);
        }
        StationStyle::MountainStone => {
            // Heavy stone base, steep slate gable and buttresses.
            canvas.rect(x + 2, 31, 47, 23, p.ink);
            canvas.rect(x + 3, 32, 45, 21, p.trim);
            canvas.polygon(&[(x - 3, 33), (x + 25, 12), (x + 54, 33)], p.ink);
            canvas.polygon(&[(x, 31), (x + 25, 15), (x + 50, 31)], p.roof);
            for dx in [4, 42] {
                canvas.rect(x + dx, 34, 4, 20, p.wall);
            }
            canvas.rect(x + 20, 38, 12, 16, p.ink);
            canvas.rect(x + 22, 40, 8, 13, p.roof);
            for dx in [10, 34] {
                canvas.rect(x + dx, 35, 6, 10, p.ink);
                canvas.rect(x + dx + 1, 36, 4, 7, glass);
            }
            if detailed {
                for y in [34, 40, 46, 52] {
                    for dx in [8, 30] {
                        canvas.rect(x + dx, y, 10, 1, p.wall);
                    }
                }
            }
            canvas.rect(x + 39, 16, 5, 13, p.ink);
            canvas.rect(x + 40, 17, 3, 9, p.trim);
        }
        StationStyle::TimberHalt => {
            // Open shelter: an unmistakable gap, slatted back and long bench.
            canvas.rect(x + 2, 34, 42, 20, p.ink);
            canvas.rect(x + 3, 35, 40, 18, p.roof);
            if detailed {
                for y in (36..53).step_by(4) {
                    canvas.rect(x + 3, y, 40, 1, p.trim);
                }
            }
            canvas.polygon(
                &[(x - 5, 34), (x + 7, 26), (x + 47, 26), (x + 53, 34)],
                p.ink,
            );
            canvas.rect(x - 3, 33, 54, 2, p.trim);
            canvas.rect(x + 11, 36, 24, 14, p.ink);
            canvas.rect(x + 13, 48, 20, 2, p.wall);
            for dx in [14, 31] {
                canvas.rect(x + dx, 50, 1, 4, p.trim);
            }
            for dx in [-1, 48] {
                canvas.rect(x + dx, 35, 2, 19, p.ink);
            }
            canvas.rect(x + 56, 36, 2, 18, p.ink);
            canvas.rect(x + 51, 36, 12, 5, p.wall);
        }
        StationStyle::Plateau => {
            // Broad hipped roof, veranda and a low clock dormer.
            canvas.rect(x, 34, 59, 20, p.ink);
            canvas.rect(x + 1, 35, 57, 18, p.wall);
            canvas.polygon(
                &[(x - 5, 34), (x + 7, 22), (x + 48, 22), (x + 64, 34)],
                p.ink,
            );
            canvas.polygon(
                &[(x - 2, 32), (x + 8, 24), (x + 47, 24), (x + 60, 32)],
                p.roof,
            );
            canvas.rect(x + 22, 18, 13, 14, p.ink);
            canvas.rect(x + 23, 19, 11, 12, p.wall);
            canvas.circle(x + 28, 25, 4, p.roof);
            canvas.circle(x + 28, 25, 3, p.trim);
            canvas.line((x + 28, 22), (x + 28, 25), p.ink);
            canvas.pixel(x + 30, 25, p.ink);
            for dx in [5, 17, 39, 51] {
                canvas.rect(x + dx, 38, 5, 8, p.ink);
                canvas.rect(x + dx + 1, 39, 3, 5, glass);
            }
            canvas.rect(x + 27, 40, 7, 14, p.roof);
            canvas.rect(x - 4, 36, 67, 2, p.trim);
            for dx in [-2, 23, 61] {
                canvas.rect(x + dx, 38, 1, 16, p.ink);
            }
        }
        StationStyle::UrbanTerminal => {
            // Two wings around a tall glazed concourse and clock tower.
            canvas.rect(x - 5, 28, 75, 26, p.ink);
            canvas.rect(x - 4, 29, 73, 24, p.wall);
            canvas.rect(x - 6, 26, 77, 3, p.roof);
            canvas.rect(x + 23, 13, 20, 41, p.ink);
            canvas.rect(x + 24, 14, 18, 39, p.trim);
            canvas.rect(x + 21, 11, 24, 3, p.roof);
            canvas.circle(x + 33, 20, 5, p.ink);
            canvas.circle(x + 33, 20, 4, p.wall);
            canvas.line((x + 33, 17), (x + 33, 20), p.ink);
            canvas.pixel(x + 35, 20, p.ink);
            canvas.rect(x + 26, 29, 14, 25, p.ink);
            canvas.rect(x + 27, 30, 12, 22, glass);
            canvas.rect(x + 32, 30, 2, 22, p.roof);
            for dx in [-1, 10, 48, 59] {
                for y in [32, 44] {
                    canvas.rect(x + dx, y, 7, 8, p.ink);
                    canvas.rect(x + dx + 1, y + 1, 5, 6, glass);
                }
            }
            if detailed {
                canvas.rect(x - 4, 41, 27, 2, p.trim);
                canvas.rect(x + 43, 41, 26, 2, p.trim);
            }
        }
        StationStyle::ServicePost => {
            // Small cabin, water tank and trackside equipment, without a hall.
            canvas.rect(x + 3, 37, 24, 17, p.ink);
            canvas.rect(x + 4, 38, 22, 15, p.wall);
            canvas.polygon(&[(x, 38), (x + 6, 31), (x + 24, 31), (x + 30, 38)], p.roof);
            canvas.rect(x + 7, 41, 7, 7, p.ink);
            canvas.rect(x + 8, 42, 5, 5, glass);
            canvas.rect(x + 19, 42, 5, 12, p.roof);
            for dx in [41, 54] {
                canvas.rect(x + dx, 33, 2, 21, p.ink);
            }
            canvas.rect(x + 38, 23, 20, 12, p.ink);
            canvas.rect(x + 39, 24, 18, 9, p.water);
            canvas.rect(x + 38, 27, 20, 1, p.trim);
            canvas.rect(x + 38, 32, 20, 1, p.trim);
            canvas.line((x + 42, 35), (x + 55, 52), p.roof);
            canvas.line((x + 55, 35), (x + 42, 52), p.roof);
            if detailed {
                canvas.rect(x + 30, 49, 6, 5, p.roof);
                canvas.rect(x + 29, 48, 8, 1, p.trim);
            }
        }
    }
}
