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
/// Limites físicos suportados pela faixa: 160 a 32768 colunas, sempre 70 linhas.
pub const MIN_WIDTH: usize = 160;
pub const MAX_WIDTH: usize = 32768;
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
    scenery: super::SceneryContext,
    scenery_life: super::SceneryLifeOptions,
    local_atmosphere: Option<(Weather, bool)>,
    local_lighting: Option<f64>,
    weather_wind: Option<f32>,
    weather_precipitation: Option<f32>,
    coaches: [CoachStyle; 3],
    station_service: Option<StationService>,
    station_styles: super::StationStyles,
    passenger_options: super::PassengerOptions,
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
            scenery: super::SceneryContext::default(),
            scenery_life: super::SceneryLifeOptions::default(),
            local_atmosphere: None,
            local_lighting: None,
            weather_wind: None,
            weather_precipitation: None,
            station_service: None,
            station_styles: super::StationStyles::default(),
            passenger_options: super::PassengerOptions::default(),
            engine: EngineStyle::default(),
            coaches: [
                CoachStyle::Observation,
                CoachStyle::Sleeper,
                CoachStyle::Freight,
            ],
        }
    }

    /// Realoca somente quando a largura muda; render() não aloca.
    pub fn set_width(&mut self, width: usize) -> bool {
        let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        if width == self.width() {
            return false;
        }
        self.pixels.resize(width * HEIGHT, Color32::BLACK);
        self.transition.resize(width * HEIGHT, Color32::BLACK);
        true
    }

    pub fn width(&self) -> usize {
        self.pixels.len() / HEIGHT
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

    /// Explicit opt-in from the route adapter. Reset to Default for Aurora and
    /// excursions without segment tags. No catalog lookup or heap allocation.
    pub fn set_scenery_context(&mut self, scenery: super::SceneryContext) {
        self.scenery = scenery;
    }

    /// Freezes only the tagged scenery's decorative motion. Route scrolling,
    /// lighting and the simulation remain owned by their existing adapters.
    pub fn set_scenery_life_options(&mut self, options: super::SceneryLifeOptions) {
        self.scenery_life = options;
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

    /// Route-selected architecture; the default retains Aurora and excursions.
    pub fn set_station_styles(&mut self, styles: super::StationStyles) {
        self.station_styles = styles;
    }

    /// Pure presentation controls; the application adapter owns preferences.
    /// Static poses preserve real counts and the start/end of each service.
    pub fn set_passenger_options(&mut self, options: super::PassengerOptions) {
        self.passenger_options = options;
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
        let passenger_weather = options.weather;
        if self.quality == RenderQuality::Low {
            options.weather = Weather::Clear;
        }
        let view = View {
            scenery_life: self.scenery_life,
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
            scenery::background(
                &mut self.pixels,
                context.next_biome,
                view,
                self.scenery.next,
            );
        } else {
            scenery::background(&mut self.pixels, context.biome, view, self.scenery.current);
            if weight > 0
                && (context.biome != context.next_biome
                    || self.scenery.current != self.scenery.next)
            {
                scenery::background(
                    &mut self.transition,
                    context.next_biome,
                    view,
                    self.scenery.next,
                );
                blend_backgrounds(&mut self.pixels, &self.transition, weight);
            }
        }

        let width = self.width();
        let mut canvas = Canvas::new(&mut self.pixels);
        stations::draw_styled(
            &mut canvas,
            world,
            self.station_service,
            view,
            self.station_styles,
            (self.passenger_options, passenger_weather),
        );
        scenery::weather(&mut canvas, view);
        let origin = canvas.train_origin();
        canvas.origin = origin;
        if self.engine.emits_smoke() {
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
            Livery::Cherry => Some(([149, 52, 76], [208, 108, 115], [89, 37, 60])),
            Livery::Ivory => Some(([213, 202, 165], [245, 232, 197], [143, 132, 107])),
            Livery::Lagoon => Some(([36, 128, 139], [102, 187, 181], [27, 77, 94])),
            Livery::Heather => Some(([128, 78, 130], [191, 139, 184], [77, 48, 88])),
            Livery::Sunflower => Some(([196, 148, 44], [239, 203, 104], [121, 83, 37])),
            Livery::Slate => Some(([79, 99, 113], [144, 165, 176], [43, 59, 76])),
        };
        if let Some((body, light, shadow)) = colors {
            let (left, top, right) = if self.engine == EngineStyle::Steam {
                (199, 41, 227)
            } else {
                (184, 34, 235)
            };
            for y in top..51 {
                for x in left..right {
                    let x = x + origin;
                    if !(0..width as i32).contains(&x) {
                        continue;
                    }
                    let pixel = &mut self.pixels[y * width + x as usize];
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

#[cfg(test)]
mod width_tests {
    use super::*;
    #[test]
    fn pontes_e_tuneis_preservam_todos_os_pixels_do_trem_e_trilhos() {
        use crate::rendering::{SceneryContext, SegmentScenery};
        use crate::simulation::catalog::SceneryTag as T;
        let train = Train::new(&crate::config::Config::default());
        let world = World::default();
        let mut renderer = SceneRenderer::new();
        renderer.set_local_lighting(Some(180.0));
        renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[
            T::Bridge,
            T::Tunnel,
            T::Viaduct,
            T::Mountain,
            T::Forest,
        ])));
        for width in [160, 320, 641] {
            renderer.set_width(width);
            for engine in EngineStyle::ALL {
                renderer.set_engine(engine);
                let mut train_pixels = vec![Color32::TRANSPARENT; width * HEIGHT];
                let mut c = Canvas::new(&mut train_pixels);
                c.origin = c.train_origin();
                draw_train(
                    &mut c,
                    train.wheel_phase(),
                    &renderer.coaches,
                    engine,
                    false,
                    true,
                );
                let frame = renderer.render(&train, &world);
                for (expected, actual) in train_pixels.iter().zip(frame) {
                    if expected.a() == 255 {
                        assert_eq!(expected, actual);
                    }
                }
                let rail = frame[57 * width..61 * width].to_vec();
                renderer.set_scenery_context(SceneryContext::default());
                assert_eq!(
                    rail,
                    renderer.render(&train, &world)[57 * width..61 * width]
                );
                renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[
                    T::Bridge,
                    T::Tunnel,
                    T::Viaduct,
                    T::Mountain,
                    T::Forest,
                ])));
            }
        }
    }

    #[test]
    fn largura_variavel_cobre_superficie_e_reutiliza_buffers() {
        let train = Train::new(&crate::config::Config::default());
        let world = World::default();
        let mut renderer = SceneRenderer::new();
        for width in [160, 241, 320, 641, 1920, MAX_WIDTH] {
            renderer.set_width(width);
            for theme in [SceneTheme::Journey, SceneTheme::Legacy] {
                renderer.set_theme(theme);
                renderer.pixels.fill(Color32::TRANSPARENT);
                let expected = renderer.render(&train, &world).to_vec();
                assert_eq!(expected.len(), width * HEIGHT);
                assert!(expected.iter().all(|p| p.a() == 255));
                let pointer = renderer.pixels.as_ptr();
                assert!(!renderer.set_width(width));
                assert_eq!(expected, renderer.render(&train, &world));
                assert_eq!(pointer, renderer.pixels.as_ptr());
                // Trilho e rodas continuam na mesma linha, inclusive na última coluna.
                assert_eq!(expected[58 * width], expected[58 * width + width - 1]);
            }
        }
        renderer.set_width(usize::MAX);
        assert_eq!(renderer.width(), MAX_WIDTH);
        renderer.set_width(0);
        assert_eq!(renderer.width(), MIN_WIDTH);
    }

    #[test]
    fn largura_nao_muda_periodo_do_parallax() {
        let train = Train::new(&crate::config::Config::default());
        let mut before = World::default();
        let mut remaining = crate::simulation::SCROLL_PERIOD_PX - 0.001;
        while remaining > 0.0 {
            let step = remaining.min(200.0);
            before.update(1.0, step, 0.0);
            remaining -= step;
        }
        let mut after = before.clone();
        after.update(0.01, 0.002, 0.0);
        let mut renderer = SceneRenderer::new();
        renderer.set_options(SceneOptions {
            day_cycle: false,
            clouds: false,
            ..Default::default()
        });
        for width in [160, 320, 641, 1920] {
            renderer.set_width(width);
            let old = renderer.render(&train, &before).to_vec();
            let new = renderer.render(&train, &after);
            let changed = old.iter().zip(new).filter(|(a, b)| a != b).count();
            assert!((changed as f64 / old.len() as f64) < 0.12);
        }
    }
    #[test]
    fn framebuffer_320_regression() {
        let config = crate::config::Config::default();
        let mut train = Train::new(&config);
        let mut world = World::default();
        for _ in 0..150 {
            let distance = train.update(50.0, 1.0 / 30.0, &config);
            world.update(1.0 / 30.0, distance, train.speed_kmh());
        }
        let mut renderer = SceneRenderer::new();
        let mut hashes = Vec::new();
        for biome in [
            super::super::Biome::GreenValley,
            super::super::Biome::Forest,
            super::super::Biome::Coast,
            super::super::Biome::HighDesert,
        ] {
            renderer.set_scene_context(Some(SceneContext {
                biome,
                next_biome: super::super::Biome::Coast,
                blend: 0.35,
            }));
            for weather in [Weather::Clear, Weather::Rain, Weather::Snow] {
                renderer.set_options(SceneOptions {
                    weather,
                    ..Default::default()
                });
                let hash = renderer
                    .render(&train, &world)
                    .iter()
                    .flat_map(|p| p.to_array())
                    .fold(0xcbf29ce484222325u64, |hash, b| {
                        (hash ^ u64::from(b)).wrapping_mul(0x100000001b3)
                    });
                hashes.push(hash);
            }
        }
        // Capturado nesta fonte antes da ampliação, em 14/11/2026: RGBA, quatro biomas e três climas.
        assert_eq!(
            hashes,
            [
                14127660415368685819,
                7698616192085357368,
                10139218686624518365,
                11229319816990774796,
                4245386398267801574,
                15520713772166973552,
                1398162178872592279,
                16194461996338019282,
                16327081667931027787,
                2116709937521495873,
                3848016047260177537,
                13027917757725326014
            ]
        );
    }
}
