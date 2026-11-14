//! The app-facing renderer: one stable output buffer and one reusable transition
//! buffer. Scene state is a pure function of World plus optional presentation
//! controls. No I/O, metrics sampling, clock reads, or allocation per render.

use eframe::egui::Color32;

use super::{
    EngineStyle, legacy,
    palette::dusk_amount,
    rolling_stock::{CoachStyle, draw_train},
    scenery::{self, View},
    sprites::Canvas,
    stations::{self, StationService},
    theme::{
        DAY_CYCLE_SECONDS, RenderQuality, SceneContext, SceneOptions, SceneTheme, Weather,
        finite_nonnegative, route_at,
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
    local_lighting: Option<f64>,
    weather_wind: Option<f32>,
    weather_precipitation: Option<f32>,
    coaches: [CoachStyle; 3],
    station_service: Option<StationService>,
    engine: EngineStyle,
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
            local_lighting: None,
            weather_wind: None,
            weather_precipitation: None,
            station_service: None,
            engine: EngineStyle::default(),
            coaches: [
                CoachStyle::Observation,
                CoachStyle::Sleeper,
                CoachStyle::Freight,
            ],
        }
    }

    /// Journey works immediately with the existing app. Legacy keeps the original
    /// pastoral frame with Steam; only engine selection also applies to Legacy.
    pub fn set_theme(&mut self, theme: SceneTheme) {
        self.theme = theme;
    }

    /// Local conditions are supplied by the app; animation time stays independent.
    pub fn set_local_atmosphere(&mut self, weather: Option<(Weather, bool)>) {
        self.local_atmosphere = weather;
    }

    /// Override only lighting, not animation time or weather. Finite phases wrap
    /// into [0, DAY_CYCLE_SECONDS); None/nonfinite values restore the atmosphere
    /// day/night fallback (180/480), or the simulation cycle when absent.
    /// Art anchors: dawn 0/600, daylight 180, sunset 348, night 450..540.
    pub fn set_local_lighting(&mut self, phase: Option<f64>) {
        self.local_lighting = phase
            .filter(|value| value.is_finite())
            .map(|value| value.rem_euclid(DAY_CYCLE_SECONDS));
    }

    /// Optional artistic detail, never a weather-condition inference. Precipitation
    /// is the model interval's amount in mm, NOT a rate in mm/h. Invalid values
    /// restore the old defaults; finite values clamp to 0..250 km/h and 0..100 mm.
    /// Wind bends/drifts precipitation only; no new particles or simulation work.
    pub fn set_weather_detail(
        &mut self,
        wind_speed_kmh: Option<f32>,
        precipitation_mm: Option<f32>,
    ) {
        self.weather_wind = wind_speed_kmh
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 250.0));
        self.weather_precipitation = precipitation_mm
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 100.0));
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

    /// Equip original art on the fixed engine socket. Only Steam draws smoke;
    /// simulation particle storage and train dynamics remain untouched.
    pub fn set_engine(&mut self, style: EngineStyle) {
        self.engine = style;
    }

    #[must_use]
    pub fn scene_context(&self, world: &World) -> SceneContext {
        if let Some(branch) = world.excursion_snapshot().current {
            use crate::simulation::BranchPhase;
            let destination = super::Biome::from(branch.kind.biome());
            let progress = finite_nonnegative(branch.progress()).clamp(0.0, 1.0);
            let (biome, next_biome, blend) = match branch.phase {
                BranchPhase::Outbound => (
                    super::Biome::GreenValley,
                    destination,
                    (progress * 4.0).min(1.0),
                ),
                BranchPhase::Service => (destination, destination, 0.0),
                BranchPhase::Return => (
                    destination,
                    super::Biome::GreenValley,
                    ((progress - 0.75) * 4.0).max(0.0),
                ),
            };
            return SceneContext {
                biome,
                next_biome,
                blend: (blend * blend * (3.0 - 2.0 * blend)) as f32,
            };
        }
        self.context.unwrap_or_else(|| route_at(world.scroll_px()))
    }

    /// Reuses both preallocated buffers and repaints every output pixel.
    /// The egui owner still controls upload cadence and nearest-neighbor scaling.
    pub fn render(&mut self, train: &Train, world: &World) -> &[Color32] {
        if self.theme == SceneTheme::Legacy {
            legacy::draw(&mut self.pixels, train, world, self.engine);
            return &self.pixels;
        }

        let context = self.scene_context(world);
        let mut options = self.options;
        let mut light_seconds = if let Some((weather, is_day)) = self.local_atmosphere {
            options.weather = weather;
            options.day_cycle = true;
            options.clouds = weather != Weather::Clear;
            if is_day { 180.0 } else { 480.0 }
        } else {
            finite_nonnegative(world.elapsed_seconds())
        };
        if let Some(phase) = self.local_lighting {
            options.day_cycle = true;
            light_seconds = phase;
        }
        if self.quality == RenderQuality::Low {
            options.weather = Weather::Clear;
        }
        let view = View {
            wind_speed_kmh: self.weather_wind,
            precipitation_mm: self.weather_precipitation,
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
        if self.engine == EngineStyle::Steam {
            draw_smoke(&mut canvas, world, self.quality == RenderQuality::Full);
        }
        draw_train(
            &mut canvas,
            train.wheel_phase(),
            &self.coaches,
            self.engine,
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
            let (left, top, right) = if self.engine == EngineStyle::Steam {
                (199, 41, 227)
            } else {
                (184, 34, 235)
            };
            for y in top..51 {
                for x in left..right {
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
