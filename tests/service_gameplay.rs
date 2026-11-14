use cpu_express::{
    config::Config,
    progress,
    simulation::{CargoKind, Journey, ServiceWagon, Train, World},
};
fn advance(world: &mut World, seconds: usize, parts: usize) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..seconds * parts {
        world.advance_expedition(&mut train, 37.0, 1.0 / parts as f64, &config);
    }
}
fn path(tag: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cpu-service-{}-{nonce}-{tag}.json",
        std::process::id()
    ))
}
#[test]
fn passenger_capacity_pause_and_partition() {
    let mut a = World::default();
    let mut b = World::default();
    advance(&mut a, 700, 1);
    advance(&mut b, 700, 4);
    assert_eq!(a.service_state(), b.service_state());
    assert!(a.service_state().passengers_delivered > 0);
    assert!(a.service_state().passenger_count() <= a.service_state().capacity().0);
    let before = a.service_state().clone();
    a.set_paused(true);
    advance(&mut a, 300, 2);
    assert_eq!(&before, a.service_state());
}
#[test]
fn restart_during_station_service_does_not_replay() {
    let mut a = World::default();
    while a.station_wait() == 0.0 {
        advance(&mut a, 1, 1);
    }
    let file = path("restart");
    progress::save(&file, &a).unwrap();
    let mut b = progress::load_world(&file).unwrap().unwrap();
    assert_eq!(a.service_state(), b.service_state());
    advance(&mut a, 1100, 2);
    advance(&mut b, 1100, 2);
    assert_eq!(a.service_state(), b.service_state());
    assert!(a.service_state().contracts_completed > 0);
    progress::save(&file, &b).unwrap();
    assert_eq!(
        b.service_state(),
        progress::load_world(&file)
            .unwrap()
            .unwrap()
            .service_state()
    );
}
#[test]
fn tank_capacity_is_not_generic_and_downsize_is_rejected() {
    let mut w = World::default();
    w.set_service_composition([ServiceWagon::Tank; 3]).unwrap();
    assert_eq!(w.service_state().cargo_capacity(CargoKind::Parcels), 0);
    assert_eq!(w.service_state().cargo_capacity(CargoKind::Liquid), 36);
    while w.service_state().cargo_count() == 0 {
        advance(&mut w, 1, 1);
    }
    assert_eq!(
        w.service_state().contract.as_ref().unwrap().kind,
        CargoKind::Liquid
    );
    assert!(
        w.set_service_composition([ServiceWagon::Freight; 3])
            .is_err()
    );
    let mut passengers = World::default();
    advance(&mut passengers, 120, 1);
    assert!(passengers.service_state().passenger_count() > 0);
    assert!(
        passengers
            .set_service_composition([ServiceWagon::Freight; 3])
            .is_err()
    );
}
#[test]
fn v1_migration_preserves_unlocks_and_original_bytes() {
    let journey = Journey::restore_checkpoint(4, 40000.0, 2000.0).unwrap();
    let old = progress::Checkpoint::capture(journey.snapshot());
    let bytes = serde_json::to_vec_pretty(&old).unwrap();
    let file = path("migration");
    std::fs::write(&file, &bytes).unwrap();
    let w = progress::load_world(&file).unwrap().unwrap();
    assert_eq!(w.service_state().owned_rewards.len(), 4);
    assert_eq!(w.service_state().passengers_delivered, 0);
    progress::save(&file, &w).unwrap();
    assert_eq!(
        std::fs::read(file.with_extension("v1.backup.json")).unwrap(),
        bytes
    );
    assert_eq!(
        progress::load_world(&file)
            .unwrap()
            .unwrap()
            .service_state(),
        w.service_state()
    );
}
#[test]
fn invalid_v2_is_preserved_and_history_is_bounded() {
    let mut w = World::default();
    advance(&mut w, 4500, 1);
    assert_eq!(w.service_state().arrivals.len(), 32);
    let file = path("invalid");
    progress::save(&file, &w).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    value["service"]["credits"] = serde_json::json!(999999999);
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(&file, &bytes).unwrap();
    assert!(progress::load_world(&file).is_err());
    assert!(progress::save(&file, &w).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), bytes);
}
