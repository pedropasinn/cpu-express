use cpu_express::{
    config::Config,
    rendering::{SceneRenderer, StationService, StationStyle, StationStyles, Weather},
    simulation::{Train, World},
};

#[test]
fn styles_are_distinct_deterministic_and_opt_in_in_each_atmosphere() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_station_service(Some(StationService::default()));
    for (light, weather) in [
        (180.0, Weather::Clear),
        (348.0, Weather::Clear),
        (480.0, Weather::Clear),
        (180.0, Weather::Rain),
    ] {
        renderer.set_local_lighting(Some(light));
        renderer.set_local_atmosphere(Some((weather, light < 400.0)));
        let original = renderer.render(&train, &world).to_vec();
        let mut frames = Vec::new();
        for style in StationStyle::ALL {
            renderer.set_station_styles(StationStyles {
                departing: Some(style),
                arriving: Some(style),
            });
            let pixels = renderer.render(&train, &world).to_vec();
            assert_eq!(pixels, renderer.render(&train, &world));
            assert_ne!(pixels, original);
            assert!(!frames.contains(&pixels), "{}", style.id());
            frames.push(pixels);
        }
        renderer.set_station_styles(StationStyles::default());
        assert_eq!(original, renderer.render(&train, &world));
    }
}
