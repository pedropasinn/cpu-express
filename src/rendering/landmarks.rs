//! Original fictional landscape vignettes, not surveyed railway structures.
//! Every tile is 320 px. Ratios 1/32, 1/8 and 1/4 divide SCROLL_PERIOD_PX.
//! The train, stations and rail baseline are painted later, never occluded here.
use super::{
    SegmentScenery,
    palette::{Palette, mix},
    scenery::View,
    sprites::Canvas,
    theme::{RenderQuality, finite_nonnegative},
};
use crate::simulation::catalog::SceneryTag as Tag;

fn offset(distance: f64, ratio: f64) -> i32 {
    (finite_nonnegative(distance).rem_euclid(320.0 / ratio) * ratio).floor() as i32
}

pub(super) fn draw(c: &mut Canvas<'_>, p: &Palette, v: View, tags: SegmentScenery) {
    let has = |tag| tags.contains(tag);
    let detail = v.options.details && v.quality == RenderQuality::Full;
    let far = offset(v.distance, 0.03125);
    let middle = offset(v.distance, 0.125);
    let near = offset(v.distance, 0.25);
    let mountain = has(Tag::Mountain) || has(Tag::Tunnel) || has(Tag::Viaduct);
    let wet = has(Tag::Coast) || has(Tag::Port) || has(Tag::River);
    c.rect(0, 38, c.width(), 32, p.ground);
    for tile in c.tiles(320) {
        let x = tile - far;
        if mountain {
            c.polygon(
                &[
                    (x, 44),
                    (x + 25, 31),
                    (x + 59, 13),
                    (x + 72, 20),
                    (x + 99, 7),
                    (x + 130, 32),
                    (x + 165, 23),
                    (x + 198, 36),
                    (x + 229, 15),
                    (x + 257, 27),
                    (x + 282, 22),
                    (x + 320, 44),
                ],
                p.far,
            );
            c.polygon(&[(x + 99, 7), (x + 108, 29), (x + 130, 32)], p.far_shadow);
            c.line((x + 59, 13), (x + 48, 24), p.cloud_shadow);
            c.polygon(
                &[
                    (x, 53),
                    (x, 40),
                    (x + 36, 33),
                    (x + 70, 42),
                    (x + 117, 28),
                    (x + 152, 39),
                    (x + 207, 29),
                    (x + 248, 37),
                    (x + 288, 30),
                    (x + 320, 41),
                    (x + 320, 53),
                ],
                mix(p.ridge, p.foliage, 0.55),
            );
        } else {
            c.polygon(
                &[
                    (x, 43),
                    (x, 37),
                    (x + 42, 34),
                    (x + 94, 37),
                    (x + 147, 31),
                    (x + 209, 36),
                    (x + 266, 33),
                    (x + 320, 37),
                    (x + 320, 43),
                ],
                p.far,
            );
        }
    }
    if wet {
        c.rect(0, 40, c.width(), 13, p.water);
        c.rect(0, 40, c.width(), 1, p.water_light);
    }
    // Canopies can extend left of their tile. At a near-complete scroll offset,
    // the next tile's left fringe is visible even at an exact-width boundary.
    let last_middle_tile = ((c.width() + 319) / 320 + 1) * 320;
    for tile in (-320..=last_middle_tile).step_by(320) {
        let x = tile - middle;
        if has(Tag::Plain) {
            // Lowland drainage channels, reeds and low, uninterrupted horizons.
            c.polygon(
                &[
                    (x + 6, 46),
                    (x + 62, 44),
                    (x + 104, 48),
                    (x + 88, 50),
                    (x + 27, 48),
                ],
                p.water,
            );
            for dx in [16, 36, 91, 156, 202, 285] {
                grass(c, x + dx, 49, p);
            }
        }
        if has(Tag::Farmland) {
            c.polygon(
                &[(x, 44), (x + 130, 40), (x + 184, 51), (x, 53)],
                p.ground_light,
            );
            for row in 0..5 {
                c.line((x + row * 22, 44), (x + 40 + row * 24, 52), p.foreground);
            }
            house(c, x + 241, 43, 27, p);
            c.rect(x + 277, 27, 8, 17, p.wall);
            c.polygon(&[(x + 276, 27), (x + 281, 23), (x + 286, 27)], p.roof);
            c.rect(x + 238, 44, 53, 1, p.ink);
            for post in (232..304).step_by(9) {
                c.rect(x + post, 43, 1, 6, p.trim);
            }
            c.rect(x + 232, 46, 72, 1, p.trim);
        }
        if has(Tag::Forest) || has(Tag::Mountain) {
            for (dx, base, height) in [
                (5, 45, 15),
                (25, 40, 13),
                (46, 42, 18),
                (81, 46, 15),
                (119, 39, 14),
                (148, 43, 17),
                (177, 46, 12),
                (204, 40, 17),
                (237, 43, 16),
                (269, 42, 18),
                (302, 45, 13),
            ] {
                broadleaf(c, x + dx, base, height, p, detail);
            }
        }
        if has(Tag::River) {
            c.polygon(
                &[
                    (x + 123, 40),
                    (x + 139, 40),
                    (x + 183, 49),
                    (x + 168, 56),
                    (x + 74, 56),
                    (x + 137, 48),
                ],
                p.water_light,
            );
            c.polygon(
                &[
                    (x + 129, 40),
                    (x + 135, 40),
                    (x + 173, 49),
                    (x + 152, 55),
                    (x + 98, 55),
                    (x + 146, 48),
                ],
                p.water,
            );
            for dx in [28, 52, 219, 252] {
                grass(c, x + dx, 49, p);
            }
        }
        if has(Tag::Port) {
            ship(c, x + 100 + super::landscape_life::ship_drift(v), p);
            crane(c, x + 25, p);
            crane(c, x + 242, p);
            c.rect(x + 213, 44, 103, 3, p.ink);
            for (dx, color) in [(216, p.roof), (235, p.trim), (254, p.ridge_shadow)] {
                c.rect(x + dx, 38, 17, 6, color);
                if detail {
                    for rib in (2..16).step_by(4) {
                        c.rect(x + dx + rib, 39, 1, 4, p.ink);
                    }
                }
            }
        }
        if has(Tag::Coast) || has(Tag::Port) {
            // Exposed prop roots distinguish mangrove from the upland forest.
            for dx in [6, 68, 295] {
                mangrove(c, x + dx, p);
            }
            if has(Tag::Coast) {
                c.polygon(
                    &[
                        (x + 100, 51),
                        (x + 158, 47),
                        (x + 209, 49),
                        (x + 244, 53),
                        (x + 100, 53),
                    ],
                    p.ground_light,
                );
            }
        }
        if has(Tag::Plateau) {
            c.polygon(
                &[
                    (x, 50),
                    (x + 70, 43),
                    (x + 134, 47),
                    (x + 206, 42),
                    (x + 320, 48),
                    (x + 320, 54),
                    (x, 54),
                ],
                p.ground_light,
            );
            for (dx, height) in [(21, 25), (66, 18), (152, 28), (264, 23), (294, 16)] {
                araucaria(c, x + dx, 48, height, p);
            }
            for dx in [40, 113, 194, 240] {
                grass(c, x + dx, 51, p);
            }
        }
        if has(Tag::Urban) {
            for (dx, w, h) in [
                (12, 20, 22),
                (38, 15, 29),
                (62, 25, 18),
                (174, 18, 25),
                (199, 24, 20),
                (230, 15, 31),
            ] {
                c.rect(x + dx, 44 - h, w, h, p.far_shadow);
                c.rect(x + dx, 44 - h, w, 2, p.ridge);
                if detail {
                    for y in ((46 - h)..42).step_by(5) {
                        for column in (3..w - 2).step_by(5) {
                            c.rect(
                                x + dx + column,
                                y,
                                2,
                                2,
                                super::landscape_life::window_light(p, v),
                            );
                        }
                    }
                }
            }
            for dx in [90, 120, 257, 283] {
                house(c, x + dx, 49, 22, p);
            }
        }
        if detail && wet {
            for (dx, y) in [(18, 43), (85, 48), (158, 45), (211, 51), (272, 49)] {
                c.rect(x + dx, y, 13, 1, p.water_light);
            }
        }
        if has(Tag::Viaduct) {
            viaduct(c, x, p);
        }
        super::landscape_life::middle(c, p, v, tags, x);
    }
    super::settlements::draw(c, p, v, tags);
    // The playing track stays horizontal. Structures suggest curves only in
    // the distance; nothing moves the train's sockets or the wheel baseline.
    c.rect(0, 54, c.width(), 16, p.foreground_shadow);
    c.rect(0, 54, c.width(), 2, p.ground_light);
    if has(Tag::Bridge) || has(Tag::Viaduct) {
        c.rect(
            0,
            60,
            c.width(),
            10,
            if has(Tag::Bridge) {
                p.water
            } else {
                p.far_shadow
            },
        );
    }
    for tile in c.tiles(320) {
        let x = tile - near;
        if has(Tag::Bridge) {
            bridge(c, x, p, detail);
        }
        if has(Tag::Viaduct) {
            c.rect(x, 61, 320, 2, p.wall);
            for dx in (0..320).step_by(40) {
                c.rect(x + dx, 62, 5, 8, p.ridge);
                c.line((x + dx + 5, 66), (x + dx + 12, 63), p.trim);
                c.line((x + dx + 33, 63), (x + dx + 40, 66), p.trim);
            }
        }
        if has(Tag::Tunnel) {
            tunnel(c, x + 252, p, detail);
        }
        if detail {
            c.rect(x + 10, 24, 2, 31, p.ink);
            c.rect(x + 5, 26, 12, 1, p.trim);
            for dx in [6, 15] {
                c.rect(x + dx, 24, 1, 2, p.wall);
            }
            c.line((x + 17, 27), (x + 83, 29), p.far_shadow);
            super::landscape_life::near(c, p, v, tags, x);
        }
    }
}

fn grass(c: &mut Canvas<'_>, x: i32, y: i32, p: &Palette) {
    c.line((x, y), (x - 2, y - 4), p.foliage);
    c.line((x, y), (x + 2, y - 5), p.foliage_light);
    c.rect(x + 3, y - 5, 1, 2, p.trim);
}

fn house(c: &mut Canvas<'_>, x: i32, base: i32, width: i32, p: &Palette) {
    c.rect(x, base - 10, width, 10, p.wall);
    c.polygon(
        &[
            (x - 2, base - 10),
            (x + width / 2, base - 17),
            (x + width + 2, base - 10),
        ],
        p.roof,
    );
    c.rect(x + width - 6, base - 16, 2, 4, p.ink);
    c.rect(x + 3, base - 7, 4, 4, p.ink);
    c.rect(x + 4, base - 7, 2, 2, p.trim);
    c.rect(x + width - 8, base - 6, 4, 6, p.roof);
    c.rect(x, base, width, 1, p.ink);
}

fn broadleaf(c: &mut Canvas<'_>, x: i32, base: i32, height: i32, p: &Palette, detail: bool) {
    c.rect(x, base - height + 5, 2, height - 5, p.ink);
    let y = base - height + 5;
    for (dx, dy, r) in [(-4, 1, 5), (3, 2, 6), (0, -2, 5)] {
        c.circle(x + dx, y + dy, r, p.foliage);
    }
    if detail {
        c.rect(x - 6, y - 1, 5, 2, p.foliage_light);
        c.rect(x + 1, y - 4, 4, 1, p.foliage_light);
    }
}

fn mangrove(c: &mut Canvas<'_>, x: i32, p: &Palette) {
    for dx in [-5, 0, 5] {
        c.line((x, 43), (x + dx, 49), p.ink);
        c.line((x + 3, 43), (x + dx + 5, 49), p.ridge_shadow);
    }
    c.rect(x, 37, 3, 7, p.ink);
    c.circle(x - 3, 37, 4, p.foliage);
    c.circle(x + 4, 36, 5, p.foliage);
    c.rect(x - 5, 33, 9, 2, p.foliage_light);
}

fn araucaria(c: &mut Canvas<'_>, x: i32, base: i32, height: i32, p: &Palette) {
    let top = base - height;
    c.rect(x, top + 4, 2, height - 4, p.ink);
    c.line((x, top + 11), (x - 9, top + 5), p.ink);
    c.line((x + 1, top + 11), (x + 11, top + 4), p.ink);
    c.rect(x - 11, top + 3, 24, 3, p.foliage);
    c.rect(x - 8, top + 1, 18, 3, p.foliage);
    c.rect(x - 5, top, 12, 2, p.foliage_light);
    c.rect(x - 10, top + 3, 5, 1, p.foliage_light);
    c.line((x, top + 17), (x - 5, top + 13), p.ink);
    c.rect(x - 7, top + 11, 7, 2, p.foliage);
}

fn ship(c: &mut Canvas<'_>, x: i32, p: &Palette) {
    c.polygon(&[(x, 36), (x + 69, 36), (x + 60, 43), (x + 7, 43)], p.ink);
    c.rect(x + 9, 40, 51, 2, p.roof);
    c.rect(x + 9, 29, 14, 7, p.wall);
    c.rect(x + 12, 26, 8, 3, p.trim);
    c.rect(x + 14, 23, 3, 3, p.ink);
    c.rect(x + 10, 30, 11, 2, p.water);
    for dx in [27, 40, 53] {
        c.rect(x + dx, 32, 11, 4, p.roof);
    }
    c.line((x + 44, 24), (x + 44, 31), p.ink);
    c.rect(x + 7, 44, 52, 1, p.water_light);
}

fn crane(c: &mut Canvas<'_>, x: i32, p: &Palette) {
    c.rect(x, 13, 3, 31, p.ink);
    c.line((x + 2, 29), (x + 14, 44), p.trim);
    c.line((x + 1, 14), (x + 35, 9), p.ink);
    c.line((x + 1, 20), (x + 35, 9), p.trim);
    c.line((x + 1, 13), (x + 1, 5), p.trim);
    c.line((x + 1, 5), (x + 35, 9), p.ink);
    c.line((x + 1, 5), (x - 11, 16), p.ink);
    c.rect(x - 12, 16, 13, 3, p.trim);
    c.rect(x + 2, 20, 6, 4, p.wall);
    c.rect(x + 3, 21, 3, 2, p.water);
    c.rect(x + 32, 10, 1, 18, p.ink);
    c.line((x + 32, 28), (x + 35, 29), p.ink);
}

fn bridge(c: &mut Canvas<'_>, x: i32, p: &Palette, detail: bool) {
    c.rect(x, 33, 320, 2, p.ridge_shadow);
    c.rect(x, 51, 320, 3, p.ink);
    c.rect(x, 62, 320, 2, p.ink);
    for dx in (0..320).step_by(40) {
        c.rect(x + dx, 33, 2, 20, p.ink);
        c.line((x + dx + 2, 35), (x + dx + 38, 51), p.ridge_shadow);
        c.line((x + dx + 2, 51), (x + dx + 38, 35), p.ridge_shadow);
        c.line((x + dx, 64), (x + dx + 20, 69), p.trim);
        c.line((x + dx + 20, 69), (x + dx + 40, 64), p.trim);
        if detail {
            c.pixel(x + dx + 1, 34, p.wall);
        }
    }
}

fn viaduct(c: &mut Canvas<'_>, x: i32, p: &Palette) {
    // Perspective segments recede toward the right; the playable rail is flat.
    for i in 0..9 {
        let left = x + i * 23;
        let y = 39 - i / 2;
        c.rect(left, y, 24, 3, p.wall);
        c.rect(left, y + 3, 3, 15, p.ridge_shadow);
        c.line((left + 3, y + 9), (left + 8, y + 4), p.trim);
        c.line((left + 17, y + 4), (left + 23, y + 9), p.trim);
        c.rect(left, y - 2, 1, 2, p.ink);
        c.line((left, y - 2), (left + 23, y - 2), p.ridge_shadow);
    }
}

fn tunnel(c: &mut Canvas<'_>, x: i32, p: &Palette, detail: bool) {
    c.polygon(
        &[
            (x - 17, 55),
            (x - 13, 38),
            (x - 5, 25),
            (x + 11, 19),
            (x + 33, 24),
            (x + 47, 41),
            (x + 52, 55),
        ],
        p.ridge_shadow,
    );
    c.rect(x - 3, 35, 36, 20, p.wall);
    c.circle(x + 15, 35, 18, p.wall);
    c.circle(x + 15, 35, 13, p.ink);
    c.rect(x + 2, 35, 26, 20, p.ink);
    c.rect(x - 3, 36, 4, 19, p.trim);
    c.rect(x + 29, 36, 4, 19, p.trim);
    c.rect(x + 12, 17, 6, 6, p.trim);
    if detail {
        for y in (38..55).step_by(5) {
            c.rect(x - 3, y, 4, 1, p.ridge_shadow);
            c.rect(x + 29, y, 4, 1, p.ridge_shadow);
        }
        c.line((x + 2, 23), (x + 6, 27), p.ridge_shadow);
        c.line((x + 28, 23), (x + 24, 27), p.ridge_shadow);
        broadleaf(c, x - 8, 36, 10, p, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{Biome, SceneOptions};
    use eframe::egui::Color32;

    #[test]
    fn periodos_e_recorte_independem_da_largura() {
        let tags = SegmentScenery::from_tags(&[
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
        ]);
        let p = Palette::for_biome(Biome::Forest);
        let view = View {
            settlement: super::super::SettlementContext::default(),
            scenery_life: super::super::SceneryLifeOptions::default(),
            wind_speed_kmh: None,
            precipitation_mm: None,
            distance: 0.0,
            seconds: 0.0,
            light_seconds: 180.0,
            options: SceneOptions::default(),
            quality: RenderQuality::Full,
        };
        for distance in [0.0, 127.25, 13_527.75] {
            let mut narrow = vec![Color32::BLACK; 320 * 70];
            draw(
                &mut Canvas::new(&mut narrow),
                &p,
                View { distance, ..view },
                tags,
            );
            for width in [160, 320, 641, 1920] {
                let mut pixels = vec![Color32::BLACK; width * 70];
                draw(
                    &mut Canvas::new(&mut pixels),
                    &p,
                    View { distance, ..view },
                    tags,
                );
                let expected = pixels.clone();
                pixels.fill(Color32::BLACK);
                draw(
                    &mut Canvas::new(&mut pixels),
                    &p,
                    View {
                        distance: distance + crate::simulation::SCROLL_PERIOD_PX,
                        ..view
                    },
                    tags,
                );
                assert_eq!(pixels, expected);
                for y in 0..70 {
                    let count = width.min(320);
                    assert_eq!(
                        &pixels[y * width..y * width + count],
                        &narrow[y * 320..y * 320 + count]
                    );
                }
            }
        }
    }
}
