use cpu_express::{
    config::Config,
    rendering::{
        Biome, SceneContext, SceneOptions, SceneRenderer, SceneTheme, SceneryContext,
        SegmentScenery, Weather,
    },
    simulation::{Train, World, catalog::SceneryTag as T},
};

const TAGS: [T; 12] = [
    T::Plain,
    T::Farmland,
    T::Forest,
    T::Mountain,
    T::River,
    T::Bridge,
    T::Viaduct,
    T::Tunnel,
    T::Urban,
    T::Port,
    T::Coast,
    T::Plateau,
];

#[test]
fn etiquetas_sao_conjunto_puro_sem_ordem_nem_duplicatas() {
    assert!(SegmentScenery::from_tags(&[]).is_empty());
    for tag in TAGS {
        let one = SegmentScenery::from_tags(&[tag]);
        for candidate in TAGS {
            assert_eq!(one.contains(candidate), tag == candidate);
        }
    }
    assert_eq!(
        SegmentScenery::from_tags(&[T::Forest, T::Tunnel, T::Forest]),
        SegmentScenery::from_tags(&[T::Tunnel, T::Forest])
    );
}

#[test]
fn etiquetas_sao_distintas_deterministicas_e_reset_restaura_aurora() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_scene_context(Some(SceneContext::new(Biome::Forest)));
    for width in [160, 320, 641] {
        renderer.set_width(width);
        for (light, weather) in [
            (180.0, Weather::Clear),
            (348.0, Weather::Clear),
            (480.0, Weather::Clear),
            (180.0, Weather::Rain),
        ] {
            renderer.set_local_lighting(Some(light));
            renderer.set_local_atmosphere(Some((weather, true)));
            renderer.set_scenery_context(SceneryContext::default());
            let aurora = renderer.render(&train, &world).to_vec();
            let mut frames = Vec::new();
            for tag in TAGS {
                renderer
                    .set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[tag])));
                let frame = renderer.render(&train, &world).to_vec();
                assert_ne!(frame, aurora, "{tag:?}");
                assert!(frame.iter().all(|p| p.a() == 255));
                assert_eq!(frame, renderer.render(&train, &world));
                assert!(
                    !frames.contains(&frame),
                    "etiqueta sem silhueta própria: {tag:?}"
                );
                frames.push(frame);
            }
            renderer.set_scenery_context(SceneryContext::default());
            assert_eq!(aurora, renderer.render(&train, &world));
        }
    }
}

#[test]
fn transicao_de_etiquetas_funciona_no_mesmo_bioma() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(SceneOptions {
        day_cycle: false,
        clouds: false,
        ..Default::default()
    });
    let left = SegmentScenery::from_tags(&[T::Port]);
    let right = SegmentScenery::from_tags(&[T::Plateau]);
    renderer.set_scenery_context(SceneryContext {
        current: left,
        next: right,
    });
    let frames = [0.0, 0.5, 1.0].map(|blend| {
        renderer.set_scene_context(Some(SceneContext {
            biome: Biome::Forest,
            next_biome: Biome::Forest,
            blend,
        }));
        renderer.render(&train, &world).to_vec()
    });
    assert_ne!(frames[0], frames[1]);
    assert_ne!(frames[1], frames[2]);
    // The top 30 rows contain no train or station at this travel snapshot.
    for i in 0..320 * 30 {
        let average = |a: u8, b: u8| (u16::from(a) + u16::from(b)).div_ceil(2) as u8;
        assert_eq!(
            frames[1][i].r(),
            average(frames[0][i].r(), frames[2][i].r())
        );
    }
    renderer.set_scenery_context(SceneryContext::new(right));
    assert_eq!(frames[2], renderer.render(&train, &world));
}

#[test]
fn tema_legado_ignora_etiquetas() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_theme(SceneTheme::Legacy);
    let old = renderer.render(&train, &world).to_vec();
    renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&TAGS)));
    assert_eq!(old, renderer.render(&train, &world));
}

#[test]
fn serra_com_mata_usa_terreno_verde_sem_recolorir_o_ceu_do_bioma() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_options(SceneOptions {
        clouds: false,
        day_cycle: false,
        ..Default::default()
    });
    renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(&[
        T::Mountain,
        T::Forest,
    ])));
    renderer.set_scene_context(Some(SceneContext::new(Biome::HighDesert)));
    let highland = renderer.render(&train, &world).to_vec();
    renderer.set_scene_context(Some(SceneContext::new(Biome::Forest)));
    let forest = renderer.render(&train, &world);
    // An authored peak and a patch of canopy share the wet terrain palette.
    assert_eq!(highland[20 * 320 + 99], forest[20 * 320 + 99]);
    assert_eq!(highland[25 * 320 + 46], forest[25 * 320 + 46]);
    assert_ne!(highland[0], forest[0]);
}
