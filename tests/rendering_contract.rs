//! End-to-end checks of the same public renderer used by the application.
//! These tests never sample CPU or open a window.

use cpu_express::{
    config::Config,
    rendering::{
        BIOME_LENGTH_PX, Biome, CoachStyle, HEIGHT, ROUTE_PERIOD_PX, RenderQuality, SceneContext,
        SceneOptions, SceneRenderer, SceneTheme, TRANSITION_LENGTH_PX, WIDTH, Weather, route_at,
    },
    simulation::{SCROLL_PERIOD_PX, Train, World},
};
use eframe::egui::Color32;

const BIOMES: [Biome; 4] = [
    Biome::GreenValley,
    Biome::Forest,
    Biome::Coast,
    Biome::HighDesert,
];

fn fixture() -> (Train, World) {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..150 {
        let distance = train.update(50.0, 1.0 / 30.0, &config);
        world.update(1.0 / 30.0, distance, train.speed_kmh());
    }
    (train, world)
}

fn stationary_options() -> SceneOptions {
    SceneOptions {
        day_cycle: false,
        clouds: false,
        weather: Weather::Clear,
        details: true,
    }
}

fn fnv_rgb(pixels: &[Color32]) -> u64 {
    pixels.iter().fold(0xcbf2_9ce4_8422_2325, |hash, color| {
        [color.r(), color.g(), color.b()]
            .into_iter()
            .fold(hash, |value, byte| {
                (value ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
            })
    })
}

fn difference_fraction(a: &[Color32], b: &[Color32]) -> f64 {
    a.iter().zip(b).filter(|(a, b)| a != b).count() as f64 / a.len() as f64
}

#[test]
fn legacy_reproduces_original_frame_byte_for_byte() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_theme(SceneTheme::Legacy);
    // Captured from the untouched supplied ZIP: export_frame, CPU=50, 150
    // updates of 1/30 s. Hash covers the 320x70 native RGB, not a PNG encoding.
    assert_eq!(
        fnv_rgb(renderer.render(&train, &world)),
        0xb1bc_11a7_571f_5fc3
    );
}

#[test]
fn legacy_ignores_new_options_and_recovers_after_theme_switch() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_theme(SceneTheme::Legacy);
    let original = renderer.render(&train, &world).to_vec();
    renderer.set_theme(SceneTheme::Journey);
    renderer.set_quality(RenderQuality::Low);
    renderer.set_scene_context(Some(SceneContext {
        biome: Biome::Coast,
        next_biome: Biome::HighDesert,
        blend: 0.5,
    }));
    renderer.set_options(SceneOptions {
        day_cycle: true,
        clouds: true,
        weather: Weather::Rain,
        details: true,
    });
    renderer.set_coaches([
        CoachStyle::Freight,
        CoachStyle::Tank,
        CoachStyle::Observation,
    ]);
    assert_ne!(renderer.render(&train, &world), original.as_slice());
    renderer.set_theme(SceneTheme::Legacy);
    assert_eq!(renderer.render(&train, &world), original.as_slice());
}

#[test]
fn identical_state_produces_identical_frames_without_render_history() {
    let (train, world) = fixture();
    let mut first = SceneRenderer::new();
    let mut second = SceneRenderer::new();
    for quality in [RenderQuality::Full, RenderQuality::Low] {
        for weather in [Weather::Clear, Weather::Rain] {
            for biome in BIOMES {
                let options = SceneOptions {
                    weather,
                    ..SceneOptions::default()
                };
                first.set_options(options);
                second.set_options(options);
                first.set_quality(quality);
                second.set_quality(quality);
                first.set_scene_context(Some(SceneContext::new(biome)));
                second.set_scene_context(Some(SceneContext::new(biome)));
                let frame = first.render(&train, &world).to_vec();
                assert_eq!(frame, second.render(&train, &world));
                assert_eq!(frame, first.render(&train, &world));
            }
        }
    }
}

#[test]
fn buffer_shape_address_and_opacity_survive_all_modes() {
    assert_eq!((WIDTH, HEIGHT), (320, 70));
    assert_eq!(size_of::<Color32>(), 4);
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    let pointer = renderer.render(&train, &world).as_ptr();
    for theme in [SceneTheme::Journey, SceneTheme::Legacy] {
        renderer.set_theme(theme);
        for quality in [RenderQuality::Full, RenderQuality::Low] {
            renderer.set_quality(quality);
            for biome in BIOMES {
                for blend in [0.0, 0.01, 0.5, 0.99, 1.0] {
                    renderer.set_scene_context(Some(SceneContext {
                        biome,
                        next_biome: Biome::Coast,
                        blend,
                    }));
                    renderer.set_options(SceneOptions {
                        weather: Weather::Rain,
                        ..SceneOptions::default()
                    });
                    let pixels = renderer.render(&train, &world);
                    assert_eq!(pixels.as_ptr(), pointer);
                    assert_eq!(pixels.len(), WIDTH * HEIGHT);
                    assert!(pixels.iter().all(|color| color.a() == 255));
                }
            }
        }
    }
}

#[test]
fn biome_frames_are_visibly_distinct_beyond_the_palette_api() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(stationary_options());
    let frames: Vec<_> = BIOMES
        .into_iter()
        .map(|biome| {
            renderer.set_scene_context(Some(SceneContext::new(biome)));
            renderer.render(&train, &world).to_vec()
        })
        .collect();
    for left in 0..frames.len() {
        for right in left + 1..frames.len() {
            assert!(
                difference_fraction(&frames[left], &frames[right]) > 0.20,
                "biomes {left} and {right} should have visibly different scenery"
            );
        }
    }
}

#[test]
fn atmosphere_is_optional_and_low_quality_skips_rain_and_cloud_passes() {
    let config = Config::default();
    let train = Train::new(&config);
    let mut world = World::default();
    // The integrated World rejects sleep gaps. Reach this artistic time with
    // valid active steps, otherwise the old fixture silently stayed at t=0.
    for _ in 0..450 {
        world.update(1.0, 0.0, 0.0);
    }
    assert_eq!(world.elapsed_seconds(), 450.0);
    let mut renderer = SceneRenderer::new();
    renderer.set_scene_context(Some(SceneContext::new(Biome::Coast)));
    let mut options = stationary_options();
    renderer.set_options(options);
    let daylight = renderer.render(&train, &world).to_vec();
    options.day_cycle = true;
    renderer.set_options(options);
    let dusk = renderer.render(&train, &world).to_vec();
    assert_ne!(&daylight[..WIDTH], &dusk[..WIDTH]);
    options.clouds = true;
    renderer.set_options(options);
    let clouds = renderer.render(&train, &world).to_vec();
    assert_ne!(clouds, dusk);
    options.weather = Weather::Rain;
    renderer.set_options(options);
    assert_ne!(renderer.render(&train, &world), clouds);
    renderer.set_quality(RenderQuality::Low);
    let low_with_options = renderer.render(&train, &world).to_vec();
    options.clouds = false;
    options.weather = Weather::Clear;
    renderer.set_options(options);
    assert_eq!(renderer.render(&train, &world), low_with_options);
}

#[test]
fn transition_endpoints_reuse_the_same_scroll_and_train() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(stationary_options());
    for (biome, next_biome) in [
        (Biome::GreenValley, Biome::Coast),
        (Biome::GreenValley, Biome::Forest),
        (Biome::Forest, Biome::HighDesert),
        (Biome::Coast, Biome::HighDesert),
        (Biome::HighDesert, Biome::GreenValley),
    ] {
        renderer.set_scene_context(Some(SceneContext::new(biome)));
        let initial = renderer.render(&train, &world).to_vec();
        renderer.set_scene_context(Some(SceneContext::new(next_biome)));
        let final_frame = renderer.render(&train, &world).to_vec();
        for (blend, expected) in [(0.0, &initial), (1.0, &final_frame)] {
            renderer.set_scene_context(Some(SceneContext {
                biome,
                next_biome,
                blend,
            }));
            assert_eq!(renderer.render(&train, &world), expected);
        }
        renderer.set_scene_context(Some(SceneContext {
            biome,
            next_biome,
            blend: 0.5,
        }));
        let middle = renderer.render(&train, &world);
        assert_ne!(middle, initial.as_slice());
        assert_ne!(middle, final_frame.as_slice());
        // A transition changes scenery, never the axle geometry. This box is
        // fully inside the opaque locomotive boiler, clear of smoke/foreground.
        for y in 43..49 {
            for x in 202..224 {
                let i = y * WIDTH + x;
                assert_eq!(middle[i], initial[i]);
                assert_eq!(middle[i], final_frame[i]);
            }
        }
    }
}

#[test]
fn invalid_context_blends_are_normalized_and_clamped() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(stationary_options());
    renderer.set_scene_context(Some(SceneContext::new(Biome::GreenValley)));
    let initial = renderer.render(&train, &world).to_vec();
    for blend in [f32::NAN, f32::NEG_INFINITY, f32::INFINITY, -2.0] {
        renderer.set_scene_context(Some(SceneContext {
            biome: Biome::GreenValley,
            next_biome: Biome::Coast,
            blend,
        }));
        assert_eq!(renderer.render(&train, &world), initial);
        assert!(renderer.scene_context(&world).blend.is_finite());
    }
    renderer.set_scene_context(Some(SceneContext::new(Biome::Coast)));
    let final_frame = renderer.render(&train, &world).to_vec();
    renderer.set_scene_context(Some(SceneContext {
        biome: Biome::GreenValley,
        next_biome: Biome::Coast,
        blend: 2.0,
    }));
    assert_eq!(renderer.render(&train, &world), final_frame);
    renderer.set_scene_context(None);
    assert_eq!(renderer.scene_context(&world), route_at(world.scroll_px()));
}

#[test]
fn invalid_simulation_inputs_and_extreme_finite_time_still_render() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(SceneOptions {
        weather: Weather::Rain,
        ..SceneOptions::default()
    });
    for cpu in [f32::NAN, f32::INFINITY, -100.0, 500.0, 50.0] {
        for dt in [f64::NAN, f64::INFINITY, -1.0, 0.0, 0.03, f64::MAX] {
            let distance = train.update(cpu, dt, &config);
            world.update(dt, distance, train.speed_kmh());
            let pixels = renderer.render(&train, &world);
            assert_eq!(pixels.len(), WIDTH * HEIGHT);
            assert!(pixels.iter().all(|color| color.a() == 255));
        }
    }
}

#[test]
fn route_cycles_compatibly_with_world_and_has_no_stateful_reset() {
    assert_eq!(SCROLL_PERIOD_PX.rem_euclid(ROUTE_PERIOD_PX), 0.0);
    assert_eq!(ROUTE_PERIOD_PX, BIOME_LENGTH_PX * 3.0);
    assert!((0.0..BIOME_LENGTH_PX).contains(&TRANSITION_LENGTH_PX));
    for distance in [0.0, 123.25, 1_900.0, 2_500.0, 4_400.0, 6_143.75] {
        assert_eq!(route_at(distance), route_at(distance + ROUTE_PERIOD_PX));
        assert_eq!(route_at(distance), route_at(distance + SCROLL_PERIOD_PX));
    }
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let context = route_at(invalid);
        assert!(context.blend.is_finite());
        assert!((0.0..=1.0).contains(&context.blend));
    }
    for index in 1..=3 {
        let end = index as f64 * BIOME_LENGTH_PX;
        let before = route_at(end - 0.001);
        let after = route_at(end);
        assert!(before.blend > 0.999);
        assert_eq!(before.next_biome, after.biome);
        assert_eq!(after.blend, 0.0);
    }
}

#[test]
fn frame_continuity_at_world_wrap_does_not_reset_parallax_offsets() {
    let config = Config::default();
    let train = Train::new(&config);
    let mut before_world = World::default();
    let mut remaining = SCROLL_PERIOD_PX - 0.001;
    while remaining > 0.0 {
        let distance = remaining.min(200.0);
        before_world.update(1.0, distance, 0.0);
        remaining -= distance;
    }
    assert!((before_world.scroll_px() - (SCROLL_PERIOD_PX - 0.001)).abs() < 1e-8);
    let mut after_world = before_world.clone();
    after_world.update(0.01, 0.002, 0.0);
    assert!((after_world.scroll_px() - 0.001).abs() < 1e-8);
    // Atmosphere is disabled. This is now a real wrap; the old huge-distance
    // fixture was rejected by World and therefore compared two origin frames.
    let mut renderer = SceneRenderer::new();
    renderer.set_options(stationary_options());
    let before = renderer.render(&train, &before_world).to_vec();
    let after = renderer.render(&train, &after_world);
    // Integer rasterization may move a contour by one pixel at the boundary.
    // A full scene restart would change a much larger fraction of the image.
    assert!(difference_fraction(&before, after) < 0.12);
}

#[test]
fn coach_variants_fit_inside_their_train_footprint() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(stationary_options());
    renderer.set_coaches([CoachStyle::Passenger; 3]);
    let passenger = renderer.render(&train, &world).to_vec();
    for style in [
        CoachStyle::Sleeper,
        CoachStyle::Freight,
        CoachStyle::Tank,
        CoachStyle::Observation,
    ] {
        renderer.set_coaches([style; 3]);
        let changed: Vec<_> = renderer
            .render(&train, &world)
            .iter()
            .zip(&passenger)
            .enumerate()
            .filter_map(|(i, (a, b))| (a != b).then_some((i % WIDTH, i / WIDTH)))
            .collect();
        assert!(
            changed.len() > 40,
            "variant {style:?} has no visible character"
        );
        assert!(
            changed
                .iter()
                .all(|&(x, y)| (55..=181).contains(&x) && (28..58).contains(&y))
        );
    }
}

#[test]
fn wheels_meet_the_same_rail_without_drawing_below_it() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(stationary_options());
    for biome in BIOMES {
        renderer.set_scene_context(Some(SceneContext::new(biome)));
        for style in [
            CoachStyle::Passenger,
            CoachStyle::Sleeper,
            CoachStyle::Freight,
            CoachStyle::Tank,
            CoachStyle::Observation,
        ] {
            renderer.set_coaches([style; 3]);
            let pixels = renderer.render(&train, &world);
            for x in [65, 88, 107, 130, 149, 172, 201, 213, 225, 233] {
                assert_ne!(pixels[57 * WIDTH + x], pixels[57 * WIDTH + 40]);
                assert_eq!(pixels[58 * WIDTH + x], pixels[58 * WIDTH + 40]);
            }
        }
    }
}

#[test]
fn local_weather_has_distinct_frames_and_keeps_train_readable() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    let mut hashes = std::collections::HashSet::new();
    for weather in [
        Weather::Clear,
        Weather::Cloudy,
        Weather::Fog,
        Weather::Rain,
        Weather::Thunderstorm,
        Weather::Snow,
    ] {
        for day in [true, false] {
            renderer.set_local_atmosphere(Some((weather, day)));
            let frame = renderer.render(&train, &world);
            assert!(
                hashes.insert(fnv_rgb(frame)),
                "duplicate atmosphere {weather:?}/{day}"
            );
            assert!(frame.iter().all(|pixel| pixel.a() == 255));
            // All weather remains behind the locomotive body.
            assert_eq!(frame[47 * WIDTH + 210], Color32::from_rgb(42, 105, 88));
        }
    }
}

#[test]
fn local_night_does_not_depend_on_simulated_day_cycle() {
    let (train, world) = fixture();
    let mut renderer = SceneRenderer::new();
    renderer.set_local_atmosphere(Some((Weather::Clear, false)));
    let first = renderer.render(&train, &world)[0];
    let mut later = World::default();
    later.update(180.0, 0.0, 0.0);
    assert_eq!(first, renderer.render(&train, &later)[0]);
    renderer.set_local_atmosphere(Some((Weather::Clear, true)));
    assert_ne!(first, renderer.render(&train, &later)[0]);
}
