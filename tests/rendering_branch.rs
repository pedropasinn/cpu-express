use cpu_express::{
    config::Config,
    rendering::{Biome, SceneContext, SceneRenderer, StationService},
    simulation::{BranchKind, BranchPhase, Journey, Route, Train, World},
};

fn ready(kind: BranchKind) -> World {
    let route = Route::default();
    let mut world = World::default();
    world.restore_journey(
        Journey::restore_checkpoint(route.seed(), route.length_units(), 1000.0).unwrap(),
    );
    world.request_excursion(kind).unwrap();
    world
}

#[test]
fn excursion_context_overrides_parked_route_then_restores_it_without_mutation() {
    let config = Config::default();
    for kind in [BranchKind::Serra, BranchKind::Litoral] {
        let mut train = Train::new(&config);
        let mut world = ready(kind);
        let distance = world.journey_snapshot().total_distance_units;
        let mut renderer = SceneRenderer::new();
        let parked = SceneContext::new(Biome::GreenValley);
        renderer.set_scene_context(Some(parked));
        assert_eq!(renderer.scene_context(&world).biome, Biome::GreenValley);
        for _ in 0..90 {
            world.advance_expedition(&mut train, 40.0, 1.0, &config);
        }
        assert_eq!(
            world.excursion_snapshot().current.unwrap().phase,
            BranchPhase::Service
        );
        assert_eq!(
            renderer.scene_context(&world),
            SceneContext::new(Biome::from(kind.biome()))
        );
        renderer.set_station_service(Some(StationService::default()));
        let stop = renderer.render(&train, &world).to_vec();
        // The previous main-route service is irrelevant to this sightseeing stop.
        renderer.set_station_service(Some(StationService {
            progress: 0.5,
            boarding: 72,
            alighting: 72,
        }));
        assert_eq!(stop, renderer.render(&train, &world));
        world.set_paused(true);
        world.advance_expedition(&mut train, 100.0, 1.0, &config);
        assert_eq!(stop, renderer.render(&train, &world));
        world.set_paused(false);
        for _ in 0..98 {
            world.advance_expedition(&mut train, 40.0, 1.0, &config);
        }
        assert!(world.excursion_snapshot().current.is_none());
        assert_eq!(renderer.scene_context(&world), parked);
        assert_eq!(world.journey_snapshot().total_distance_units, distance);
    }
}
