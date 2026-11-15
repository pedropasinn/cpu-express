use cpu_express::{
    config::Config,
    rendering::{
        Biome, EngineStyle, RenderQuality, SceneContext, SceneOptions, SceneRenderer, SceneTheme,
        SceneryContext, SceneryLifeOptions, SegmentScenery, Weather,
    },
    simulation::{
        AdvanceOutcome, DEFAULT_JOURNEY_SEED, Journey, Train, World, catalog::SceneryTag as T,
    },
};

fn world(seconds: f64) -> World {
    let mut world = World::default();
    world.restore_journey(
        Journey::restore_checkpoint(DEFAULT_JOURNEY_SEED, 1280.0, seconds).unwrap(),
    );
    world
}

fn renderer() -> SceneRenderer {
    let mut renderer = SceneRenderer::new();
    renderer.set_scene_context(Some(SceneContext::new(Biome::Forest)));
    renderer.set_local_lighting(Some(480.0));
    renderer.set_engine(EngineStyle::Diesel);
    renderer.set_options(SceneOptions {
        clouds: false,
        weather: Weather::Clear,
        ..Default::default()
    });
    renderer
}

#[test]
fn tempo_ativo_determinismo_reducao_e_low_no_renderer_real() {
    let train = Train::new(&Config::default());
    let mut renderer = renderer();
    for tag in [
        T::Farmland,
        T::Forest,
        T::Mountain,
        T::River,
        T::Urban,
        T::Port,
        T::Coast,
        T::Plateau,
    ] {
        renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[tag])));
        for reduced in [false, true] {
            for quality in [RenderQuality::Full, RenderQuality::Low] {
                renderer.set_scenery_life_options(SceneryLifeOptions {
                    reduce_motion: reduced,
                });
                renderer.set_quality(quality);
                let initial = renderer.render(&train, &world(0.0)).to_vec();
                let mut changed = false;
                for seconds in [2.0, 8.0, 24.0, 48.0] {
                    let later = renderer.render(&train, &world(seconds)).to_vec();
                    if reduced || quality == RenderQuality::Low {
                        assert_eq!(initial, later, "{tag:?}: pose deve ficar estavel");
                    }
                    changed |= initial != later;
                }
                if !reduced && quality == RenderQuality::Full {
                    assert!(changed, "{tag:?}: movimento deve ficar visivel");
                }
                // Rendering other times does not mutate a decorative clock.
                assert_eq!(initial, renderer.render(&train, &world(0.0)));
            }
        }
    }
}

#[test]
fn pausa_e_suspensao_congelam_vida_sem_acumular_tempo() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = world(12.0);
    let mut renderer = renderer();
    renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[
        T::Farmland,
        T::Forest,
        T::Mountain,
        T::River,
        T::Urban,
        T::Port,
        T::Coast,
    ])));
    let before = renderer.render(&train, &world).to_vec();
    world.set_paused(true);
    assert_eq!(
        world.advance_expedition(&mut train, 100.0, 1.0, &config),
        AdvanceOutcome::Paused
    );
    assert_eq!(before, renderer.render(&train, &world));
    world.set_paused(false);
    world.advance_expedition(&mut train, 100.0, 3600.0, &config);
    assert_eq!(world.elapsed_seconds(), 12.0);
    assert_eq!(before, renderer.render(&train, &world));
}

#[test]
fn opcoes_nao_alteram_aurora_vazio_nem_legacy_e_reset_limpa_etiquetas() {
    let train = Train::new(&Config::default());
    let world = world(24.0);
    let mut renderer = renderer();
    for theme in [SceneTheme::Journey, SceneTheme::Legacy] {
        renderer.set_theme(theme);
        renderer.set_scenery_context(SceneryContext::default());
        let empty = renderer.render(&train, &world).to_vec();
        renderer.set_scenery_life_options(SceneryLifeOptions {
            reduce_motion: true,
        });
        assert_eq!(empty, renderer.render(&train, &world));
        renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[
            T::Farmland,
        ])));
        let tagged = renderer.render(&train, &world).to_vec();
        if theme == SceneTheme::Legacy {
            assert_eq!(empty, tagged);
        } else {
            assert_ne!(empty, tagged);
        }
        renderer.set_scenery_context(SceneryContext::default());
        assert_eq!(empty, renderer.render(&train, &world));
    }
}
