//! Content unlocks, service transactions, stable identities and v4 round trips.
use cpu_express::{
    config::Config,
    progress,
    rendering::{CoachStyle, EngineStyle, SceneRenderer},
    simulation::{CargoKind, ContractStatus, Journey, ServiceState, ServiceWagon, Train, World},
};

fn unlocked_world() -> World {
    let mut world = World::default();
    world.restore_journey(Journey::restore_checkpoint(42, 40_000.0, 2_000.0).unwrap());
    assert!(world.journey_snapshot().stations_visited >= 10);
    world
}

fn tick(world: &mut World) {
    let config = Config::default();
    world.advance_expedition(&mut Train::new(&config), 37.0, 1.0, &config);
}

#[test]
fn all_stock_has_a_stable_identity_and_bidirectional_visual_adapter() {
    assert_eq!(CoachStyle::ALL.len(), ServiceWagon::ALL.len());
    for wagon in ServiceWagon::ALL {
        let style = CoachStyle::from(wagon);
        assert!(CoachStyle::ALL.contains(&style));
        assert_eq!(ServiceWagon::from(style), wagon);
        assert_eq!(style.name(), wagon.name());
        let json = serde_json::to_string(&wagon).unwrap();
        assert_eq!(serde_json::to_string(&style).unwrap(), json);
        assert_eq!(serde_json::from_str::<ServiceWagon>(&json).unwrap(), wagon);
        assert_eq!(serde_json::from_str::<CoachStyle>(&json).unwrap(), style);
    }
    for kind in CargoKind::ALL {
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(serde_json::from_str::<CargoKind>(&json).unwrap(), kind);
    }
    for (id, wagon) in [
        ("passenger", ServiceWagon::Passenger),
        ("sleeper", ServiceWagon::Sleeper),
        ("freight", ServiceWagon::Freight),
        ("tank", ServiceWagon::Tank),
        ("observation", ServiceWagon::Observation),
    ] {
        assert_eq!(serde_json::to_string(&wagon).unwrap(), format!("\"{id}\""));
    }
}

#[test]
fn arrival_unlocks_are_enforced_at_the_domain_boundary() {
    for wagon in ServiceWagon::ALL
        .into_iter()
        .filter(|w| w.required_stops() > 0)
    {
        let mut state = ServiceState::migrated(World::default().journey_snapshot());
        state.last_processed_visit = wagon.required_stops() - 1;
        let original = state.clone();
        assert!(state.set_composition([wagon; 3]).is_err());
        assert_eq!(state, original);
        state.last_processed_visit += 1;
        state.set_composition([wagon; 3]).unwrap();
        assert!(wagon.available(state.last_processed_visit));
    }
}

#[test]
fn specialized_cargo_loads_delivers_and_survives_v4_restart() {
    let folder = std::env::temp_dir().join(format!(
        "cpu-content-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).unwrap();
    for (wagon, kind) in [
        (ServiceWagon::Mail, CargoKind::Parcels),
        (ServiceWagon::Gondola, CargoKind::Ore),
        (ServiceWagon::Hopper, CargoKind::Grain),
        (ServiceWagon::Timber, CargoKind::Timber),
        (ServiceWagon::Refrigerator, CargoKind::Chilled),
    ] {
        let mut world = unlocked_world();
        world.set_service_composition([wagon; 3]).unwrap();
        for _ in 0..1_000 {
            if world.service_state().cargo_count() > 0 {
                break;
            }
            tick(&mut world);
        }
        let contract = world.service_state().contract.as_ref().unwrap().clone();
        assert_eq!(contract.kind, kind);
        assert_eq!(contract.status, ContractStatus::Loaded);
        let before = world.service_state().clone();
        assert!(
            world
                .set_service_composition([ServiceWagon::Passenger; 3])
                .is_err()
        );
        assert_eq!(world.service_state(), &before);
        let path = folder.join(format!("{wagon:?}.json"));
        progress::save_checkpoint(&path, &world).unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(json["version"], 4);
        let (restored, warning) = progress::load_profile(&path).unwrap();
        assert!(warning.is_none());
        let mut restored = restored.unwrap();
        assert_eq!(world.service_state(), restored.service_state());
        for _ in 0..1_000 {
            if world.service_state().cargo_delivered > 0 {
                break;
            }
            tick(&mut world);
            tick(&mut restored);
        }
        assert_eq!(
            world.service_state().cargo_delivered,
            u64::from(contract.units)
        );
        assert_eq!(world.service_state().credits, contract.reward);
        assert_eq!(world.service_state(), restored.service_state());
        world
            .service_state()
            .validate(world.journey_snapshot())
            .unwrap();
    }
}

#[test]
fn every_new_combination_is_deterministic_at_native_resolution() {
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    for coach in CoachStyle::ALL {
        renderer.set_coaches([coach; 3]);
        for engine in EngineStyle::ALL {
            renderer.set_engine(engine);
            let first = renderer.render(&train, &world).to_vec();
            assert_eq!(renderer.render(&train, &world), first);
        }
    }
}
