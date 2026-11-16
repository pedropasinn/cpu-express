use cpu_express::{
    config::Config,
    progress::{self, Checkpoint, WorldCheckpoint},
    simulation::{CargoContract, ContractStatus, ServiceWagon, Train, World},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const SEED: u64 = 41;
const MAX_SECONDS: usize = 2_000;

fn advance_until(world: &mut World, predicate: impl Fn(&World) -> bool) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..MAX_SECONDS {
        if predicate(world) {
            return;
        }
        world.advance_expedition(&mut train, 37.0, 1.0, &config);
    }
    assert!(
        predicate(world),
        "condition not reached in {MAX_SECONDS} active seconds: {:?}",
        world.service_state()
    );
}

fn next_arrival(world: &mut World) {
    let visit = world.journey_snapshot().stations_visited;
    advance_until(world, |w| w.journey_snapshot().stations_visited > visit);
}

fn contract(world: &World) -> &CargoContract {
    world.service_state().contract.as_ref().expect("contract")
}

fn manual_offer() -> World {
    let mut world = World::with_seed(SEED);
    world.set_automatic_contracts(false);
    // No passenger earnings can mask a duplicated cargo reward.
    world
        .set_service_composition([ServiceWagon::Freight; 3])
        .unwrap();
    next_arrival(&mut world);
    assert_eq!(contract(&world).status, ContractStatus::Offered);
    world
}

fn file(tag: &str) -> PathBuf {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "cpu-dispatch-{}-{nonce}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    // Deliberately retain isolated evidence; never touch the user's profile.
    dir.join(format!("{tag}.json"))
}

fn restart(world: &World) -> World {
    let path = file("restart");
    progress::save(&path, world).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(value["version"], 6);
    let restored = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(restored.service_state(), world.service_state());
    assert_eq!(restored.station_wait(), world.station_wait());
    assert_eq!(
        restored.journey_snapshot().total_distance_units,
        world.journey_snapshot().total_distance_units
    );
    restored
}

#[test]
fn manual_offer_never_expires_and_refusal_replaces_only_on_next_arrival() {
    let mut world = manual_offer();
    let offered = contract(&world).clone();
    for _ in 0..8 {
        next_arrival(&mut world);
        assert_eq!(contract(&world), &offered);
        assert_eq!(world.service_state().cargo_count(), 0);
        assert_eq!(world.service_state().credits, 0);
    }
    world.cancel_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Cancelled);
    let cancelled = world.service_state().clone();
    assert!(world.cancel_cargo_contract().is_err());
    assert!(world.accept_cargo_contract().is_err());
    assert_eq!(world.service_state(), &cancelled);
    next_arrival(&mut world);
    assert_eq!(contract(&world).status, ContractStatus::Offered);
    assert_ne!(contract(&world).id, offered.id);
}

#[test]
fn accept_at_origin_loads_once_and_cancel_at_origin_returns_once() {
    let mut world = manual_offer();
    let units = contract(&world).units;
    assert!(world.station_wait() > 0.0);
    world.accept_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Loaded);
    assert_eq!(world.service_state().cargo_count(), units);
    assert_eq!(
        world.service_state().arrivals.last().unwrap().cargo_loaded,
        units
    );
    let loaded = world.service_state().clone();
    assert!(world.accept_cargo_contract().is_err());
    assert_eq!(world.service_state(), &loaded);
    world.cancel_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Cancelled);
    assert_eq!(world.service_state().cargo_count(), 0);
    assert_eq!(
        world
            .service_state()
            .arrivals
            .last()
            .unwrap()
            .cargo_returned,
        units
    );
    let mut restored = restart(&world);
    assert!(restored.cancel_cargo_contract().is_err());
    next_arrival(&mut restored);
    assert_eq!(restored.service_state().credits, 0);
    assert_eq!(restored.service_state().contracts_completed, 0);
    assert_eq!(
        restored
            .service_state()
            .arrivals
            .iter()
            .map(|a| a.cargo_returned)
            .sum::<u32>(),
        units
    );
}

#[test]
fn acceptance_without_capacity_is_atomic_and_can_be_retried() {
    let mut world = manual_offer();
    for stock in [ServiceWagon::Passenger, ServiceWagon::Tank] {
        world.set_service_composition([stock; 3]).unwrap();
        let before = world.service_state().clone();
        assert!(world.accept_cargo_contract().is_err());
        assert_eq!(world.service_state(), &before);
    }
    world
        .set_service_composition([ServiceWagon::Freight; 3])
        .unwrap();
    world.accept_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Loaded);
}

#[test]
fn accept_after_service_waits_for_next_origin_and_accepted_can_be_cancelled() {
    let mut world = manual_offer();
    let origin = contract(&world).origin;
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..8 {
        let remaining = world.station_wait();
        if remaining == 0.0 {
            break;
        }
        world.advance_expedition(&mut train, 37.0, remaining.min(1.0), &config);
    }
    assert_eq!(world.station_wait(), 0.0);
    // Still geometrically at the origin, but the service window is over.
    assert_eq!(world.journey_snapshot().at_station, Some(origin));
    world.accept_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Accepted);
    assert_eq!(world.service_state().cargo_count(), 0);
    let mut cancelled = restart(&world);
    cancelled.cancel_cargo_contract().unwrap();
    assert_eq!(contract(&cancelled).status, ContractStatus::Cancelled);
    next_arrival(&mut cancelled);
    assert_eq!(cancelled.service_state().cargo_count(), 0);
    assert_eq!(cancelled.service_state().credits, 0);
    for _ in 0..3 {
        next_arrival(&mut world);
        assert_eq!(contract(&world).status, ContractStatus::Accepted);
        assert_eq!(world.service_state().cargo_count(), 0);
    }
    next_arrival(&mut world);
    assert_eq!(world.journey_snapshot().at_station, Some(origin));
    assert_eq!(contract(&world).status, ContractStatus::Loaded);
}

#[test]
fn loaded_restart_delivers_and_pays_exactly_once() {
    let mut world = manual_offer();
    world.accept_cargo_contract().unwrap();
    let cargo = contract(&world).clone();
    let mut restored = restart(&world);
    advance_until(&mut world, |w| {
        contract(w).status == ContractStatus::Delivered
    });
    advance_until(&mut restored, |w| {
        contract(w).status == ContractStatus::Delivered
    });
    assert_eq!(world.service_state(), restored.service_state());
    assert_eq!(restored.service_state().credits, cargo.reward);
    assert_eq!(
        restored.service_state().cargo_delivered,
        u64::from(cargo.units)
    );
    assert_eq!(restored.service_state().contracts_completed, 1);
    restored = restart(&restored);
    assert!(restored.cancel_cargo_contract().is_err());
    restored.drain_travel_events().for_each(drop);
    for _ in 0..5 {
        next_arrival(&mut restored);
    }
    assert_eq!(restored.service_state().credits, cargo.reward);
    assert_eq!(restored.service_state().contracts_completed, 1);
    assert_eq!(
        restored.service_state().cargo_delivered,
        u64::from(cargo.units)
    );
}

#[test]
fn returning_restart_keeps_capacity_until_origin_and_never_pays() {
    let mut world = manual_offer();
    world.accept_cargo_contract().unwrap();
    let cargo = contract(&world).clone();
    advance_until(&mut world, |w| w.station_wait() == 0.0);
    world.cancel_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Returning);
    let mut restored = restart(&world);
    for _ in 0..3 {
        let before = restored.service_state().clone();
        assert!(
            restored
                .set_service_composition([ServiceWagon::Tank; 3])
                .is_err()
        );
        assert!(restored.cancel_cargo_contract().is_err());
        assert_eq!(restored.service_state(), &before);
        next_arrival(&mut restored);
        assert_eq!(contract(&restored).status, ContractStatus::Returning);
        assert_eq!(restored.service_state().cargo_count(), cargo.units);
        assert_eq!(restored.service_state().credits, 0);
    }
    next_arrival(&mut restored);
    assert_eq!(restored.journey_snapshot().at_station, Some(cargo.origin));
    assert_eq!(contract(&restored).status, ContractStatus::Cancelled);
    assert_eq!(restored.service_state().cargo_count(), 0);
    assert_eq!(
        restored
            .service_state()
            .arrivals
            .last()
            .unwrap()
            .cargo_returned,
        cargo.units
    );
    restored = restart(&restored);
    restored
        .set_service_composition([ServiceWagon::Tank; 3])
        .unwrap();
    for _ in 0..5 {
        next_arrival(&mut restored);
    }
    assert_eq!(restored.service_state().credits, 0);
    assert_eq!(restored.service_state().cargo_delivered, 0);
    assert_eq!(restored.service_state().contracts_completed, 0);
    assert_eq!(
        restored
            .service_state()
            .arrivals
            .iter()
            .map(|a| a.cargo_returned)
            .sum::<u32>(),
        cargo.units
    );
}

#[test]
fn automatic_default_still_loads_delivers_and_survives_restart() {
    let mut world = World::with_seed(SEED);
    assert!(world.service_state().automatic_contracts);
    world
        .set_service_composition([ServiceWagon::Freight; 3])
        .unwrap();
    next_arrival(&mut world);
    assert_eq!(contract(&world).status, ContractStatus::Loaded);
    let reward = contract(&world).reward;
    let mut restored = restart(&world);
    advance_until(&mut restored, |w| {
        contract(w).status == ContractStatus::Delivered
    });
    assert_eq!(restored.service_state().credits, reward);
    assert_eq!(restored.service_state().contracts_completed, 1);
    next_arrival(&mut restored);
    assert_eq!(contract(&restored).status, ContractStatus::Loaded);
}

fn legacy_v2(world: &World) -> Vec<u8> {
    let mut journey = Checkpoint::capture(world.journey_snapshot());
    journey.station_wait = world.station_wait();
    let mut value = serde_json::to_value(WorldCheckpoint {
        version: 2,
        catalog_version: 0,
        line: String::new(),
        journey,
        service: world.service_state().clone(),
        economy: None,
        lines: None,
        line_services: None,
        boarding_closed: None,
    })
    .unwrap();
    // Fields introduced by v4 never existed in a v2 document.
    value.as_object_mut().unwrap().remove("catalog_version");
    value.as_object_mut().unwrap().remove("line");
    let service = value["service"].as_object_mut().unwrap();
    service.remove("automatic_contracts");
    for arrival in service["arrivals"].as_array_mut().unwrap() {
        arrival.as_object_mut().unwrap().remove("cargo_returned");
    }
    // Noncanonical whitespace makes byte preservation stricter than JSON equality.
    format!(
        " \r\n{}\r\n\t",
        serde_json::to_string_pretty(&value).unwrap()
    )
    .into_bytes()
}

#[test]
fn v2_missing_fields_defaults_to_auto_and_backup_preserves_exact_bytes() {
    let mut world = World::with_seed(SEED);
    next_arrival(&mut world);
    let original = legacy_v2(&world);
    let path = file("v2");
    fs::write(&path, &original).unwrap();
    let mut migrated = progress::load_world(&path).unwrap().unwrap();
    assert!(migrated.service_state().automatic_contracts);
    assert!(!migrated.service_state().arrivals.is_empty());
    assert!(
        migrated
            .service_state()
            .arrivals
            .iter()
            .all(|a| a.cargo_returned == 0)
    );
    assert_eq!(migrated.service_state(), world.service_state());
    assert_eq!(fs::read(&path).unwrap(), original);
    progress::save(&path, &migrated).unwrap();
    let backup = path.with_extension("v2.backup.json");
    assert_eq!(fs::read(&backup).unwrap(), original);
    let current: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(current["version"], 6);
    next_arrival(&mut migrated);
    progress::save(&path, &migrated).unwrap();
    assert_eq!(fs::read(&backup).unwrap(), original);
    assert_eq!(
        progress::load_world(&path)
            .unwrap()
            .unwrap()
            .service_state(),
        migrated.service_state()
    );
}

#[test]
fn v2_conflicting_backup_blocks_migration_without_overwriting_either_file() {
    let world = World::with_seed(SEED);
    let original = legacy_v2(&world);
    let path = file("conflict");
    let backup = path.with_extension("v2.backup.json");
    fs::write(&path, &original).unwrap();
    fs::write(&backup, b"existing backup evidence").unwrap();
    assert!(progress::save(&path, &world).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(fs::read(&backup).unwrap(), b"existing backup evidence");
}

#[test]
fn future_checkpoint_is_protected_even_when_replaced_after_load() {
    let world = World::with_seed(SEED);
    let path = file("future");
    progress::save(&path, &world).unwrap();
    let loaded = progress::load_world(&path).unwrap().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["version"] = serde_json::json!(7);
    let future = serde_json::to_vec_pretty(&value).unwrap();
    fs::write(&path, &future).unwrap();
    assert!(progress::load_world(&path).is_err());
    assert!(progress::save(&path, &loaded).is_err());
    assert_eq!(fs::read(&path).unwrap(), future);
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn incomplete_v3_is_rejected_without_silently_reactivating_automatic_dispatch() {
    let world = manual_offer();
    for field in ["automatic_contracts", "branch", "cargo_returned"] {
        let path = file(field);
        progress::save(&path, &world).unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        if field == "cargo_returned" {
            value["service"]["arrivals"][0]
                .as_object_mut()
                .unwrap()
                .remove(field);
        } else {
            value["service"].as_object_mut().unwrap().remove(field);
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(progress::load_world(&path).is_err());
        assert!(progress::save(&path, &world).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn returned_lot_cannot_be_restored_as_cargo_in_transit() {
    let mut world = manual_offer();
    world.accept_cargo_contract().unwrap();
    world.cancel_cargo_contract().unwrap();
    assert_eq!(contract(&world).status, ContractStatus::Cancelled);
    assert!(
        world
            .service_state()
            .arrivals
            .last()
            .unwrap()
            .cargo_returned
            > 0
    );
    let path = file("resurrected-cargo");
    progress::save(&path, &world).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["service"]["contract"]["status"] = serde_json::json!("returning");
    let bytes = serde_json::to_vec(&value).unwrap();
    fs::write(&path, &bytes).unwrap();
    assert!(progress::load_world(&path).is_err());
    assert!(progress::save(&path, &world).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
}
