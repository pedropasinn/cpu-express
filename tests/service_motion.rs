use cpu_express::{
    config::Config,
    simulation::{BranchKind, Journey, Route, Train, World},
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
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
fn animation_and_braking_allocate_nothing_including_branch() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    let mut branch = World::default();
    let route = Route::default();
    branch.restore_journey(
        Journey::restore_checkpoint(route.seed(), route.length_units(), 1000.0).unwrap(),
    );
    branch.request_excursion(BranchKind::Serra).unwrap();
    ALLOCATIONS.with(|c| c.set(0));
    COUNTING.with(|c| c.set(true));
    for _ in 0..600 {
        train.update(50.0, 1.0 / 60.0, &config);
        train.update_with_braking(50.0, 1.0 / 60.0, &config, 0.2);
        world.advance_expedition(&mut train, 50.0, 1.0 / 60.0, &config);
        branch.advance_expedition(&mut train, 50.0, 1.0 / 60.0, &config);
    }
    COUNTING.with(|c| c.set(false));
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}
#[test]
fn scaled_numeric_parameters_match_former_scaled_config_path() {
    for (min, max, k) in [
        (12.0, 100.0, 1.8),
        (f64::NAN, f64::INFINITY, f64::NAN),
        (-5.0, 1000.0, 0.0),
        (200.0, 100.0, 30.0),
    ] {
        let config = Config {
            min_speed_kmh: min,
            max_speed_kmh: max,
            smoothing_k: k,
            ..Config::default()
        };
        for factor in [0.0, 0.05, 0.5, 1.0] {
            let mut scaled = config.clone();
            scaled.min_speed_kmh *= factor;
            scaled.max_speed_kmh *= factor;
            let scaled = scaled.normalized();
            let mut a = Train::new(&config);
            let mut b = a.clone();
            for dt in [0.0, 1.0 / 60.0, 0.25, 1.0, 2.0] {
                let da = a.update_with_braking(65.0, dt, &config, factor);
                let db = b.update(65.0, dt, &scaled);
                assert_eq!(da, db);
                assert_eq!(a.speed_kmh(), b.speed_kmh());
                assert_eq!(a.wheel_phase(), b.wheel_phase());
            }
        }
    }
}
#[test]
fn braking_preserves_analytical_partition_invariance() {
    let config = Config::default();
    let mut a = Train::new(&config);
    let mut b = a.clone();
    let da = a.update_with_braking(75.0, 1.0, &config, 0.4);
    let db: f64 = (0..60)
        .map(|_| b.update_with_braking(75.0, 1.0 / 60.0, &config, 0.4))
        .sum();
    assert!((da - db).abs() < 1e-10);
    assert!((a.speed_kmh() - b.speed_kmh()).abs() < 1e-10);
    assert!((a.wheel_phase() - b.wheel_phase()).abs() < 1e-10);
}
