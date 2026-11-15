//! Original fictional ambient art. No events, clocks, RNG, I/O or saved state.
//! Local motion is bounded inside each 320 px tile; spatial periods stay intact.
use super::{
    SegmentScenery,
    palette::{Palette, dusk_amount, mix},
    scenery::View,
    sprites::Canvas,
    theme::{RenderQuality, Weather, finite_nonnegative},
};
use crate::simulation::catalog::SceneryTag as Tag;

/// Presentation only. Empty scenery and Legacy never use these controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SceneryLifeOptions {
    pub reduce_motion: bool,
}

fn seconds(v: View) -> f64 {
    if v.scenery_life.reduce_motion || v.quality == RenderQuality::Low || !v.options.details {
        0.0
    } else {
        finite_nonnegative(v.seconds)
    }
}

fn phase(v: View, period: f64, steps: i32) -> i32 {
    (seconds(v).rem_euclid(period) / period * f64::from(steps)).floor() as i32
}

fn sway(v: View, period: f64, amplitude: i32) -> i32 {
    let step = phase(v, period, amplitude * 4);
    amplitude - (step - amplitude * 2).abs()
}

pub(super) fn ship_drift(v: View) -> i32 {
    sway(v, 96.0, 6)
}

pub(super) fn window_light(p: &Palette, v: View) -> eframe::egui::Color32 {
    mix(
        p.water,
        p.sun,
        dusk_amount(v.light_seconds, v.options.day_cycle),
    )
}

pub(super) fn middle(c: &mut Canvas<'_>, p: &Palette, v: View, tags: SegmentScenery, x: i32) {
    let has = |t| tags.contains(t);
    let detail = v.options.details && v.quality == RenderQuality::Full;
    let wet = has(Tag::River) || has(Tag::Coast) || has(Tag::Port);
    let rural = (has(Tag::Farmland) || has(Tag::Plain) || has(Tag::Plateau)) && !wet;
    let dusk = dusk_amount(v.light_seconds, v.options.day_cycle);
    if rural {
        animal(c, p, x + 65, 49, false, phase(v, 16.0, 4));
        animal(c, p, x + 91, 47, true, phase(v, 24.0, 4));
    }
    if has(Tag::Farmland) {
        mill(c, p, x + 194, v, false);
        // The existing farmhouse chimney ends at (x+264, 27).
        chimney(c, p, x + 264, 26, v, detail);
        clothes(c, p, x + 211, v);
        c.rect(x + 245, 36, 2, 2, window_light(p, v));
    } else if (has(Tag::Plateau) || has(Tag::Plain)) && !wet {
        mill(c, p, x + 192, v, true);
    }
    if has(Tag::Urban) {
        chimney(c, p, x + 106, 32, v, detail);
        clothes(c, p, x + 146, v);
        for dx in [94, 124, 261, 287] {
            c.rect(x + dx, 42, 2, 2, window_light(p, v));
        }
    }
    if wet && !has(Tag::Port) {
        boat(c, p, x + 145 + sway(v, 64.0, 8), 47, v);
        fisher(c, p, x + 223, v);
    }
    if has(Tag::Coast) {
        lighthouse(c, p, x + 270, v);
    }
    if has(Tag::Mountain) {
        waterfall(c, p, x + 114, v);
    }
    if detail {
        if (rural || wet || has(Tag::Forest)) && dusk < 0.7 {
            birds(c, p, x + 48 + sway(v, 64.0, 12), v);
        }
        if (has(Tag::Forest) || has(Tag::River)) && dusk > 0.4 {
            for (i, dx, y) in [(0, 28, 45), (1, 49, 49), (2, 270, 47)] {
                let pulse = (phase(v, 16.0, 8) + i * 2) % 8;
                let glow = if pulse < 4 { 0.8 } else { 0.4 } * dusk;
                c.pixel(x + dx + sway(v, 32.0, 1), y, mix(p.foliage, p.sun, glow));
            }
        }
        if (has(Tag::Mountain) || has(Tag::Viaduct) || has(Tag::Plain))
            && (dusk > 0.35 || v.options.weather == Weather::Fog)
        {
            // Sparse low-contrast valley ribbons, never over the rail or train.
            let shift = sway(v, 64.0, 4);
            for (dx, y, w) in [(27, 36, 35), (35, 38, 49), (240, 40, 31)] {
                c.rect(x + dx + shift, y, w, 1, mix(p.ridge, p.cloud_shadow, 0.28));
            }
        }
    }
}

fn animal(c: &mut Canvas<'_>, p: &Palette, x: i32, y: i32, horse: bool, frame: i32) {
    let coat = if horse { p.roof } else { p.wall };
    c.rect(x, y - 6, 11, 5, coat);
    if !horse {
        c.rect(x + 2, y - 6, 3, 3, p.ink);
        c.rect(x + 7, y - 4, 2, 2, p.ink);
    }
    let graze = i32::from(frame >= 2);
    c.rect(x + 9, y - 8 + graze * 4, 3, 5, coat);
    c.rect(x + 11, y - 8 + graze * 4, 4, 3, coat);
    c.pixel(x + 13, y - 7 + graze * 4, p.ink);
    c.pixel(x + 10, y - 9 + graze * 4, p.ink);
    for dx in [1, 8] {
        c.rect(x + dx, y - 1, 1, 3, p.ink);
    }
    c.line((x, y - 5), (x - 2, y - 2 + frame % 2), p.ink);
    if horse {
        c.rect(x + 9, y - 8 + graze * 4, 1, 4, p.ink);
    }
}

fn mill(c: &mut Canvas<'_>, p: &Palette, x: i32, v: View, windpump: bool) {
    if windpump {
        c.line((x, 31), (x - 4, 50), p.ink);
        c.line((x, 31), (x + 4, 50), p.ink);
        c.line((x - 3, 42), (x + 3, 47), p.trim);
        c.line((x + 3, 42), (x - 3, 47), p.trim);
        c.line((x, 31), (x + 12, 31), p.ink);
        c.rect(x + 9, 29, 4, 4, p.roof);
    } else {
        c.polygon(
            &[(x - 4, 34), (x + 4, 34), (x + 6, 50), (x - 6, 50)],
            p.wall,
        );
        c.polygon(&[(x - 6, 34), (x, 27), (x + 6, 34)], p.roof);
        c.rect(x - 1, 45, 3, 5, p.ink);
    }
    let y = if windpump { 31 } else { 35 };
    // Four two-second poses; the fourfold rotor repeats without a large jump.
    let (dx, dy) = [(7, 0), (6, 3), (5, 5), (3, 6)][phase(v, 8.0, 4) as usize];
    for (a, b) in [(dx, dy), (-dy, dx), (-dx, -dy), (dy, -dx)] {
        c.line((x, y), (x + a, y + b), p.ink);
        c.line((x + a / 2 + 1, y + b / 2), (x + a + 1, y + b), p.trim);
    }
    c.rect(x, y, 2, 2, p.roof);
}

fn clothes(c: &mut Canvas<'_>, p: &Palette, x: i32, v: View) {
    for dx in [0, 22] {
        c.rect(x + dx, 39, 1, 11, p.ink);
    }
    c.line((x, 40), (x + 22, 41), p.ink);
    let shift = sway(v, 8.0, 1);
    for (dx, color) in [(4, p.wall), (11, p.roof), (17, p.water_light)] {
        c.rect(x + dx, 41, 4, 2, color);
        c.rect(x + dx + shift, 43, 4, 4, color);
        c.pixel(x + dx, 40, p.trim);
    }
}

fn chimney(c: &mut Canvas<'_>, p: &Palette, x: i32, y: i32, v: View, detail: bool) {
    if detail {
        for i in 0..3 {
            let rise = (phase(v, 12.0, 12) + i * 4) % 12;
            c.rect(
                x + rise / 4,
                y - rise,
                2 + rise / 5,
                2,
                mix(p.sky[2], p.cloud_shadow, 0.5),
            );
        }
    }
}

fn birds(c: &mut Canvas<'_>, p: &Palette, x: i32, v: View) {
    let wing = [0, -1, 0, 1][phase(v, 4.0, 4) as usize];
    for (dx, dy) in [(0, 0), (9, -3), (18, 1)] {
        c.line(
            (x + dx - 2, 18 + dy + wing),
            (x + dx, 19 + dy),
            p.far_shadow,
        );
        c.line(
            (x + dx, 19 + dy),
            (x + dx + 2, 18 + dy + wing),
            p.far_shadow,
        );
    }
}

fn boat(c: &mut Canvas<'_>, p: &Palette, x: i32, y: i32, v: View) {
    c.polygon(
        &[(x - 2, y - 3), (x + 16, y - 3), (x + 12, y), (x + 2, y)],
        p.roof,
    );
    c.rect(x + 4, y - 6, 7, 3, p.wall);
    c.rect(x + 5, y - 5, 4, 2, p.water);
    c.rect(x + 1 + phase(v, 8.0, 3), y + 1, 11, 1, p.water_light);
}

fn fisher(c: &mut Canvas<'_>, p: &Palette, x: i32, v: View) {
    c.rect(x - 4, 48, 12, 2, p.ink);
    c.rect(x + 4, 50, 1, 3, p.ink);
    c.rect(x, 40, 3, 3, p.wall);
    c.rect(x - 1, 39, 5, 1, p.trim);
    c.rect(x, 43, 3, 5, p.roof);
    c.line((x + 2, 48), (x + 5, 50), p.ink);
    let tip = phase(v, 16.0, 2);
    c.line((x + 3, 44), (x + 13, 37 + tip), p.ink);
    c.line((x + 13, 37 + tip), (x + 17, 48), p.cloud_shadow);
    c.pixel(x + 17, 48, p.trim);
}

fn lighthouse(c: &mut Canvas<'_>, p: &Palette, x: i32, v: View) {
    c.polygon(
        &[(x - 5, 47), (x - 3, 22), (x + 3, 22), (x + 5, 47)],
        p.wall,
    );
    c.rect(x - 4, 34, 9, 3, p.roof);
    c.rect(x - 4, 21, 9, 5, p.ink);
    c.rect(x - 3, 22, 7, 2, window_light(p, v));
    c.polygon(&[(x - 5, 21), (x, 17), (x + 5, 21)], p.roof);
    let dusk = dusk_amount(v.light_seconds, v.options.day_cycle);
    if dusk > 0.35 {
        let reach = 8 + sway(v, 16.0, 3);
        c.line(
            (x + 5, 23),
            (x + reach, 23),
            mix(p.sky[1], p.sun, dusk * 0.4),
        );
    }
}

fn waterfall(c: &mut Canvas<'_>, p: &Palette, x: i32, v: View) {
    c.polygon(
        &[(x, 28), (x + 6, 28), (x + 7, 40), (x + 12, 49), (x + 3, 49)],
        p.water,
    );
    c.line((x + 1, 29), (x + 3, 41), p.water_light);
    for i in 0..4 {
        let y = 30 + (i * 5 + phase(v, 4.0, 5)) % 20;
        c.rect(x + 2 + (y - 30) / 6, y, 3, 1, p.cloud);
    }
    c.rect(x + 2, 50, 14, 1, p.water_light);
}

pub(super) fn near(c: &mut Canvas<'_>, p: &Palette, v: View, tags: SegmentScenery, x: i32) {
    // The signal stays on the same socket as Agent 2's static signal.
    let stop = phase(v, 32.0, 4) != 3;
    c.rect(x + 309, 39, 2, 16, p.ink);
    c.rect(x + 307, 36, 6, 8, p.ink);
    c.rect(x + 309, 38, 2, 2, if stop { p.roof } else { p.far_shadow });
    c.rect(
        x + 309,
        41,
        2,
        2,
        if stop { p.far_shadow } else { p.foliage_light },
    );
    c.line((x + 309, 35), (x + 302, if stop { 35 } else { 31 }), p.trim);
    if (tags.contains(Tag::Urban) || tags.contains(Tag::Farmland) || tags.contains(Tag::Plain))
        && !tags.contains(Tag::Bridge)
        && !tags.contains(Tag::Viaduct)
        && !tags.contains(Tag::Tunnel)
    {
        crossing(c, p, v, x + 34);
    }
}

fn crossing(c: &mut Canvas<'_>, p: &Palette, v: View, x: i32) {
    c.rect(x + 4, 40, 12, 16, p.ridge_shadow);
    c.rect(x + 10, 42, 1, 5, p.trim);
    c.rect(x, 48, 2, 8, p.ink);
    // Geometry, not elapsed time, keeps the road closed while our train passes.
    // Reduced motion keeps the conservative closed pose at every position.
    // Includes both the normal train and its recentered narrow-viewport pose.
    // Fixed bounds keep clipping identical at every viewport width.
    let closed = v.scenery_life.reduce_motion || (-96..=285).contains(&x);
    let end = if closed { (x + 18, 49) } else { (x + 4, 35) };
    c.line((x, 49), end, p.wall);
    for i in [1, 3, 5] {
        let px = x + (end.0 - x) * i / 6;
        let py = 49 + (end.1 - 49) * i / 6;
        c.rect(px, py, 2, 1, p.roof);
    }
    // Two stationary vehicles wait behind the barrier; no traffic simulation.
    for (dy, color) in [(0, p.roof), (-9, p.water_light)] {
        c.rect(x + 5, 41 + dy, 9, 5, color);
        c.rect(x + 7, 39 + dy, 5, 3, color);
        c.rect(x + 8, 40 + dy, 3, 2, p.water);
        c.rect(x + 5, 45 + dy, 2, 2, p.ink);
        c.rect(x + 12, 45 + dy, 2, 2, p.ink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{Biome, SceneOptions};
    use eframe::egui::Color32;

    const TAGS: [Tag; 12] = [
        Tag::Plain,
        Tag::Farmland,
        Tag::Forest,
        Tag::Mountain,
        Tag::River,
        Tag::Bridge,
        Tag::Viaduct,
        Tag::Tunnel,
        Tag::Urban,
        Tag::Port,
        Tag::Coast,
        Tag::Plateau,
    ];

    fn view() -> View {
        View {
            scenery_life: SceneryLifeOptions::default(),
            wind_speed_kmh: None,
            precipitation_mm: None,
            distance: 0.0,
            seconds: 0.0,
            light_seconds: 480.0,
            options: SceneOptions::default(),
            quality: RenderQuality::Full,
        }
    }

    fn frame(tag: Tag, v: View) -> Vec<Color32> {
        let mut pixels = vec![Color32::TRANSPARENT; 320 * 70];
        let p = Palette::at_time(Biome::Forest, v.light_seconds, true);
        let tags = SegmentScenery::from_tags(&[tag]);
        let mut c = Canvas::new(&mut pixels);
        middle(&mut c, &p, v, tags, 0);
        near(&mut c, &p, v, tags, 0);
        pixels
    }

    #[test]
    fn animacoes_tem_quadros_distintos_e_reducao_congela_todas_as_etiquetas() {
        for tag in TAGS {
            let initial = frame(tag, view());
            let mut changed = false;
            for seconds in [1.0, 2.0, 4.0, 8.0, 12.0, 24.0, 48.0, 96.0] {
                let v = View { seconds, ..view() };
                changed |= frame(tag, v) != initial;
                assert_eq!(
                    initial,
                    frame(
                        tag,
                        View {
                            scenery_life: SceneryLifeOptions {
                                reduce_motion: true
                            },
                            ..v
                        }
                    ),
                    "{tag:?}: pose reduzida"
                );
            }
            assert!(changed, "{tag:?}: nenhum movimento");
        }
    }

    #[test]
    fn componentes_tem_movimento_proprio_limitado_e_estatico_em_low() {
        let p = Palette::for_biome(Biome::GreenValley);
        // Isolate every time-dependent drawing routine; signals cannot mask
        // missing motion in an unrelated animal, boat or piece of clothing.
        for component in 0..10 {
            let draw = |v: View| {
                let mut pixels = vec![Color32::TRANSPARENT; 320 * 70];
                let mut c = Canvas::new(&mut pixels);
                match component {
                    0 => animal(&mut c, &p, 100, 49, false, phase(v, 16.0, 4)),
                    1 => animal(&mut c, &p, 100, 49, true, phase(v, 24.0, 4)),
                    2 => mill(&mut c, &p, 100, v, false),
                    3 => mill(&mut c, &p, 100, v, true),
                    4 => clothes(&mut c, &p, 100, v),
                    5 => chimney(&mut c, &p, 100, 30, v, true),
                    6 => birds(&mut c, &p, 100, v),
                    7 => boat(&mut c, &p, 100 + sway(v, 64.0, 8), 47, v),
                    8 => fisher(&mut c, &p, 100, v),
                    _ => waterfall(&mut c, &p, 100, v),
                }
                pixels
            };
            let initial = draw(view());
            assert!(
                (1..32).any(|s| draw(View {
                    seconds: f64::from(s),
                    ..view()
                }) != initial),
                "componente {component}"
            );
            for seconds in [
                0.0,
                2.0,
                12.0,
                31.0,
                f64::MAX,
                f64::NAN,
                f64::INFINITY,
                -1.0,
            ] {
                assert_eq!(
                    initial,
                    draw(View {
                        seconds,
                        quality: RenderQuality::Low,
                        ..view()
                    })
                );
            }
        }
        assert_ne!(
            ship_drift(view()),
            ship_drift(View {
                seconds: 48.0,
                ..view()
            })
        );
    }

    #[test]
    fn luzes_seguem_iluminacao_mesmo_com_movimento_reduzido() {
        let v = View {
            scenery_life: SceneryLifeOptions {
                reduce_motion: true,
            },
            ..view()
        };
        let p = Palette::for_biome(Biome::Coast);
        assert_ne!(
            window_light(&p, v),
            window_light(
                &p,
                View {
                    light_seconds: 180.0,
                    ..v
                }
            )
        );
        for tag in [Tag::Coast, Tag::Forest, Tag::River, Tag::Urban] {
            assert_ne!(
                frame(tag, v),
                frame(
                    tag,
                    View {
                        light_seconds: 180.0,
                        ..v
                    }
                )
            );
        }
    }

    #[test]
    fn cancela_fica_fechada_perto_do_trem_e_reducao_preserva_pose() {
        let p = Palette::for_biome(Biome::GreenValley);
        for width in [160, 320, 641, 1920] {
            let mut pixels = vec![Color32::TRANSPARENT; width * 70];
            let mut c = Canvas::new(&mut pixels);
            let x = c.train_origin() + 100;
            crossing(&mut c, &p, view(), x);
            assert_eq!(pixels[49 * width + (x + 17) as usize], p.wall);
        }
        let mut pixels = vec![Color32::TRANSPARENT; 641 * 70];
        crossing(&mut Canvas::new(&mut pixels), &p, view(), 400);
        assert_eq!(pixels[35 * 641 + 404], p.wall);
        pixels.fill(Color32::TRANSPARENT);
        crossing(
            &mut Canvas::new(&mut pixels),
            &p,
            View {
                scenery_life: SceneryLifeOptions {
                    reduce_motion: true,
                },
                ..view()
            },
            400,
        );
        assert_eq!(pixels[49 * 641 + 417], p.wall);
    }

    #[test]
    fn vida_preserva_wrap_e_recorte_em_todas_as_fases() {
        let p = Palette::for_biome(Biome::Forest);
        for tag in TAGS {
            let tags = SegmentScenery::from_tags(&[tag]);
            for seconds in [0.0, 12.0, 24.0] {
                for distance in [0.0, 127.25, 2559.75] {
                    let v = View {
                        seconds,
                        distance,
                        ..view()
                    };
                    let mut wide = vec![Color32::BLACK; 1920 * 70];
                    super::super::landmarks::draw(&mut Canvas::new(&mut wide), &p, v, tags);
                    for width in [160, 320, 641, 1920] {
                        let mut pixels = vec![Color32::BLACK; width * 70];
                        super::super::landmarks::draw(
                            &mut Canvas::new(&mut pixels),
                            &p,
                            View {
                                distance: distance + crate::simulation::SCROLL_PERIOD_PX,
                                ..v
                            },
                            tags,
                        );
                        for y in 0..70 {
                            assert_eq!(
                                &pixels[y * width..(y + 1) * width],
                                &wide[y * 1920..y * 1920 + width],
                                "{tag:?}, {seconds}, {distance}, {width}"
                            );
                        }
                    }
                }
            }
        }
    }
}
