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
        Biome, RenderQuality, SceneContext, SceneOptions, SceneRenderer, SceneTheme, Weather,
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
    COUNTING.with(|active| active.set(false));
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}
