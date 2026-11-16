//! Additive livery identities and old-v4 ownership compatibility.
use cpu_express::{
    config::Config,
    progress::{Checkpoint, WorldCheckpoint},
    simulation::{Journey, Train, World, objectives::Livery},
};

fn roundtrip(world: &World) -> World {
    let mut checkpoint = WorldCheckpoint {
        version: 4,
        catalog_version: 1,
        line: "aurora-v1".into(),
        journey: Checkpoint::capture(world.journey_snapshot()),
        service: world.service_state().clone(),
        economy: None,
        lines: None,
    };
    checkpoint.journey.station_wait = world.station_wait();
    let bytes = serde_json::to_vec(&checkpoint).unwrap();
    let restored: WorldCheckpoint = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored.version, 4);
    let mut result = World::default();
    result.restore_journey(restored.journey.restore().unwrap());
    result.set_station_wait(restored.journey.station_wait);
    result.restore_service(restored.service).unwrap();
    result
}

#[test]
fn stable_paint_ids_and_arrival_thresholds_are_additive() {
    let expected = [
        ("aurora", 0),
        ("ocean", 1),
        ("sunset", 4),
        ("midnight", 12),
        ("cherry", 16),
        ("ivory", 20),
        ("lagoon", 24),
        ("heather", 28),
        ("sunflower", 36),
        ("slate", 48),
    ];
    assert_eq!(Livery::ALL.len(), expected.len());
    for (livery, (id, stops)) in Livery::ALL.into_iter().zip(expected) {
        let json = serde_json::to_string(&livery).unwrap();
        assert_eq!(json, format!("\"{id}\""));
        assert_eq!(serde_json::from_str::<Livery>(&json).unwrap(), livery);
        assert_eq!(livery.required_stops(), stops);
        let mut snapshot = Journey::default().snapshot();
        snapshot.stations_visited = stops;
        assert!(livery.available(snapshot));
        if stops > 0 {
            snapshot.stations_visited -= 1;
            assert!(!livery.available(snapshot));
        }
    }
}

#[test]
fn advanced_old_v4_keeps_ownership_until_next_arrival_then_grants_once() {
    let mut old = World::default();
    old.restore_journey(Journey::restore_checkpoint(42, 150_000.0, 8_000.0).unwrap());
    assert!(old.journey_snapshot().stations_visited >= 48);
    let mut service = old.service_state().clone();
    service
        .owned_rewards
        .retain(|r| Livery::ORIGINAL.contains(r));
    old.restore_service(service.clone()).unwrap();
    let mut world = roundtrip(&old);
    assert_eq!(world.service_state(), &service);
    let config = Config::default();
    let mut train = Train::new(&config);
    world.advance_expedition(&mut train, 100.0, 0.0, &config);
    assert_eq!(world.service_state(), &service);
    let visits = world.journey_snapshot().stations_visited;
    for _ in 0..1000 {
        world.advance_expedition(&mut train, 0.0, 1.0, &config);
        if world.journey_snapshot().stations_visited > visits {
            break;
        }
    }
    assert!(world.journey_snapshot().stations_visited > visits);
    assert_eq!(world.service_state().owned_rewards.len(), Livery::ALL.len());
    for livery in Livery::ALL {
        assert!(world.service_state().owns(livery));
    }
    let restored = roundtrip(&world);
    assert_eq!(restored.service_state(), world.service_state());
    for _ in 0..1000 {
        world.advance_expedition(&mut train, 100.0, 1.0, &config);
    }
    assert_eq!(world.service_state().owned_rewards.len(), Livery::ALL.len());
    // The original four remain mandatory; expansion does not weaken old checks.
    let mut invalid = old.service_state().clone();
    invalid.owned_rewards.retain(|r| *r != Livery::Midnight);
    assert!(invalid.validate(old.journey_snapshot()).is_err());
}

#[test]
fn new_profiles_grant_only_original_paints_at_their_thresholds() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..20000 {
        world.advance_expedition(&mut train, 40.0, 1.0, &config);
        let snapshot = world.journey_snapshot();
        for livery in Livery::ALL {
            assert_eq!(
                world.service_state().owns(livery),
                Livery::ORIGINAL.contains(&livery) && livery.available(snapshot)
            );
        }
        if snapshot.stations_visited >= 49 {
            return;
        }
    }
    panic!("the active-time journey did not reach the final threshold");
}
