//! The app-facing renderer: one stable output buffer and one reusable transition
//! buffer. Scene state is a pure function of World plus optional presentation
//! controls. No I/O, metrics sampling, clock reads, or allocation per render.

use eframe::egui::Color32;

use super::{
    legacy,
    palette::{Palette, dusk_amount},
    rolling_stock::{CoachStyle, draw_train},
    scenery::{self, View},
    sprites::Canvas,
    stations::{self, StationService},
    theme::{
        RenderQuality, SceneContext, SceneOptions, SceneTheme, Weather, finite_nonnegative,
        route_at,
    },
};
use crate::simulation::{Train, World};

/// Native art resolution; the existing app uploads this exact 320 x 70 texture.
pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 70;
pub const DISPLAY_WIDTH: usize = WIDTH * 2;
pub const DISPLAY_HEIGHT: usize = HEIGHT * 2;
pub const FRAMEBUFFER_BYTES: usize = WIDTH * HEIGHT * size_of::<Color32>();

pub struct SceneRenderer {
    livery: crate::simulation::objectives::Livery,
    pixels: Vec<Color32>,
    transition: Vec<Color32>,
    theme: SceneTheme,
    quality: RenderQuality,
    options: SceneOptions,
    context: Option<SceneContext>,
    local_atmosphere: Option<(Weather, bool)>,
    coaches: [CoachStyle; 3],
    station_service: Option<StationService>,
}

impl Default for SceneRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            livery: crate::simulation::objectives::Livery::default(),
            pixels: vec![Color32::BLACK; WIDTH * HEIGHT],
            transition: vec![Color32::BLACK; WIDTH * HEIGHT],
            theme: SceneTheme::Journey,
            quality: RenderQuality::Full,
            options: SceneOptions::default(),
            context: None,
            local_atmosphere: None,
            station_service: None,
            coaches: [
                CoachStyle::Observation,
                CoachStyle::Sleeper,
                CoachStyle::Freight,
            ],
        }
    }

    /// Journey works immediately with the existing app. Legacy ignores every
    /// optional control and reproduces the original pastoral renderer exactly.
    pub fn set_theme(&mut self, theme: SceneTheme) {
        self.theme = theme;
    }

    /// Local conditions are supplied by the app; animation time stays independent.
    pub fn set_local_atmosphere(&mut self, weather: Option<(Weather, bool)>) {
        self.local_atmosphere = weather;
    }

    pub fn set_quality(&mut self, quality: RenderQuality) {
        self.quality = quality;
    }

    pub fn set_options(&mut self, options: SceneOptions) {
        self.options = options;
    }

    /// An optional Agent 03 adapter; None restores the automatic distance route.
    /// No new fields or responsibilities are required from the baseline World.
    pub fn set_scene_context(&mut self, context: Option<SceneContext>) {
        self.context = context.map(SceneContext::normalized);
    }

    pub fn set_coaches(&mut self, coaches: [CoachStyle; 3]) {
        self.coaches = coaches;
    }

    /// Optional real passenger service; keep the last arrival's value through
    /// departure. Zero counts draw no passengers. None retains the legacy
    /// illustrative service for exports/adapters, not product passenger data.
    pub fn set_station_service(&mut self, service: Option<StationService>) {
        self.station_service = service.map(StationService::normalized);
    }

    pub fn set_livery(&mut self, livery: crate::simulation::objectives::Livery) {
        self.livery = livery;
    }

    #[must_use]
    pub fn scene_context(&self, world: &World) -> SceneContext {
        self.context.unwrap_or_else(|| route_at(world.scroll_px()))
    }

    /// Reuses both preallocated buffers and repaints every output pixel.
    /// The egui owner still controls upload cadence and nearest-neighbor scaling.
    pub fn render(&mut self, train: &Train, world: &World) -> &[Color32] {
        if self.theme == SceneTheme::Legacy {
            legacy::draw(&mut self.pixels, train, world);
            return &self.pixels;
        }

        let context = self.scene_context(world);
        let mut options = self.options;
        let light_seconds = if let Some((weather, is_day)) = self.local_atmosphere {
            options.weather = weather;
            options.day_cycle = true;
            options.clouds = weather != Weather::Clear;
            if is_day { 180.0 } else { 480.0 }
        } else {
            finite_nonnegative(world.elapsed_seconds())
        };
        if self.quality == RenderQuality::Low {
            options.weather = Weather::Clear;
        }
        let view = View {
            distance: finite_nonnegative(world.scroll_px()),
            seconds: finite_nonnegative(world.elapsed_seconds()),
            options,
            light_seconds,
            quality: self.quality,
        };
        let weight = (context.blend * 256.0).round() as u32;
        if weight >= 256 {
            scenery::background(&mut self.pixels, context.next_biome, view);
        } else {
            scenery::background(&mut self.pixels, context.biome, view);
            if weight > 0 && context.biome != context.next_biome {
                scenery::background(&mut self.transition, context.next_biome, view);
                blend_backgrounds(&mut self.pixels, &self.transition, weight);
            }
        }

        let mut canvas = Canvas::new(&mut self.pixels);
        stations::draw(&mut canvas, world, self.station_service, view);
        scenery::weather(&mut canvas, view);
        draw_smoke(&mut canvas, world, self.quality == RenderQuality::Full);
        draw_train(
            &mut canvas,
            train.wheel_phase(),
            &self.coaches,
            dusk_amount(view.light_seconds, view.options.day_cycle) >= 0.35,
            self.options.details && self.quality == RenderQuality::Full,
        );
        use crate::simulation::objectives::Livery;
        let colors = match self.livery {
            Livery::Aurora => None,
            Livery::Ocean => Some(([47, 104, 148], [94, 165, 190], [28, 64, 105])),
            Livery::Sunset => Some(([163, 78, 51], [219, 142, 79], [111, 51, 42])),
            Livery::Midnight => Some(([79, 65, 122], [137, 112, 170], [48, 42, 78])),
        };
        if let Some((body, light, shadow)) = colors {
            for y in 41..51 {
                for x in 199..227 {
                    let pixel = &mut self.pixels[y * WIDTH + x];
                    for (from, to) in [
                        ([42, 105, 88], body),
                        ([83, 143, 106], light),
                        ([28, 79, 70], shadow),
                    ] {
                        if *pixel == Color32::from_rgb(from[0], from[1], from[2]) {
                            *pixel = Color32::from_rgb(to[0], to[1], to[2]);
                            break;
                        }
                    }
                }
            }
        }
        &self.pixels
    }
}

fn blend_backgrounds(destination: &mut [Color32], source: &[Color32], weight: u32) {
    let inverse = 256 - weight;
    for (left, right) in destination.iter_mut().zip(source) {
        let channel =
            |a: u8, b: u8| ((u32::from(a) * inverse + u32::from(b) * weight + 128) >> 8) as u8;
        *left = Color32::from_rgb(
            channel(left.r(), right.r()),
            channel(left.g(), right.g()),
            channel(left.b(), right.b()),
        );
    }
}

fn draw_smoke(canvas: &mut Canvas<'_>, world: &World, detailed: bool) {
    for puff in world.smoke() {
        if ![puff.x, puff.y, puff.age, puff.lifetime, puff.size]
            .iter()
            .all(|v| v.is_finite())
            || puff.lifetime <= 0.0
        {
            continue;
        }
        let life = (1.0 - puff.age / puff.lifetime).clamp(0.0, 1.0);
        let px = 220.0 + (puff.x * 0.5).round();
        let py = 32.0 + (puff.y * 0.5).round();
        let radius = (puff.size * 0.5).round().clamp(1.0, 6.0) as i32;
        // Fully off-screen particles are harmless and cost no pixel iteration.
        if !(-6.0..=326.0).contains(&px) || !(-6.0..=76.0).contains(&py) {
            continue;
        }
        let (x, y) = (px as i32, py as i32);
        canvas.smoke_circle(x, y, radius, (life * 205.0) as u8);
        if detailed {
            let lobe = (radius / 2).max(1);
            canvas.smoke_circle(x - radius / 3, y - radius / 3, lobe, (life * 92.0) as u8);
            canvas.smoke_circle(x + radius / 3, y, lobe, (life * 45.0) as u8);
        }
    }
}
