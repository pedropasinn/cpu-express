//! Count allocations only on the rendering test thread, after construction.
//! egui's texture staging and the test harness are deliberately outside this
//! measurement; this contract concerns SceneRenderer::render and its setters.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

use cpu_express::{
    config::Config,
    rendering::{
        Biome, CoachStyle, EngineStyle, RenderQuality, SceneContext, SceneOptions, SceneRenderer,
        SceneTheme, StationService, Weather,
    },
    simulation::{Train, World},
};

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct TrackingAllocator;

fn count_allocation() {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
    }
}

// SAFETY: Every allocation operation is forwarded with its original arguments
// to the System allocator. The thread-local counters contain only Cell values,
// never allocate, and do not modify or retain any allocated pointer.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        // SAFETY: The caller provides the layout required by GlobalAlloc.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        // SAFETY: Forwarding the caller's valid layout without changing it.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Pointer and layout are passed unchanged to their allocator.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count_allocation();
        // SAFETY: Pointer, layout and requested size retain the caller contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

#[test]
fn render_and_mode_changes_make_no_heap_allocations_after_construction() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..150 {
        let distance = train.update(50.0, 1.0 / 30.0, &config);
        world.update(1.0 / 30.0, distance, train.speed_kmh());
    }
    let mut renderer = SceneRenderer::new();
    let branch_worlds = [
        cpu_express::simulation::BranchKind::Serra,
        cpu_express::simulation::BranchKind::Litoral,
    ]
    .map(|kind| {
        let route = cpu_express::simulation::Route::default();
        let mut branch_world = World::default();
        branch_world.restore_journey(
            cpu_express::simulation::Journey::restore_checkpoint(
                route.seed(),
                route.length_units(),
                1000.0,
            )
            .unwrap(),
        );
        branch_world.request_excursion(kind).unwrap();
        let mut branch_train = Train::new(&config);
        for _ in 0..90 {
            branch_world.advance_expedition(&mut branch_train, 40.0, 1.0, &config);
        }
        branch_world
    });
    ALLOCATIONS.with(|count| count.set(0));
    COUNTING.with(|active| active.set(true));
    for theme in [SceneTheme::Journey, SceneTheme::Legacy] {
        renderer.set_theme(theme);
        for quality in [RenderQuality::Full, RenderQuality::Low] {
            renderer.set_quality(quality);
            for weather in [
                Weather::Clear,
                Weather::Cloudy,
                Weather::Fog,
                Weather::Rain,
                Weather::Thunderstorm,
                Weather::Snow,
            ] {
                renderer.set_local_atmosphere(Some((weather, false)));
                renderer.set_options(SceneOptions {
                    weather,
                    ..SceneOptions::default()
                });
                for blend in [0.0, 0.01, 0.5, 0.99, 1.0] {
                    renderer.set_scene_context(Some(SceneContext {
                        biome: Biome::Forest,
                        next_biome: Biome::Coast,
                        blend,
                    }));
                    std::hint::black_box(renderer.render(&train, &world));
                }
                renderer
                    .set_scene_context(Some(SceneContext::from_journey(&world.journey_snapshot())));
                std::hint::black_box(renderer.render(&train, &world));
            }
        }
    }
    // Exercise a visible platform with real-service overrides and bounded crowds.
    renderer.set_theme(SceneTheme::Journey);
    renderer.set_quality(RenderQuality::Full);
    world.set_station_wait(4.0);
    for progress in [0.0, 0.5, 1.0, f32::NAN] {
        renderer.set_station_service(Some(StationService {
            progress,
            boarding: u32::MAX,
            alighting: u32::MAX,
        }));
        std::hint::black_box(renderer.render(&train, &world));
    }
    for engine in EngineStyle::ALL {
        renderer.set_engine(engine);
        renderer.set_local_lighting(Some(348.0));
        for livery in cpu_express::simulation::objectives::Livery::ALL {
            renderer.set_livery(livery);
            std::hint::black_box(renderer.render(&train, &world));
        }
    }
    for style in CoachStyle::ALL {
        renderer.set_coaches([style; 3]);
        for quality in [RenderQuality::Full, RenderQuality::Low] {
            renderer.set_quality(quality);
            std::hint::black_box(renderer.render(&train, &world));
        }
    }
    renderer.set_local_lighting(None);
    renderer.set_scene_context(Some(SceneContext {
        biome: Biome::Forest,
        next_biome: Biome::Forest,
        blend: 0.5,
    }));
    for tag in [
        cpu_express::simulation::catalog::SceneryTag::Plain,
        cpu_express::simulation::catalog::SceneryTag::Farmland,
        cpu_express::simulation::catalog::SceneryTag::Forest,
        cpu_express::simulation::catalog::SceneryTag::Mountain,
        cpu_express::simulation::catalog::SceneryTag::River,
        cpu_express::simulation::catalog::SceneryTag::Bridge,
        cpu_express::simulation::catalog::SceneryTag::Viaduct,
        cpu_express::simulation::catalog::SceneryTag::Tunnel,
        cpu_express::simulation::catalog::SceneryTag::Urban,
        cpu_express::simulation::catalog::SceneryTag::Port,
        cpu_express::simulation::catalog::SceneryTag::Coast,
        cpu_express::simulation::catalog::SceneryTag::Plateau,
    ] {
        renderer.set_scenery_context(cpu_express::rendering::SceneryContext {
            current: cpu_express::rendering::SegmentScenery::from_tags(&[tag]),
            next: cpu_express::rendering::SegmentScenery::EMPTY,
        });
        for quality in [RenderQuality::Full, RenderQuality::Low] {
            renderer.set_quality(quality);
            std::hint::black_box(renderer.render(&train, &world));
        }
    }
    renderer.set_scenery_context(cpu_express::rendering::SceneryContext::default());
    for style in cpu_express::rendering::StationStyle::ALL {
        renderer.set_station_styles(cpu_express::rendering::StationStyles {
            departing: Some(style),
            arriving: Some(style),
        });
        for quality in [RenderQuality::Full, RenderQuality::Low] {
            renderer.set_quality(quality);
            std::hint::black_box(renderer.render(&train, &world));
        }
    }
    renderer.set_station_styles(cpu_express::rendering::StationStyles::default());
    for reduce_motion in [false, true] {
        for cold in [false, true] {
            renderer.set_passenger_options(cpu_express::rendering::PassengerOptions {
                reduce_motion,
                cold,
            });
            for weather in [Weather::Clear, Weather::Rain, Weather::Snow] {
                renderer.set_local_atmosphere(Some((weather, true)));
                for progress in [0.0, 0.1, 0.5, 0.9, 1.0] {
                    renderer.set_station_service(Some(StationService {
                        progress,
                        boarding: 3,
                        alighting: 3,
                    }));
                    std::hint::black_box(renderer.render(&train, &world));
                }
            }
        }
    }
    let mut portraits = [eframe::egui::Color32::TRANSPARENT; 320 * 70];
    for kind in cpu_express::rendering::PassengerKind::ALL {
        for action in cpu_express::rendering::PassengerAction::ALL {
            for frame in 0..4 {
                cpu_express::rendering::draw_passenger_preview(
                    &mut portraits,
                    (160, 53),
                    cpu_express::rendering::PassengerSprite {
                        kind,
                        action,
                        frame,
                        outfit: frame,
                        weather: Weather::Rain,
                        options: cpu_express::rendering::PassengerOptions::default(),
                    },
                    &cpu_express::rendering::Palette::for_biome(Biome::Forest),
                );
            }
        }
    }
    for branch_world in &branch_worlds {
        for weather in [Weather::Rain, Weather::Snow, Weather::Thunderstorm] {
            renderer.set_local_atmosphere(Some((weather, true)));
            for detail in [None, Some(0.0), Some(100.0), Some(f32::MAX), Some(f32::NAN)] {
                renderer.set_weather_detail(detail, detail);
                std::hint::black_box(renderer.render(&train, branch_world));
            }
        }
    }
    COUNTING.with(|active| active.set(false));
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}

#[test]
fn largura_estavel_nao_aloca_inclusive_transicoes_e_clima() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    for width in [160, 320, 641, 1920, 7680] {
        renderer.set_width(width);
        ALLOCATIONS.with(|count| count.set(0));
        COUNTING.with(|active| active.set(true));
        for theme in [SceneTheme::Journey, SceneTheme::Legacy] {
            renderer.set_theme(theme);
            renderer.set_scene_context(Some(SceneContext {
                biome: Biome::Forest,
                next_biome: Biome::Coast,
                blend: 0.5,
            }));
            renderer.set_local_atmosphere(Some((Weather::Rain, false)));
            for _ in 0..3 {
                renderer.set_width(width);
                std::hint::black_box(renderer.render(&train, &world));
            }
        }
        COUNTING.with(|active| active.set(false));
        assert_eq!(ALLOCATIONS.with(Cell::get), 0, "largura {width}");
    }
}
