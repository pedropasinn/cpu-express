use cpu_express::{
    config::Config,
    rendering::{EngineStyle, SceneRenderer, Weather},
    simulation::{Train, World},
};

#[test]
fn optional_weather_detail_is_bounded_and_none_preserves_existing_frames() {
    let train = Train::new(&Config::default());
    let mut world = World::default();
    world.update(1.0, 0.0, 0.0);
    let mut renderer = SceneRenderer::new();
    renderer.set_engine(EngineStyle::Electric);
    renderer.set_local_atmosphere(Some((Weather::Rain, true)));
    let baseline = renderer.render(&train, &world).to_vec();
    for (wind, mm) in [(None, None), (Some(f32::NAN), Some(f32::INFINITY))] {
        renderer.set_weather_detail(wind, mm);
        assert_eq!(baseline, renderer.render(&train, &world));
    }
    renderer.set_weather_detail(Some(250.0), Some(100.0));
    let bounded = renderer.render(&train, &world).to_vec();
    renderer.set_weather_detail(Some(f32::MAX), Some(f32::MAX));
    assert_eq!(bounded, renderer.render(&train, &world));
    renderer.set_weather_detail(Some(-10.0), Some(-10.0));
    let empty = renderer.render(&train, &world).to_vec();
    renderer.set_weather_detail(Some(0.0), Some(0.0));
    assert_eq!(empty, renderer.render(&train, &world));
    renderer.set_weather_detail(Some(0.0), Some(1.0));
    let light = renderer.render(&train, &world).to_vec();
    renderer.set_weather_detail(Some(0.0), Some(100.0));
    let heavy = renderer.render(&train, &world).to_vec();
    let differences = |pixels: &[eframe::egui::Color32]| {
        pixels.iter().zip(&empty).filter(|(a, b)| a != b).count()
    };
    assert!(differences(&heavy) > differences(&light));
    renderer.set_weather_detail(Some(250.0), Some(100.0));
    assert_ne!(heavy, renderer.render(&train, &world));
}

#[test]
fn detail_never_sets_weather_condition_and_paused_frames_remain_stable() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_local_atmosphere(Some((Weather::Clear, true)));
    let clear = renderer.render(&train, &world).to_vec();
    renderer.set_weather_detail(Some(100.0), Some(80.0));
    assert_eq!(clear, renderer.render(&train, &world));
    renderer.set_local_atmosphere(Some((Weather::Snow, false)));
    world.set_paused(true);
    let frozen = renderer.render(&train, &world).to_vec();
    for _ in 0..4 {
        world.advance_expedition(&mut train, 100.0, 1.0, &config);
        assert_eq!(frozen, renderer.render(&train, &world));
    }
}
