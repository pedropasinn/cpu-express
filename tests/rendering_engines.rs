use cpu_express::{
    config::Config,
    rendering::{EngineStyle, SceneRenderer, SceneTheme, WIDTH},
    simulation::{Journey, Train, World, objectives::Livery},
};

#[test]
fn engine_catalog_round_trips_with_steam_as_missing_field_default() {
    #[derive(Default, serde::Deserialize)]
    #[serde(default)]
    struct OldPreferences {
        engine: EngineStyle,
    }
    assert_eq!(
        serde_json::from_str::<OldPreferences>("{}").unwrap().engine,
        EngineStyle::Steam
    );
    for (style, id) in EngineStyle::ALL.into_iter().zip([
        "steam",
        "diesel",
        "electric",
        "freight_steam",
        "shunter",
        "railcar",
    ]) {
        let json = serde_json::to_string(&style).unwrap();
        assert_eq!(json, format!("\"{id}\""));
        assert_eq!(serde_json::from_str::<EngineStyle>(&json).unwrap(), style);
    }
}

#[test]
fn nonsteam_engines_ignore_existing_simulation_smoke_in_both_themes() {
    let train = Train::new(&Config::default());
    let mut smoky = World::default();
    for _ in 0..30 {
        smoky.update(0.1, 0.0, 80.0);
    }
    assert!(!smoky.smoke().is_empty());
    let snapshot = smoky.journey_snapshot();
    let mut clean = smoky.clone();
    clean.restore_journey(
        Journey::restore_checkpoint(
            snapshot.seed,
            snapshot.total_distance_units,
            snapshot.active_seconds,
        )
        .unwrap(),
    );
    assert!(clean.smoke().is_empty());
    let mut renderer = SceneRenderer::new();
    for theme in [SceneTheme::Journey, SceneTheme::Legacy] {
        renderer.set_theme(theme);
        for style in EngineStyle::ALL.into_iter().filter(|e| e.emits_smoke()) {
            renderer.set_engine(style);
            assert_ne!(
                renderer.render(&train, &smoky).to_vec(),
                renderer.render(&train, &clean)
            );
        }
        for style in EngineStyle::ALL.into_iter().filter(|e| !e.emits_smoke()) {
            renderer.set_engine(style);
            assert_eq!(
                renderer.render(&train, &smoky).to_vec(),
                renderer.render(&train, &clean)
            );
        }
    }
}

#[test]
fn all_liveries_work_on_each_engine_without_changing_running_gear() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    for style in EngineStyle::ALL {
        renderer.set_engine(style);
        renderer.set_livery(Livery::Aurora);
        let original = renderer.render(&train, &world).to_vec();
        let mut variants = Vec::new();
        for livery in Livery::ALL {
            renderer.set_livery(livery);
            let pixels = renderer.render(&train, &world).to_vec();
            for y in 51..59 {
                assert_eq!(
                    &pixels[y * WIDTH + 180..y * WIDTH + 240],
                    &original[y * WIDTH + 180..y * WIDTH + 240]
                );
            }
            for other in &variants {
                assert_ne!(&pixels, other);
            }
            variants.push(pixels);
        }
    }
}
