//! Small original drawing primitives and train sprites.
//!
//! All geometry and palettes were authored for CPU Express. No Tiny Rails,
//! desktop-pet, WindowPet, or other third-party artwork is included.

use eframe::egui::Color32;

pub(super) const LOGICAL_HEIGHT: i32 = 70;

pub(super) const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Each cell is stored once. The texture is enlarged by the UI with nearest
/// filtering, so a cell still occupies a crisp 2×2 block in a 640×140 window.
/// Clipping keeps partially visible scenery and wraparound drawing simple.
pub(super) struct Canvas<'a> {
    pixels: &'a mut [Color32],
    width: i32,
    clip_rows: (i32, i32),
    pub(super) origin: i32,
}

impl<'a> Canvas<'a> {
    pub(super) fn new(pixels: &'a mut [Color32]) -> Self {
        let width = (pixels.len() / LOGICAL_HEIGHT as usize) as i32;
        Self {
            pixels,
            width,
            clip_rows: (0, LOGICAL_HEIGHT),
            origin: 0,
        }
    }

    pub(super) fn width(&self) -> i32 {
        self.width
    }

    pub(super) fn tiles(&self, period: i32) -> std::iter::StepBy<std::ops::RangeInclusive<i32>> {
        (-period..=((self.width + period - 1) / period) * period).step_by(period as usize)
    }

    pub(super) fn train_origin(&self) -> i32 {
        ((self.width - 320) / 2).min(0)
    }

    pub(super) fn clip_rows(&mut self, top: i32, bottom: i32) -> (i32, i32) {
        let old = self.clip_rows;
        self.clip_rows = (
            top.clamp(0, LOGICAL_HEIGHT),
            bottom.clamp(top.clamp(0, LOGICAL_HEIGHT), LOGICAL_HEIGHT),
        );
        old
    }

    pub(super) fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color32) {
        let x = x + self.origin;
        let left = x.clamp(0, self.width) as usize;
        let right = (x + width).clamp(0, self.width) as usize;
        let top = y.clamp(self.clip_rows.0, self.clip_rows.1) as usize;
        let bottom = (y + height).clamp(self.clip_rows.0, self.clip_rows.1) as usize;
        if left >= right || top >= bottom {
            return;
        }
        for row in top..bottom {
            self.pixels[row * self.width as usize + left..row * self.width as usize + right]
                .fill(color);
        }
    }

    pub(super) fn pixel(&mut self, x: i32, y: i32, color: Color32) {
        self.rect(x, y, 1, 1, color);
    }

    pub(super) fn line(&mut self, start: (i32, i32), end: (i32, i32), color: Color32) {
        let (mut x, mut y) = start;
        let (end_x, end_y) = end;
        let dx = (end_x - x).abs();
        let sx = if x < end_x { 1 } else { -1 };
        let dy = -(end_y - y).abs();
        let sy = if y < end_y { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.pixel(x, y, color);
            if x == end_x && y == end_y {
                break;
            }
            let doubled = 2 * error;
            if doubled >= dy {
                error += dy;
                x += sx;
            }
            if doubled <= dx {
                error += dx;
                y += sy;
            }
        }
    }

    pub(super) fn circle(&mut self, x: i32, y: i32, radius: i32, color: Color32) {
        for row in -radius..=radius {
            for col in -radius..=radius {
                if col * col + row * row <= radius * radius {
                    self.pixel(x + col, y + row, color);
                }
            }
        }
    }

    pub(super) fn smoke_circle(&mut self, x: i32, y: i32, radius: i32, alpha: u8) {
        let source = rgb(255, 245, 216);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let px = x + dx + self.origin;
                let py = y + dy;
                if dx * dx + dy * dy > radius * radius
                    || !(0..self.width).contains(&px)
                    || !(0..LOGICAL_HEIGHT).contains(&py)
                {
                    continue;
                }
                let base = self.pixels[py as usize * self.width as usize + px as usize];
                let blend = |from: u8, to: u8| {
                    ((u32::from(from) * (255 - u32::from(alpha))
                        + u32::from(to) * u32::from(alpha))
                        / 255) as u8
                };
                self.pixel(
                    px - self.origin,
                    py,
                    rgb(
                        blend(base.r(), source.r()),
                        blend(base.g(), source.g()),
                        blend(base.b(), source.b()),
                    ),
                );
            }
        }
    }

    /// Scanline fill for the deliberately small polygonal mountain silhouettes.
    pub(super) fn polygon(&mut self, points: &[(i32, i32)], color: Color32) {
        if points.len() < 3 {
            return;
        }
        let bottom = points.iter().map(|point| point.1).max().unwrap_or(0);
        let top = points.iter().map(|point| point.1).min().unwrap_or(0);
        for y in top.max(0)..bottom.min(LOGICAL_HEIGHT) {
            let mut intersections = [0_i32; 32];
            let mut count = 0;
            for index in 0..points.len() {
                let (x1, y1) = points[index];
                let (x2, y2) = points[(index + 1) % points.len()];
                if ((y1 <= y && y2 > y) || (y2 <= y && y1 > y)) && count < intersections.len() {
                    intersections[count] = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                    count += 1;
                }
            }
            intersections[..count].sort_unstable();
            for pair in intersections[..count].as_chunks::<2>().0 {
                self.rect(pair[0], y, pair[1] - pair[0] + 1, 1, color);
            }
        }
    }
}

pub(super) fn cloud(canvas: &mut Canvas<'_>, x: i32, y: i32, small: bool) {
    let cream = rgb(253, 238, 205);
    let shade = rgb(224, 205, 176);
    let width = if small { 21 } else { 34 };
    canvas.rect(x, y + 4, width, 3, shade);
    canvas.rect(x + 2, y + 1, width - 5, 5, cream);
    canvas.rect(x + 6, y - 1, 9, 3, cream);
    if !small {
        canvas.rect(x + 16, y - 3, 10, 7, cream);
        canvas.rect(x + 24, y + 2, 12, 3, cream);
    }
    canvas.rect(x + 4, y + 6, width - 8, 1, rgb(240, 224, 193));
}

pub(super) fn pine(canvas: &mut Canvas<'_>, x: i32, base: i32, height: i32, dark: bool) {
    let shadow = if dark {
        rgb(43, 93, 78)
    } else {
        rgb(76, 126, 105)
    };
    let light = if dark {
        rgb(65, 118, 89)
    } else {
        rgb(105, 147, 115)
    };
    canvas.rect(x, base - height + 4, 2, height - 2, rgb(95, 105, 75));
    for layer in 0..3 {
        let top = base - height + layer * (height / 5);
        let half = 2 + layer * 2;
        let bottom = top + height / 2;
        canvas.polygon(
            &[(x, top), (x - half, bottom), (x + half + 1, bottom)],
            shadow,
        );
        canvas.line((x, top + 1), (x - half + 1, bottom - 1), light);
    }
}

pub(super) fn wheel(canvas: &mut Canvas<'_>, x: i32, y: i32, radius: i32, phase: f64) {
    canvas.circle(x, y, radius, rgb(39, 55, 53));
    canvas.circle(x, y, radius - 1, rgb(105, 124, 111));
    let arm = f64::from(radius - 1);
    let dx = (phase.cos() * arm).round() as i32;
    let dy = (phase.sin() * arm).round() as i32;
    canvas.line((x - dx, y - dy), (x + dx, y + dy), rgb(220, 202, 145));
    canvas.pixel(x, y, rgb(247, 216, 154));
}

pub(super) fn coach(canvas: &mut Canvas<'_>, x: i32, phase: f64, variant: usize) {
    let colors = [
        (rgb(147, 76, 62), rgb(186, 106, 73)),
        (rgb(176, 143, 89), rgb(207, 174, 114)),
        (rgb(67, 106, 91), rgb(97, 138, 106)),
    ];
    let (body, highlight) = colors[variant % colors.len()];
    let dark = rgb(47, 66, 59);
    let trim = rgb(230, 200, 133);
    canvas.rect(x - 2, 51, 42, 2, dark);
    canvas.rect(x, 38, 38, 13, dark);
    canvas.rect(x + 1, 39, 36, 11, body);
    canvas.rect(x + 2, 39, 34, 1, trim);
    canvas.rect(x + 2, 47, 34, 2, highlight);
    canvas.rect(x + 1, 49, 36, 1, trim);
    for column in 0..4 {
        let wx = x + 4 + column * 8;
        canvas.rect(wx, 41, 6, 6, dark);
        canvas.rect(wx + 1, 42, 4, 4, rgb(157, 183, 167));
        canvas.rect(wx + 1, 42, 4, 1, rgb(245, 221, 167));
        canvas.rect(wx + 3, 42, 1, 4, trim);
        canvas.pixel(wx + 1, 43, rgb(224, 221, 180));
    }
    canvas.rect(x - 1, 36, 40, 3, dark);
    canvas.rect(x + 1, 35, 36, 1, rgb(87, 94, 71));
    canvas.rect(x + 1, 36, 36, 1, rgb(201, 180, 124));
    canvas.rect(x + 1, 40, 1, 8, rgb(243, 211, 144));
    canvas.rect(x + 36, 40, 1, 8, rgb(118, 116, 80));
    wheel(canvas, x + 7, 54, 3, phase);
    wheel(canvas, x + 30, 54, 3, phase);
    canvas.rect(x + 4, 51, 7, 1, dark);
    canvas.rect(x + 27, 51, 7, 1, dark);
}

/// The stack mouth is (440, 64) in output pixels. Smoke particles supplied by
/// the simulation are offsets from that mouth, so rendering owns placement.
pub(super) fn locomotive(canvas: &mut Canvas<'_>, phase: f64) {
    let ink = rgb(37, 56, 52);
    let green = rgb(42, 105, 88);
    let light_green = rgb(83, 143, 106);
    let copper = rgb(174, 91, 57);
    let gold = rgb(239, 194, 112);
    canvas.rect(180, 51, 54, 2, ink);
    canvas.rect(184, 37, 16, 14, copper);
    canvas.rect(185, 38, 14, 12, rgb(138, 69, 48));
    canvas.rect(187, 39, 11, 8, gold);
    canvas.rect(188, 40, 9, 6, rgb(41, 78, 75));
    canvas.rect(189, 41, 3, 4, rgb(160, 185, 159));
    canvas.rect(194, 40, 1, 6, copper);
    canvas.pixel(196, 44, rgb(249, 222, 156));
    canvas.rect(183, 34, 19, 3, ink);
    canvas.rect(185, 34, 16, 1, rgb(148, 150, 108));
    canvas.rect(184, 37, 1, 13, gold);
    canvas.rect(185, 49, 14, 1, gold);
    canvas.circle(224, 46, 5, ink);
    canvas.rect(199, 41, 26, 11, ink);
    canvas.rect(200, 42, 26, 9, green);
    canvas.rect(202, 42, 23, 2, light_green);
    canvas.rect(201, 49, 24, 1, rgb(28, 79, 70));
    canvas.rect(205, 42, 1, 8, gold);
    canvas.rect(222, 42, 1, 8, gold);
    canvas.rect(202, 46, 23, 1, rgb(136, 160, 109));
    canvas.circle(208, 40, 3, copper);
    canvas.rect(206, 38, 5, 1, gold);
    canvas.rect(217, 33, 5, 9, ink);
    canvas.rect(218, 34, 3, 7, copper);
    canvas.rect(216, 31, 8, 3, ink);
    canvas.rect(217, 31, 6, 1, gold);
    canvas.rect(218, 38, 4, 1, gold);
    canvas.rect(227, 42, 3, 9, ink);
    canvas.rect(228, 44, 2, 4, rgb(80, 96, 83));
    canvas.circle(232, 43, 2, copper);
    canvas.pixel(232, 43, rgb(255, 235, 169));
    canvas.rect(230, 48, 3, 4, ink);
    canvas.polygon(&[(232, 50), (239, 56), (230, 56)], copper);
    canvas.line((232, 52), (236, 55), gold);
    canvas.rect(183, 53, 4, 1, gold);
    canvas.rect(184, 55, 3, 1, gold);
    for x in [201, 213, 225] {
        wheel(canvas, x, 54, 3, phase);
    }
    wheel(canvas, 233, 55, 2, phase);
    let crank_x = (phase.cos() * 2.0).round() as i32;
    let crank_y = (phase.sin() * 2.0).round() as i32;
    canvas.line(
        (201 + crank_x, 54 + crank_y),
        (225 + crank_x, 54 + crank_y),
        gold,
    );
    canvas.line((225 + crank_x, 54 + crank_y), (230, 50), rgb(222, 220, 167));
    canvas.pixel(213 + crank_x, 54 + crank_y, rgb(254, 231, 172));
}
