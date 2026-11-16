//! Static v1-v5 migration and recovery evidence, always in isolated directories.
use cpu_express::{
    progress,
    simulation::{
        ServiceWagon,
        economy::{EconomyAction, ItemId},
    },
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn path() -> PathBuf {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let folder = std::env::temp_dir().join(format!(
        "ce-economy-fixture-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&folder).unwrap();
    folder.join("config.journey.json")
}
const LEGACY: [&[u8]; 4] = [
    include_bytes!("fixtures/economy/v1.json"),
    include_bytes!("fixtures/economy/v2.json"),
    include_bytes!("fixtures/economy/v3.json"),
    include_bytes!("fixtures/economy/v4.json"),
];
const NEW: &[u8] = include_bytes!("fixtures/economy/v5-new.json");
const PENDING: &[u8] = include_bytes!("fixtures/economy/v5-pending.json");

#[test]
fn all_legacy_versions_keep_rights_and_exact_backups_without_retroactive_cargo() {
    for (index, original) in LEGACY.into_iter().enumerate() {
        let path = path();
        fs::write(&path, original).unwrap();
        let world = progress::load_world(&path).unwrap().unwrap();
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(world.economy().balance, 30);
        assert!(world.economy().welcome_crate_available);
        assert!(world.economy().legacy_mission_claim_pending);
        assert!(world.economy().cargo_by_kind.values().all(|v| *v == 0));
        for wagon in ServiceWagon::ALL {
            assert_eq!(
                world.economy().owns(&ItemId::wagon(wagon)),
                wagon.available(world.journey_snapshot().stations_visited)
            );
        }
        // v2-v4 fixtures have both postcards; each pre-existing engine is acquired.
        if index > 0 {
            assert_eq!(world.service_state().cargo_delivered, 20);
            assert_eq!(world.service_state().credits, 240);
            for engine in cpu_express::rendering::EngineStyle::ALL {
                assert!(
                    world
                        .economy()
                        .owns(&ItemId::Locomotive(engine.id().into()))
                );
            }
        }
        progress::save_checkpoint(&path, &world).unwrap();
        let backup = path.with_extension(format!("v{}.backup.json", index + 1));
        assert_eq!(fs::read(&backup).unwrap(), original);
        let bytes = fs::read(&path).unwrap();
        let reopened = progress::load_world(&path).unwrap().unwrap();
        assert_eq!(reopened.economy(), world.economy());
        assert_eq!(reopened.service_state(), world.service_state());
        progress::save_checkpoint(&path, &reopened).unwrap();
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(fs::read(&backup).unwrap(), original);
    }
}

#[test]
fn v5_fixtures_roundtrip_every_reserved_field_and_pending_reveal() {
    for original in [NEW, PENDING] {
        let path = path();
        fs::write(&path, original).unwrap();
        let mut world = progress::load_world(&path).unwrap().unwrap();
        let before = world.economy().clone();
        let lines = world.line_progress();
        progress::save_checkpoint(&path, &world).unwrap();
        let reopened = progress::load_world(&path).unwrap().unwrap();
        assert_eq!(reopened.economy(), &before);
        assert_eq!(reopened.line_progress(), lines);
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(value["version"], 6);
        assert_eq!(value["lines"].as_object().unwrap().len(), 1);
        for field in [
            "inventory",
            "balance",
            "transactions",
            "next_transaction_id",
            "pending_purchase",
            "cargo_by_kind",
            "crate_misses",
            "random_state",
            "paid_missions",
            "daily_orders",
            "next_order_id",
            "museum_stamps",
            "legacy_mission_claim_pending",
        ] {
            assert!(value["economy"].get(field).is_some(), "{field}");
        }
        if let Some(pending) = &before.pending_purchase {
            progress::economy_action(
                &path,
                &mut world,
                EconomyAction::Reveal {
                    transaction_id: pending.transaction_id,
                },
            )
            .unwrap();
            assert_eq!(world.economy().balance, before.balance);
            assert_eq!(world.economy().inventory, before.inventory);
            assert_eq!(world.economy().random_state, before.random_state);
            assert!(world.economy().pending_purchase.is_none());
        }
    }
}

#[test]
fn missing_or_inconsistent_v5_fields_are_not_repaired_or_overwritten() {
    let original: serde_json::Value = serde_json::from_slice(NEW).unwrap();
    let cases = [
        "economy",
        "lines",
        "balance",
        "random_state",
        "cargo_by_kind",
        "pending_purchase",
        "paid_missions",
        "crate_misses",
    ];
    for field in cases {
        let mut invalid = original.clone();
        if field == "economy" || field == "lines" {
            invalid.as_object_mut().unwrap().remove(field);
        } else {
            invalid["economy"].as_object_mut().unwrap().remove(field);
        }
        let path = path();
        let bytes = serde_json::to_vec(&invalid).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(progress::load_world(&path).is_err(), "{field}");
        assert!(
            progress::save_checkpoint(&path, &cpu_express::simulation::World::default()).is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    for invalid in [
        {
            let mut v = original.clone();
            v["economy"]["balance"] = 999.into();
            v
        },
        {
            let mut v = original.clone();
            v["economy"]["passenger_remainder"] = 4.into();
            v
        },
        {
            let mut v = original.clone();
            v["economy"]["inventory"] = serde_json::json!([]);
            v
        },
        {
            let mut v = original.clone();
            v["economy"]["cargo_by_kind"]["ore"] = 1.into();
            v
        },
        {
            let mut v = original.clone();
            v["lines"]["aurora-v1"]["distance"] = 1.into();
            v
        },
    ] {
        let path = path();
        fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(progress::load_world(&path).is_err());
    }
}

#[test]
fn v4_backup_conflict_and_future_save_never_destroy_originals() {
    let path = path();
    fs::write(&path, LEGACY[3]).unwrap();
    let world = progress::load_world(&path).unwrap().unwrap();
    let backup = path.with_extension("v4.backup.json");
    fs::write(&backup, b"outro backup").unwrap();
    assert!(progress::save_checkpoint(&path, &world).is_err());
    assert_eq!(fs::read(&path).unwrap(), LEGACY[3]);
    assert_eq!(fs::read(&backup).unwrap(), b"outro backup");
    let future = br#"{"version":7}"#;
    fs::write(&path, future).unwrap();
    fs::write(progress::previous_path(&path), NEW).unwrap();
    assert!(progress::load_profile(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), future);
}

#[test]
fn recovery_of_pending_purchase_keeps_result_and_does_not_repeat_refund() {
    let path = path();
    fs::write(&path, b"danificado").unwrap();
    fs::write(progress::previous_path(&path), PENDING).unwrap();
    let (world, notice) = progress::load_profile(&path).unwrap();
    assert!(notice.is_some());
    let world = world.unwrap();
    assert!(world.economy().pending_purchase.is_some());
    progress::save_checkpoint(&path, &world).unwrap();
    let reopened = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(world.economy(), reopened.economy());
}
