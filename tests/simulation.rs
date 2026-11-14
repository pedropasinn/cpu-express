use std::f64::consts::TAU;

use cpu_express::{
    config::Config,
    simulation::{MAX_SMOKE_PARTICLES, SCROLL_PERIOD_PX, Train, World},
};

#[test]
fn requested_cpu_mapping_and_monotonicity() {
    let config = Config::default();
    let mut train = Train::new(&config);
    for (cpu, expected) in [(0.0, 12.0), (50.0, 56.0), (100.0, 100.0)] {
        train.update(cpu, 0.0, &config);
        assert_eq!(train.target_speed_kmh(), expected);
    }
    let mut previous = 0.0;
    for cpu in 0..=100 {
        let mut train = Train::new(&config);
        train.update(cpu as f32, 0.5, &config);
        assert!(train.speed_kmh() >= previous);
        assert!(train.speed_kmh() <= train.target_speed_kmh());
        previous = train.speed_kmh();
    }
}

#[test]
fn analytical_solution_matches_reference_speed_and_distance() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let distance = train.update(100.0, 1.0, &config);
    let expected_speed = 100.0 - 88.0 * (-1.8_f64).exp();
    let expected_distance = (100.0 - 88.0 * (1.0 - (-1.8_f64).exp()) / 1.8) * 0.55;
    assert!((train.speed_kmh() - expected_speed).abs() < 1e-12);
    assert!((distance - expected_distance).abs() < 1e-12);
}

#[test]
fn speed_distance_and_wheels_are_independent_of_frame_rate() {
    fn simulate(fps: u32) -> (f64, f64, f64) {
        let config = Config::default();
        let mut train = Train::new(&config);
        let mut distance = 0.0;
        for second in 0..10 {
            let cpu = if second < 5 { 100.0 } else { 10.0 };
            for _ in 0..fps {
                distance += train.update(cpu, 1.0 / f64::from(fps), &config);
            }
        }
        (train.speed_kmh(), distance, train.wheel_phase())
    }
    let at_30 = simulate(30);
    for fps in [15, 60, 144] {
        let other = simulate(fps);
        assert!((at_30.0 - other.0).abs() < 1e-10);
        assert!((at_30.1 - other.1).abs() < 1e-9);
        assert!((at_30.2 - other.2).abs() < 1e-9);
    }
}

#[test]
fn invalid_inputs_do_not_poison_state_and_speed_never_overshoots() {
    let config = Config::default();
    let mut train = Train::new(&config);
    for cpu in [0.0, 50.0, 100.0, -50.0, 500.0, f32::NAN, f32::INFINITY] {
        for dt in [0.0, -1.0, f64::NAN, f64::INFINITY, 0.03, 10.0, f64::MAX] {
            let distance = train.update(cpu, dt, &config);
            assert!(distance.is_finite() && distance >= 0.0);
            assert!((12.0..=100.0).contains(&train.speed_kmh()));
            assert!((12.0..=100.0).contains(&train.target_speed_kmh()));
            assert!((0.0..TAU).contains(&train.wheel_phase()));
            if !dt.is_finite() || dt <= 0.0 {
                assert_eq!(distance, 0.0);
            }
        }
    }
}

#[test]
fn configuration_repairs_invalid_and_out_of_range_values() {
    let config = Config {
        min_speed_kmh: f64::NAN,
        max_speed_kmh: f64::INFINITY,
        smoothing_k: -2.0,
        fps: 0,
        metrics_interval_ms: 0,
        ..Config::default()
    }
    .normalized();
    assert_eq!(config.min_speed_kmh, 12.0);
    assert_eq!(config.max_speed_kmh, 100.0);
    assert!(config.smoothing_k > 0.0);
    assert!(config.fps > 0);
    assert!(config.metrics_interval_ms >= 250);
    let config = Config {
        min_speed_kmh: 200.0,
        max_speed_kmh: 100.0,
        ..Config::default()
    }
    .normalized();
    assert!(config.max_speed_kmh >= config.min_speed_kmh);
    assert_eq!(config.clone().normalized(), config);
}

#[test]
fn scenery_wrap_and_smoke_remain_bounded_for_a_simulated_hour() {
    let mut world = World::default();
    // 3,600 inexpensive updates represent an hour without waiting or generating
    // machine load. This checks bounded state, not a real-time memory benchmark.
    for _ in 0..3_600 {
        world.update(1.0, 55.0, 100.0);
        assert!(world.smoke().len() <= MAX_SMOKE_PARTICLES);
        assert!((0.0..SCROLL_PERIOD_PX).contains(&world.scroll_px()));
        for particle in world.smoke() {
            assert!(particle.x.is_finite() && particle.y.is_finite());
            assert!(particle.size.is_finite() && particle.size > 0.0);
            assert!((0.0..particle.lifetime).contains(&particle.age));
        }
    }
    assert_eq!(world.elapsed_seconds(), 3_600.0);
    assert_eq!(world.scroll_px(), (3_600.0 * 55.0) % SCROLL_PERIOD_PX);
    assert!(!world.smoke().is_empty());
}

#[test]
fn world_handles_suspension_and_invalid_input_without_unbounded_spawning() {
    let mut world = World::default();
    for dt in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        world.update(dt, 100.0, 100.0);
    }
    assert_eq!(world.elapsed_seconds(), 0.0);
    assert_eq!(world.scroll_px(), 0.0);
    for (dt, distance, speed) in [
        (0.5, f64::NAN, 100.0),
        (10_000.0, 1_000_000.0, f64::NAN),
        (f64::MAX, f64::MAX, f64::MAX),
        (0.5, 20.0, 100.0),
    ] {
        world.update(dt, distance, speed);
        assert!(world.elapsed_seconds().is_finite());
        assert!((0.0..SCROLL_PERIOD_PX).contains(&world.scroll_px()));
        assert!(world.smoke().len() <= MAX_SMOKE_PARTICLES);
        for particle in world.smoke() {
            assert!(particle.x.is_finite() && particle.y.is_finite());
            assert!(particle.age.is_finite() && particle.size.is_finite());
        }
    }
}
