//! Authored palettes, mixed only at logical-pixel resolution. No texture filter.

use eframe::egui::Color32;

use super::{
    sprites::rgb,
    theme::{Biome, DAY_CYCLE_SECONDS, finite_nonnegative},
};

/// Color roles shared by the four different sets of landscape geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub sky: [Color32; 4],
    pub sun: Color32,
    pub cloud: Color32,
    pub cloud_shadow: Color32,
    pub far: Color32,
    pub far_shadow: Color32,
    pub ridge: Color32,
    pub ridge_shadow: Color32,
    pub ground: Color32,
    pub ground_light: Color32,
    pub foliage: Color32,
    pub foliage_light: Color32,
    pub water: Color32,
    pub water_light: Color32,
    pub wall: Color32,
    pub roof: Color32,
    pub trim: Color32,
    pub ink: Color32,
    pub foreground: Color32,
    pub foreground_shadow: Color32,
}

impl Palette {
    #[must_use]
    pub const fn for_biome(biome: Biome) -> Self {
        match biome {
            Biome::GreenValley => Self {
                sky: [
                    rgb(168, 199, 196),
                    rgb(193, 214, 198),
                    rgb(226, 226, 190),
                    rgb(243, 228, 182),
                ],
                sun: rgb(255, 239, 184),
                cloud: rgb(248, 239, 211),
                cloud_shadow: rgb(198, 211, 196),
                far: rgb(137, 174, 168),
                far_shadow: rgb(114, 151, 153),
                ridge: rgb(95, 145, 128),
                ridge_shadow: rgb(74, 121, 112),
                ground: rgb(153, 176, 111),
                ground_light: rgb(194, 199, 135),
                foliage: rgb(53, 107, 83),
                foliage_light: rgb(98, 147, 99),
                water: rgb(100, 156, 161),
                water_light: rgb(175, 205, 187),
                wall: rgb(239, 214, 159),
                roof: rgb(169, 94, 66),
                trim: rgb(229, 194, 127),
                ink: rgb(62, 89, 75),
                foreground: rgb(93, 135, 79),
                foreground_shadow: rgb(66, 111, 73),
            },
            Biome::Forest => Self {
                sky: [
                    rgb(144, 181, 180),
                    rgb(170, 202, 192),
                    rgb(195, 218, 196),
                    rgb(219, 227, 188),
                ],
                sun: rgb(244, 235, 178),
                cloud: rgb(228, 235, 211),
                cloud_shadow: rgb(163, 193, 184),
                far: rgb(109, 155, 149),
                far_shadow: rgb(85, 131, 132),
                ridge: rgb(70, 126, 111),
                ridge_shadow: rgb(51, 104, 94),
                ground: rgb(104, 142, 91),
                ground_light: rgb(157, 177, 111),
                foliage: rgb(33, 86, 66),
                foliage_light: rgb(79, 131, 83),
                water: rgb(87, 151, 151),
                water_light: rgb(157, 200, 179),
                wall: rgb(209, 190, 138),
                roof: rgb(125, 85, 62),
                trim: rgb(220, 189, 124),
                ink: rgb(42, 66, 52),
                foreground: rgb(63, 107, 69),
                foreground_shadow: rgb(43, 85, 60),
            },
            Biome::Coast => Self {
                sky: [
                    rgb(142, 190, 208),
                    rgb(170, 209, 218),
                    rgb(211, 229, 219),
                    rgb(234, 235, 211),
                ],
                sun: rgb(255, 241, 193),
                cloud: rgb(248, 242, 221),
                cloud_shadow: rgb(190, 216, 218),
                far: rgb(130, 169, 177),
                far_shadow: rgb(102, 147, 161),
                ridge: rgb(130, 167, 151),
                ridge_shadow: rgb(96, 143, 137),
                ground: rgb(214, 197, 145),
                ground_light: rgb(242, 222, 169),
                foliage: rgb(75, 127, 113),
                foliage_light: rgb(132, 166, 121),
                water: rgb(65, 145, 162),
                water_light: rgb(150, 206, 201),
                wall: rgb(246, 231, 198),
                roof: rgb(187, 93, 68),
                trim: rgb(227, 195, 135),
                ink: rgb(61, 96, 100),
                foreground: rgb(167, 166, 119),
                foreground_shadow: rgb(118, 145, 109),
            },
            Biome::HighDesert => Self {
                sky: [
                    rgb(194, 180, 177),
                    rgb(224, 196, 177),
                    rgb(244, 214, 180),
                    rgb(250, 227, 188),
                ],
                sun: rgb(255, 241, 193),
                cloud: rgb(250, 231, 204),
                cloud_shadow: rgb(222, 195, 174),
                far: rgb(186, 147, 133),
                far_shadow: rgb(162, 119, 113),
                ridge: rgb(190, 124, 86),
                ridge_shadow: rgb(155, 95, 77),
                ground: rgb(218, 172, 110),
                ground_light: rgb(242, 202, 138),
                foliage: rgb(84, 124, 100),
                foliage_light: rgb(139, 160, 114),
                water: rgb(102, 148, 146),
                water_light: rgb(186, 199, 159),
                wall: rgb(235, 189, 128),
                roof: rgb(152, 91, 68),
                trim: rgb(247, 215, 158),
                ink: rgb(108, 83, 70),
                foreground: rgb(175, 132, 86),
                foreground_shadow: rgb(143, 109, 78),
            },
        }
    }

    /// A short, calm art cycle, not the user's actual local time. Colors stay
    /// legible at dusk; train lamps are handled separately from the landscape.
    #[must_use]
    pub fn at_time(biome: Biome, seconds: f64, enabled: bool) -> Self {
        let mut palette = Self::for_biome(biome);
        if !enabled {
            return palette;
        }
        let (tint, strength) = light_at(seconds);
        let light = |color| mix(color, tint, strength);
        palette.sky = palette.sky.map(light);
        palette.cloud = light(palette.cloud);
        palette.cloud_shadow = light(palette.cloud_shadow);
        palette.far = light(palette.far);
        palette.far_shadow = light(palette.far_shadow);
        palette.ridge = light(palette.ridge);
        palette.ridge_shadow = light(palette.ridge_shadow);
        palette.ground = light(palette.ground);
        palette.ground_light = light(palette.ground_light);
        palette.foliage = light(palette.foliage);
        palette.foliage_light = light(palette.foliage_light);
        palette.water = light(palette.water);
        palette.water_light = light(palette.water_light);
        palette.wall = light(palette.wall);
        palette.roof = light(palette.roof);
        palette.trim = light(palette.trim);
        palette.ink = light(palette.ink);
        palette.foreground = light(palette.foreground);
        palette.foreground_shadow = light(palette.foreground_shadow);
        palette.sun = mix(palette.sun, rgb(246, 172, 116), strength * 0.45);
        palette
    }
}

/// Round RGB channels, keeping every framebuffer pixel fully opaque.
pub(super) fn mix(a: Color32, b: Color32, amount: f32) -> Color32 {
    let t = if amount.is_finite() {
        amount.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let channel = |left: u8, right: u8| {
        (f32::from(left) + (f32::from(right) - f32::from(left)) * t).round() as u8
    };
    rgb(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
    )
}

fn light_at(seconds: f64) -> (Color32, f32) {
    let t = finite_nonnegative(seconds).rem_euclid(DAY_CYCLE_SECONDS) / DAY_CYCLE_SECONDS;
    let keys = [
        (0.0, rgb(246, 187, 139), 0.16),
        (0.30, rgb(255, 244, 210), 0.0),
        (0.58, rgb(234, 137, 107), 0.28),
        (0.75, rgb(37, 51, 88), 0.66),
        (0.90, rgb(48, 63, 94), 0.61),
        (1.0, rgb(246, 187, 139), 0.16),
    ];
    for pair in keys.windows(2) {
        if t <= pair[1].0 {
            let u = ((t - pair[0].0) / (pair[1].0 - pair[0].0)) as f32;
            let u = u * u * (3.0 - 2.0 * u);
            return (
                mix(pair[0].1, pair[1].1, u),
                pair[0].2 + (pair[1].2 - pair[0].2) * u,
            );
        }
    }
    (keys[0].1, keys[0].2)
}

pub(super) fn dusk_amount(seconds: f64, enabled: bool) -> f32 {
    if !enabled {
        return 0.0;
    }
    let t = finite_nonnegative(seconds).rem_euclid(DAY_CYCLE_SECONDS) / DAY_CYCLE_SECONDS;
    if (0.60..0.75).contains(&t) {
        ((t - 0.60) / 0.15) as f32
    } else if (0.75..0.90).contains(&t) {
        1.0
    } else if t >= 0.90 {
        ((1.0 - t) / 0.10) as f32
    } else {
        0.0
    }
}
