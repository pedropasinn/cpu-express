use cpu_express::{
    config::Config,
    rendering::{
        Biome, SceneContext, SceneOptions, SceneRenderer, SceneryContext, SceneryLifeOptions,
        SegmentScenery, SettlementContext, SettlementElement, Weather, draw_settlement_preview,
    },
    simulation::{
        Train, World,
        catalog::{Catalog, SceneryTag as T},
    },
};
use eframe::egui::Color32;

#[test]
fn elementos_tem_silhuetas_proprias_e_luz_noturna() {
    assert!(SettlementElement::ALL.len() >= 30);
    let mut frames = Vec::new();
    for e in SettlementElement::ALL {
        let mut day = vec![Color32::BLACK; 64 * 70];
        draw_settlement_preview(&mut day, e, 2026, 180.0, Weather::Clear);
        assert!(!frames.contains(&day), "silhueta repetida: {}", e.name());
        let mut night = day.clone();
        draw_settlement_preview(&mut night, e, 2026, 480.0, Weather::Clear);
        assert_ne!(day, night);
        frames.push(day);
    }
}

#[test]
fn aproximacao_respeita_os_extremos_habitados() {
    let d = |p, a, b| SettlementContext::along_segment(42, p, a, b).density;
    assert_eq!(d(0.0, true, true), 1.0);
    assert_eq!(d(0.5, true, true), 0.0);
    assert_eq!(d(1.0, true, true), 1.0);
    assert!(d(0.8, false, true) < d(0.9, false, true));
    assert!(d(0.1, true, false) > d(0.2, true, false));
    for p in [0.0, 0.5, 1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(d(p, false, false), 0.0);
    }
    assert_eq!(d(0.0, false, true), 0.0);
    assert_eq!(d(1.0, true, false), 0.0);
}

#[test]
fn sementes_deterministicas_sem_historico_e_reducao_de_movimento() {
    let train = Train::new(&Config::default());
    let mut world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_width(641);
    renderer.set_scene_context(Some(SceneContext::new(Biome::GreenValley)));
    renderer.set_options(SceneOptions {
        clouds: false,
        ..Default::default()
    });
    renderer.set_local_lighting(Some(180.0));
    renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[
        T::City,
        T::Town,
        T::Industrial,
        T::Village,
        T::Suburb,
    ])));
    renderer.set_settlement_context(SettlementContext {
        seed: 42,
        density: 1.0,
    });
    let first = renderer.render(&train, &world).to_vec();
    renderer.set_settlement_context(SettlementContext {
        seed: 99,
        density: 1.0,
    });
    assert_ne!(first, renderer.render(&train, &world));
    renderer.set_settlement_context(SettlementContext {
        seed: 42,
        density: 1.0,
    });
    assert_eq!(first, renderer.render(&train, &world));
    renderer.set_scenery_life_options(SceneryLifeOptions {
        reduce_motion: true,
    });
    // Examine skyline only, excluding existing station passengers and train.
    let frozen = renderer.render(&train, &world)[..641 * 25].to_vec();
    world.update(3.0, 0.0, 0.0);
    assert_eq!(frozen, renderer.render(&train, &world)[..641 * 25]);
    renderer.set_settlement_context(SettlementContext {
        seed: 42,
        density: 0.0,
    });
    assert_ne!(first, renderer.render(&train, &world));
}

#[test]
fn catalogo_distribui_cidades_sem_urbanizar_a_serra() {
    let catalog = Catalog::builtin().as_ref().unwrap();
    let line = catalog.line("curitiba-paranagua").unwrap();
    for s in &line.segments {
        if s.from == "pinhais" || s.to == "pinhais" {
            assert!(s.scenery.contains(&T::City));
            assert!(s.scenery.contains(&T::Suburb));
        }
        if s.from == "paranagua" {
            assert!(s.scenery.contains(&T::Port));
            assert!(s.scenery.contains(&T::Industrial));
        }
        if s.from == "morretes" || s.to == "morretes" {
            assert!(s.scenery.contains(&T::Village));
            assert!(s.scenery.contains(&T::Town));
        }
        if s.biome == "highland" {
            assert!(s.scenery.contains(&T::Forest));
            assert!(!s.scenery.contains(&T::City));
        }
    }
    assert!(
        catalog.line("aurora-v1").unwrap().segments[0]
            .scenery
            .contains(&T::Village)
    );
}
