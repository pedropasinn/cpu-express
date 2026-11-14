use cpu_express::{
    config::Config,
    rendering::{SceneRenderer, Weather},
    simulation::{Train, World},
};

#[test]
fn explicit_phase_overrides_day_flag_and_wraps_without_changing_world() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let snapshot = world.journey_snapshot();
    let mut renderer = SceneRenderer::new();
    renderer.set_local_atmosphere(Some((Weather::Clear, false)));
    let night = renderer.render(&train, &world).to_vec();
    renderer.set_local_lighting(Some(180.0));
    let day = renderer.render(&train, &world).to_vec();
    assert_ne!(day, night);
    for phase in [780.0, -420.0] {
        renderer.set_local_lighting(Some(phase));
        assert_eq!(day, renderer.render(&train, &world));
    }
    for phase in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        renderer.set_local_lighting(phase);
        assert_eq!(night, renderer.render(&train, &world));
    }
    assert_eq!(snapshot, world.journey_snapshot());
}

#[test]
fn dawn_noon_sunset_and_night_are_distinct_and_pause_is_stable() {
    let train = Train::new(&Config::default());
    let mut world = World::default();
    world.set_paused(true);
    let mut renderer = SceneRenderer::new();
    let mut frames = Vec::new();
    for phase in [0.0, 180.0, 348.0, 480.0] {
        renderer.set_local_lighting(Some(phase));
        let pixels = renderer.render(&train, &world).to_vec();
        for old in &frames {
            assert_ne!(&pixels, old);
        }
        assert_eq!(pixels, renderer.render(&train, &world));
        frames.push(pixels);
    }
    renderer.set_local_lighting(Some(600.0));
    assert_eq!(frames[0], renderer.render(&train, &world));
    renderer.set_local_atmosphere(Some((Weather::Rain, true)));
    assert_ne!(frames[0], renderer.render(&train, &world));
}
