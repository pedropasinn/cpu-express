//! Integration regressions: the real station circuit drives presentation,
//! while rendering stays read-only and every active step obeys World bounds.

use cpu_express::{
    config::Config,
    rendering::{Biome, SceneContext, SceneOptions, SceneRenderer},
    simulation::{Journey, Train, World},
};

fn travel(journey: &mut Journey, mut distance: f64) {
    while distance > 0.0 {
        let step = distance.min(200.0);
        journey.advance(1.0, step);
        distance -= step;
    }
}

#[test]
fn station_circuit_controls_all_four_visual_biomes_for_every_seed() {
    for seed in [0, 42, u64::MAX] {
        let mut journey = Journey::from_seed(seed);
        let biomes = [
            Biome::GreenValley,
            Biome::Forest,
            Biome::HighDesert,
            Biome::Coast,
        ];
        for (index, biome) in biomes.into_iter().enumerate() {
            let snapshot = journey.snapshot();
            let context = SceneContext::from_journey(&snapshot);
            assert_eq!(context.biome, biome);
            assert_eq!(context.next_biome, biomes[(index + 1) % biomes.len()]);
            assert_eq!(context.blend, 0.0);

            travel(&mut journey, snapshot.route_segment.length_units * 0.875);
            let middle = SceneContext::from_journey(&journey.snapshot());
            assert_eq!(middle.biome, biome);
            assert!((middle.blend - 0.5).abs() < 1e-6);
            let remaining = journey.snapshot().distance_to_next_station_units;
            travel(&mut journey, remaining);
        }
        assert_eq!(journey.snapshot().completed_laps, 1);
        assert_eq!(
            SceneContext::from_journey(&journey.snapshot()).biome,
            Biome::GreenValley
        );
    }
}

#[test]
fn station_crossings_keep_scenery_continuous_and_do_not_reset_parallax() {
    let mut world = World::default();
    let train = Train::new(&Config::default());
    let mut renderer = SceneRenderer::new();
    renderer.set_options(SceneOptions {
        day_cycle: false,
        clouds: false,
        ..SceneOptions::default()
    });

    for _ in 0..4 {
        let mut remaining = world.journey_snapshot().distance_to_next_station_units - 0.001;
        while remaining > 0.0 {
            let distance = remaining.min(200.0);
            world.update(1.0, distance, 0.0);
            remaining -= distance;
        }
        let before = world.journey_snapshot();
        let before_scroll = world.scroll_px();
        let before_context = SceneContext::from_journey(&before);
        renderer.set_scene_context(Some(before_context));
        let pixels_before = renderer.render(&train, &world).to_vec();
        // The blend has reached the next environment at pixel precision.
        renderer.set_scene_context(Some(SceneContext::new(before_context.next_biome)));
        assert_eq!(renderer.render(&train, &world), pixels_before.as_slice());
        world.update(0.01, 0.002, 0.0);
        let after = world.journey_snapshot();
        assert_eq!(before.route_segment.to, after.route_segment.from);
        assert!((world.scroll_px() - before_scroll - 0.002).abs() < 1e-8);
        renderer.set_scene_context(Some(SceneContext::from_journey(&after)));
        let pixels_after = renderer.render(&train, &world);
        let differences = pixels_before
            .iter()
            .zip(pixels_after)
            .filter(|(a, b)| a != b)
            .count();
        // Integer rasterization can shift a contour/tree by one whole pixel
        // across a fractional boundary, as in the existing wrap regression.
        let fraction = differences as f64 / pixels_before.len() as f64;
        assert!(
            fraction < 0.12,
            "station {:?}, changed fraction {fraction}",
            after.route_segment.from
        );
    }
}

#[test]
fn paused_journey_has_a_stable_visual_context_and_rendering_is_read_only() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..20 {
        world.advance_train(&mut train, 80.0, 1.0, &config);
    }
    world.set_paused(true);
    let snapshot = world.journey_snapshot();
    let context = SceneContext::from_journey(&snapshot);
    let mut renderer = SceneRenderer::new();
    renderer.set_scene_context(Some(context));
    let frame = renderer.render(&train, &world).to_vec();
    for _ in 0..10 {
        world.advance_train(&mut train, 100.0, 1.0, &config);
        assert_eq!(world.journey_snapshot(), snapshot);
        assert_eq!(
            SceneContext::from_journey(&world.journey_snapshot()),
            context
        );
        assert_eq!(renderer.render(&train, &world), frame.as_slice());
    }
}

#[test]
fn invalid_external_snapshot_progress_cannot_poison_visual_blending() {
    let mut snapshot = Journey::default().snapshot();
    for progress in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 2.0] {
        snapshot.segment_progress = progress;
        let context = SceneContext::from_journey(&snapshot);
        assert!(context.blend.is_finite());
        assert!((0.0..=1.0).contains(&context.blend));
    }
}
