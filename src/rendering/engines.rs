//! Original engine silhouettes on the existing 60 x 29 engine socket.
use super::sprites::{Canvas, rgb, wheel};
use eframe::egui::Color32;

/// Visual equipment only; simulation speed and bounded particles are unchanged.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineStyle {
    #[default]
    Steam,
    Diesel,
    Electric,
    FreightSteam,
    Shunter,
    Railcar,
    ExpressSteam,
    ArticulatedSteam,
    TwinDiesel,
    RackElectric,
}

impl EngineStyle {
    pub const ALL: [Self; 10] = [
        Self::Steam,
        Self::Diesel,
        Self::Electric,
        Self::FreightSteam,
        Self::Shunter,
        Self::Railcar,
        Self::ExpressSteam,
        Self::ArticulatedSteam,
        Self::TwinDiesel,
        Self::RackElectric,
    ];

    pub const fn emits_smoke(self) -> bool {
        matches!(
            self,
            Self::Steam | Self::FreightSteam | Self::ExpressSteam | Self::ArticulatedSteam
        )
    }

    pub const fn required_reward(self) -> Option<crate::simulation::ServiceReward> {
        use crate::simulation::ServiceReward;
        match self {
            Self::Steam => None,
            Self::Diesel
            | Self::FreightSteam
            | Self::Shunter
            | Self::ArticulatedSteam
            | Self::TwinDiesel => Some(ServiceReward::CargoPostcard),
            Self::Electric | Self::Railcar | Self::ExpressSteam | Self::RackElectric => {
                Some(ServiceReward::PassengerPostcard)
            }
        }
    }

    pub const fn unlock_description(self) -> &'static str {
        match self.required_reward() {
            None => "Vapor original · disponível desde o início",
            Some(crate::simulation::ServiceReward::CargoPostcard) => {
                "Entregue 20 unidades de carga"
            }
            Some(crate::simulation::ServiceReward::PassengerPostcard) => "Entregue 20 passageiros",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Steam => "Vapor Aurora",
            Self::Diesel => "Diesel do Porto",
            Self::Electric => "Elétrica da Serra",
            Self::FreightSteam => "Vapor das Pedreiras",
            Self::Shunter => "Manobreira do Pátio",
            Self::Railcar => "Automotriz dos Campos",
            Self::ExpressSteam => "Expresso das Andorinhas",
            Self::ArticulatedSteam => "Articulada dos Vales",
            Self::TwinDiesel => "Diesel Dois Horizontes",
            Self::RackElectric => "Cremalheira do Mirante",
        }
    }
}

const INK: Color32 = rgb(37, 56, 52);
// Shared enamel keys: scene.rs applies the equipped livery to these bands.
const BODY: Color32 = rgb(42, 105, 88);
const LIGHT: Color32 = rgb(83, 143, 106);
const SHADE: Color32 = rgb(28, 79, 70);
const CREAM: Color32 = rgb(230, 218, 175);
const BRASS: Color32 = rgb(239, 194, 112);
const NAVY: Color32 = rgb(48, 71, 85);
const GLASS: Color32 = rgb(130, 181, 188);

/// Steam remains in rolling_stock; this function draws only the new bodies.
pub(super) fn draw_modern(canvas: &mut Canvas<'_>, phase: f64, style: EngineStyle, lit: bool) {
    match style {
        EngineStyle::Steam => return,
        EngineStyle::Diesel => diesel(canvas, lit),
        EngineStyle::Electric => electric(canvas, lit),
        EngineStyle::FreightSteam => {
            super::engine_details::freight_steam(canvas, phase, lit);
            return;
        }
        EngineStyle::Shunter => super::engine_details::shunter(canvas, lit),
        EngineStyle::Railcar => super::engine_details::railcar(canvas, lit),
        EngineStyle::ExpressSteam => {
            super::engine_specialists::express(canvas, phase, lit);
            return;
        }
        EngineStyle::ArticulatedSteam => {
            super::engine_specialists::articulated(canvas, phase, lit);
            return;
        }
        EngineStyle::TwinDiesel => super::engine_specialists::twin_diesel(canvas, lit),
        EngineStyle::RackElectric => super::engine_specialists::rack_electric(canvas, lit),
    }
    // Same rear coupling, wheel centers, rail contact and front pilot footprint
    // as sprites::locomotive. Modern axles intentionally have no steam side rod.
    canvas.rect(180, 51, 54, 2, INK);
    canvas.rect(184, 51, 49, 1, NAVY);
    canvas.rect(183, 53, 4, 1, BRASS);
    canvas.rect(184, 55, 3, 1, BRASS);
    canvas.polygon(&[(232, 50), (239, 56), (230, 56)], NAVY);
    canvas.line((232, 52), (236, 55), BRASS);
    for x in [201, 213, 225] {
        wheel(canvas, x, 54, 3, phase);
    }
    wheel(canvas, 233, 55, 2, phase);
}

fn window(canvas: &mut Canvas<'_>, x: i32, y: i32, width: i32, lit: bool) {
    canvas.rect(x, y, width, 6, INK);
    canvas.rect(x + 1, y + 1, width - 2, 4, if lit { BRASS } else { GLASS });
    canvas.rect(x + 1, y + 1, width - 2, 1, CREAM);
    canvas.pixel(x + 1, y + 4, rgb(87, 137, 148));
}

fn diesel(canvas: &mut Canvas<'_>, lit: bool) {
    // High rear cab, low long hood: a working harbor shunter, not a boiler.
    canvas.rect(184, 34, 17, 17, INK);
    canvas.rect(185, 35, 15, 15, BODY);
    canvas.rect(183, 33, 19, 2, NAVY);
    canvas.rect(185, 33, 15, 1, rgb(126, 148, 151));
    canvas.rect(185, 35, 15, 1, CREAM);
    window(canvas, 187, 37, 6, lit);
    window(canvas, 194, 37, 5, lit);
    canvas.rect(186, 44, 13, 1, CREAM);
    canvas.rect(188, 46, 7, 4, SHADE);
    canvas.pixel(194, 47, BRASS);
    canvas.rect(199, 39, 34, 12, INK);
    canvas.rect(200, 40, 31, 10, BODY);
    canvas.rect(201, 40, 29, 2, LIGHT);
    canvas.rect(200, 48, 31, 2, SHADE);
    canvas.rect(200, 46, 31, 2, CREAM);
    // Recessed radiator louvers, separate maintenance doors and roof fans.
    canvas.rect(204, 42, 13, 4, NAVY);
    for x in [205, 208, 211, 214] {
        canvas.rect(x, 42, 1, 4, rgb(113, 143, 138));
    }
    canvas.rect(219, 42, 1, 4, SHADE);
    canvas.rect(228, 42, 1, 4, SHADE);
    canvas.rect(222, 43, 4, 2, BRASS);
    for x in [206, 218] {
        canvas.rect(x, 38, 6, 1, NAVY);
        canvas.rect(x + 1, 38, 3, 1, rgb(114, 139, 138));
    }
    // Front walkway, handrail and diagonal buffer-board markings.
    canvas.rect(232, 44, 1, 7, BRASS);
    canvas.rect(229, 43, 5, 1, BRASS);
    canvas.rect(231, 47, 4, 4, NAVY);
    canvas.line((231, 47), (234, 50), CREAM);
    canvas.pixel(233, 41, if lit { rgb(255, 244, 190) } else { BRASS });
}

fn electric(canvas: &mut Canvas<'_>, lit: bool) {
    // Long double-ended saloon with sloping nose and a diamond pantograph.
    canvas.polygon(
        &[
            (184, 39),
            (188, 35),
            (228, 35),
            (235, 42),
            (235, 50),
            (184, 50),
        ],
        INK,
    );
    canvas.polygon(
        &[
            (185, 40),
            (189, 36),
            (227, 36),
            (233, 42),
            (233, 49),
            (185, 49),
        ],
        BODY,
    );
    canvas.rect(189, 36, 38, 2, CREAM);
    canvas.rect(185, 46, 48, 2, LIGHT);
    canvas.rect(185, 48, 48, 2, SHADE);
    canvas.rect(186, 45, 46, 1, CREAM);
    canvas.rect(190, 34, 36, 1, NAVY);
    // Insulators sit below the two open arms, keeping the top inside y=30.
    for x in [201, 215] {
        canvas.pixel(x, 33, BRASS);
        canvas.pixel(x, 34, BRASS);
    }
    canvas.line((203, 34), (209, 31), INK);
    canvas.line((209, 31), (215, 34), INK);
    canvas.rect(205, 30, 9, 1, BRASS);
    canvas.rect(203, 34, 13, 1, rgb(116, 136, 133));
    window(canvas, 187, 38, 7, lit);
    window(canvas, 223, 38, 7, lit);
    canvas.line((230, 40), (232, 42), GLASS);
    // Central cooling bank and access panel clearly differ from the diesel hood.
    canvas.rect(197, 39, 20, 5, NAVY);
    for x in [198, 202, 206, 210, 214] {
        canvas.rect(x, 40, 2, 3, rgb(113, 143, 138));
    }
    canvas.rect(195, 38, 1, 7, CREAM);
    canvas.rect(219, 38, 1, 7, CREAM);
    canvas.pixel(220, 44, BRASS);
    canvas.rect(204, 46, 10, 2, NAVY);
    canvas.rect(207, 46, 4, 1, BRASS);
    canvas.rect(232, 44, 2, 1, if lit { rgb(255, 244, 190) } else { BRASS });
    canvas.pixel(185, 44, rgb(189, 96, 69));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{HEIGHT, WIDTH, sprites::locomotive};

    fn frame(style: EngineStyle, phase: f64, lit: bool) -> Vec<Color32> {
        let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
        let mut canvas = Canvas::new(&mut pixels);
        if style == EngineStyle::Steam {
            locomotive(&mut canvas, phase);
        } else {
            draw_modern(&mut canvas, phase, style, lit);
        }
        pixels
    }

    #[test]
    fn engine_crop_preserves_all_wheels_and_coupling_sockets() {
        for style in EngineStyle::ALL {
            for phase in [0.0, 1.2, 3.0, 5.8] {
                for lit in [false, true] {
                    let pixels = frame(style, phase, lit);
                    for (i, pixel) in pixels.iter().enumerate() {
                        if *pixel != Color32::TRANSPARENT {
                            assert!((180..240).contains(&(i % WIDTH)), "{style:?}");
                            assert!((30..59).contains(&(i / WIDTH)), "{style:?}");
                        }
                    }
                    for x in [201, 213, 225, 233] {
                        assert_eq!(pixels[57 * WIDTH + x], rgb(39, 55, 53));
                        assert_eq!(pixels[58 * WIDTH + x], Color32::TRANSPARENT);
                    }
                    for x in 180..184 {
                        assert_eq!(pixels[51 * WIDTH + x], INK);
                    }
                }
            }
        }
    }

    #[test]
    fn engines_have_distinct_silhouettes_not_just_different_colors() {
        let masks = EngineStyle::ALL.map(|style| {
            frame(style, 0.0, false)
                .iter()
                .map(|p| p.a() != 0)
                .collect::<Vec<_>>()
        });
        for (i, mask) in masks.iter().enumerate() {
            for other in &masks[..i] {
                assert!(mask.iter().zip(other).filter(|(a, b)| a != b).count() > 80);
            }
        }
    }

    #[test]
    fn freight_steam_has_four_coupled_axles_and_the_original_chimney_socket() {
        for phase in [0.0, 0.8, 2.5, 5.9] {
            let pixels = frame(EngineStyle::FreightSteam, phase, false);
            for x in [189, 201, 213, 225] {
                assert_eq!(pixels[57 * WIDTH + x], rgb(39, 55, 53));
                let dy = (phase.sin() * 2.0).round() as i32;
                let x = x as i32 + (phase.cos() * 2.0).round() as i32;
                assert_eq!(pixels[(54 + dy) as usize * WIDTH + x as usize], CREAM);
            }
            let original = frame(EngineStyle::Steam, phase, false);
            for y in 31..34 {
                assert_eq!(
                    &pixels[y * WIDTH + 216..y * WIDTH + 224],
                    &original[y * WIDTH + 216..y * WIDTH + 224]
                );
            }
        }
    }

    #[test]
    fn second_wave_steam_keeps_emitter_and_has_phase_locked_cranks() {
        for phase in [0.0_f64, 0.8, 2.5, 5.9] {
            let original = frame(EngineStyle::Steam, phase, false);
            for style in [EngineStyle::ExpressSteam, EngineStyle::ArticulatedSteam] {
                let pixels = frame(style, phase, false);
                for y in 31..34 {
                    assert_eq!(
                        &pixels[y * WIDTH + 216..y * WIDTH + 224],
                        &original[y * WIDTH + 216..y * WIDTH + 224]
                    );
                }
                let (centers, cy, arm): (&[i32], i32, f64) = if style == EngineStyle::ExpressSteam {
                    (&[201, 213, 225], 53, 3.0)
                } else {
                    (&[189, 201, 213, 225], 54, 2.0)
                };
                for &cx in centers {
                    let angle = phase
                        + if style == EngineStyle::ArticulatedSteam && cx >= 213 {
                            1.2
                        } else {
                            0.0
                        };
                    let x = cx + (angle.cos() * arm).round() as i32;
                    let y = cy + (angle.sin() * arm).round() as i32;
                    assert_eq!(pixels[y as usize * WIDTH + x as usize], CREAM, "{style:?}");
                }
                assert_ne!(pixels, frame(style, phase + 0.7, false));
            }
        }
    }

    #[test]
    fn second_wave_sprites_clip_exactly_at_both_edges_for_all_widths() {
        for style in &EngineStyle::ALL[6..] {
            for phase in [0.0, 1.2, 3.0, 5.8] {
                for lit in [false, true] {
                    let reference = frame(*style, phase, lit);
                    for width in [160, 320, 641, 1920] {
                        for origin in [-210, -180, width as i32 - 210, width as i32 - 240] {
                            let mut pixels = vec![Color32::TRANSPARENT; width * HEIGHT];
                            let mut c = Canvas::new(&mut pixels);
                            c.origin = origin;
                            draw_modern(&mut c, phase, *style, lit);
                            for y in 0..HEIGHT {
                                for x in 0..width {
                                    let native = x as i32 - origin;
                                    let expected = if (0..WIDTH as i32).contains(&native) {
                                        reference[y * WIDTH + native as usize]
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    assert_eq!(
                                        pixels[y * width + x],
                                        expected,
                                        "{style:?} at {x},{y}"
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
