//! Original station architecture and active-time platform animation.

use super::{palette::{Palette, dusk_amount}, scenery::View, sprites::Canvas, theme::{Biome, RenderQuality}};

fn draw_station(
    canvas: &mut Canvas<'_>,
    x: i32,
    station: usize,
    wait: f64,
    biome: super::Biome,
    view: View,
) {
    use super::Biome;
    let p = Palette::at_time(biome, view.light_seconds, view.options.day_cycle);
    let ink = p.ink;
    let cream = p.wall;
    let roof = p.roof;
    let detailed = view.options.details && view.quality == RenderQuality::Full;
    // Platform remains on the established rail row; roof silhouettes vary by place.
    canvas.rect(x - 7, 54, 79, 3, ink);
    canvas.rect(x - 7, 54, 79, 1, p.trim);
    canvas.rect(x, 32, 48, 22, ink);
    canvas.rect(x + 1, 33, 46, 20, cream);
    canvas.rect(x + 39, 33, 8, 20, p.trim);
    canvas.rect(x + 1, 50, 46, 3, p.roof);
    match biome {
        Biome::HighDesert => {
            // Adobe parapet and a raised bell gable.
            canvas.rect(x - 2, 29, 52, 4, roof);
            canvas.rect(x - 1, 29, 50, 1, p.trim);
            canvas.rect(x + 16, 24, 16, 6, cream);
            canvas.rect(x + 15, 24, 18, 2, roof);
            canvas.rect(x + 22, 26, 4, 4, ink);
            canvas.pixel(x + 23, 28, p.sun);
        }
        _ => {
            let peak = if biome == Biome::Forest { 20 } else { 23 };
            canvas.polygon(
                &[(x - 4, 33), (x + 10, peak), (x + 36, peak), (x + 52, 33)],
                ink,
            );
            canvas.polygon(
                &[
                    (x - 2, 32),
                    (x + 10, peak + 1),
                    (x + 36, peak + 1),
                    (x + 49, 32),
                ],
                roof,
            );
            canvas.line((x + 10, peak + 1), (x + 36, peak + 1), p.trim);
            if detailed {
                canvas.line((x + 5, 28), (x + 42, 28), p.trim);
                canvas.rect(x + 38, peak - 2, 4, 7, ink);
                canvas.rect(x + 38, peak - 2, 3, 5, roof);
            }
        }
    }
    // Recessed windows and doorway share a consistent light direction.
    let glass = if dusk_amount(view.light_seconds, view.options.day_cycle) >= 0.35 {
        p.sun
    } else {
        p.water_light
    };
    for dx in [4, 32] {
        canvas.rect(x + dx, 37, 10, 10, ink);
        canvas.rect(x + dx + 1, 38, 8, 7, glass);
        canvas.rect(x + dx + 5, 38, 1, 7, p.trim);
        canvas.rect(x + dx, 42, 10, 1, p.trim);
        canvas.rect(x + dx - 1, 47, 12, 1, p.trim);
    }
    canvas.rect(x + 19, 39, 9, 15, ink);
    canvas.rect(x + 20, 40, 7, 12, roof);
    canvas.rect(x + 21, 41, 5, 5, glass);
    canvas.pixel(x + 25, 49, p.trim);
    canvas.rect(x + 17, 53, 13, 1, cream);
    // Canopy, braces and sign stay readable at native scale.
    canvas.rect(x - 4, 35, 56, 2, ink);
    canvas.rect(x - 4, 35, 56, 1, roof);
    for dx in [-2, 49] {
        canvas.rect(x + dx, 37, 1, 17, ink);
        canvas.line(
            (x + dx, 41),
            (x + dx + if dx < 0 { 4 } else { -4 }, 37),
            ink,
        );
    }
    canvas.rect(x + 16, 32, 15, 4, ink);
    canvas.rect(x + 18, 33, 11, 1, p.trim);
    if detailed {
        // A tiny station ordinal plaque, no illegible fake text.
        for dx in 0..=station % 4 {
            canvas.pixel(x + 20 + dx as i32 * 2, 34, cream);
        }
        match biome {
            Biome::Forest => {
                for dx in [2, 16, 30, 44] {
                    canvas.rect(x + dx, 37, 1, 16, roof);
                }
                canvas.rect(x + 53, 50, 12, 3, roof);
                for dx in [55, 59, 63] {
                    canvas.pixel(x + dx, 51, p.trim);
                }
            }
            Biome::Coast => {
                canvas.rect(x + 54, 37, 1, 17, ink);
                canvas.polygon(&[(x + 55, 37), (x + 64, 39), (x + 55, 42)], p.wall);
                canvas.rect(x + 53, 51, 8, 3, roof);
                canvas.rect(x + 55, 50, 4, 1, p.trim);
            }
            Biome::HighDesert => {
                canvas.rect(x + 53, 48, 6, 6, roof);
                canvas.rect(x + 54, 47, 4, 1, p.trim);
                canvas.rect(x + 56, 43, 1, 5, p.foliage);
                canvas.rect(x + 54, 44, 4, 1, p.foliage);
            }
            Biome::GreenValley => {
                for dx in [4, 33] {
                    canvas.rect(x + dx, 48, 9, 2, roof);
                    canvas.rect(x + dx + 1, 47, 7, 1, p.foliage);
                    canvas.pixel(x + dx + 2, 47, p.sun);
                }
                canvas.rect(x + 54, 48, 11, 2, roof);
                for dx in [55, 63] {
                    canvas.rect(x + dx, 50, 1, 4, ink);
                }
            }
        }
    }
    if wait > 0.0 {
        let step = ((8.0 - wait) * 4.0) as i32;
        for index in 0..3 {
            let px = x - 5 + index * 7 - ((8.0 - wait) * 2.0) as i32;
            canvas.rect(px, 46, 2, 2, cream);
            canvas.rect(px - 1, 48, 4, 4, if index == 1 { p.water } else { roof });
            let stride = if step % 2 == 0 { 0 } else { 1 };
            canvas.rect(px - stride, 52, 1, 2, ink);
            canvas.rect(px + 2 + stride, 52, 1, 2, ink);
        }
    }
}

