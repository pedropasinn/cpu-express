use cpu_express::{
    config::Config,
    progress,
    simulation::{
        CargoKind, ServiceWagon, Train, World,
        economy::{CrateKind, EconomyAction, ItemId, Rarity, TRANSACTION_LIMIT},
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
        "ce-economy-{}-{}-{}",
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
fn action(world: &mut World, action: EconomyAction) {
    world.economy_action(action, |_| Ok(())).unwrap();
}
fn fund(world: &mut World, coins: u64) {
    world
        .transact_economy(
            |state| state.record("Crédito de teste", coins, 0),
            |_| Ok(()),
        )
        .unwrap();
}
fn reveal(world: &mut World) {
    let transaction_id = world
        .economy()
        .pending_purchase
        .as_ref()
        .unwrap()
        .transaction_id;
    action(world, EconomyAction::Reveal { transaction_id });
}
fn advance(world: &mut World, seconds: usize) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..seconds {
        world.advance_expedition(&mut train, 0.0, 1.0, &config);
    }
}

#[test]
fn new_profile_owns_models_and_only_small_milestones_grant_more() {
    let mut world = World::default();
    assert_eq!(world.economy().balance, 30);
    assert_eq!(world.economy().inventory.len(), 5);
    assert_eq!(
        world.service_state().composition,
        [
            ServiceWagon::Passenger,
            ServiceWagon::Passenger,
            ServiceWagon::Freight
        ]
    );
    world
        .set_service_composition([ServiceWagon::Tank; 3])
        .unwrap();
    assert!(
        world
            .set_service_composition([ServiceWagon::Observation; 3])
            .is_err()
    );
    world
        .set_service_composition([
            ServiceWagon::Passenger,
            ServiceWagon::Passenger,
            ServiceWagon::Freight,
        ])
        .unwrap();
    advance(&mut world, 6000);
    assert!(world.economy().owns(&ItemId::wagon(ServiceWagon::Sleeper)));
    assert!(world.economy().owns(&ItemId::Locomotive("diesel".into())));
    assert!(!world.economy().owns(&ItemId::Locomotive("electric".into())));
    assert!(
        !world
            .economy()
            .owns(&ItemId::wagon(ServiceWagon::Observation))
    );
    assert!(!world.economy().owns(&ItemId::Livery("cherry".into())));
}

#[test]
fn general_pool_is_explicit_complete_and_has_no_empty_weighted_band() {
    let def = CrateKind::GeneralYard.definition().unwrap();
    def.validate().unwrap();
    assert_eq!(def.items.len(), 29);
    assert_eq!(def.weights, [70, 22, 7, 1]);
    assert_eq!(def.effective_weights(9), [0, 0, 7, 1]);
    for wagon in ServiceWagon::ALL {
        assert!(def.items.contains(&ItemId::wagon(wagon)));
    }
    let mut invalid = def.clone();
    invalid.items.retain(|i| i.rarity() != Some(Rarity::Epic));
    assert!(invalid.validate().is_err());
    let mut invalid = def.clone();
    invalid.items.push(def.items[0].clone());
    assert!(invalid.validate().is_err());
    let mut invalid = def;
    invalid.weights[0] += 1;
    assert!(invalid.validate().is_err());
    for kind in CrateKind::ALL.into_iter().skip(1) {
        assert!(kind.definition().is_err());
    }
}

#[test]
fn guarantee_and_novelty_are_persistent_and_deterministic() {
    for seed in 0..32 {
        let mut world = World::with_seed(seed);
        fund(&mut world, 10000);
        let mut duplicate = world.clone();
        let mut misses = 0;
        for _ in 0..35 {
            let before = world.economy().clone();
            let buy = EconomyAction::BuyCrate {
                kind: CrateKind::GeneralYard,
                free: false,
            };
            action(&mut world, buy.clone());
            action(&mut duplicate, buy);
            assert_eq!(world.economy(), duplicate.economy());
            let result = world.economy().pending_purchase.as_ref().unwrap();
            if misses == 9 {
                assert!(result.rarity >= Rarity::Rare);
            }
            if result.duplicate {
                for item in CrateKind::GeneralYard
                    .definition()
                    .unwrap()
                    .items
                    .into_iter()
                    .filter(|i| i.rarity() == Some(result.rarity))
                {
                    assert!(before.owns(&item));
                }
                assert_eq!(result.refund, 12);
            } else {
                assert!(!before.owns(&result.item));
                assert_eq!(result.refund, 0);
            }
            misses = if result.rarity >= Rarity::Rare {
                0
            } else {
                misses + 1
            };
            assert_eq!(
                world.economy().crate_misses[&CrateKind::GeneralYard],
                misses
            );
            reveal(&mut world);
            reveal(&mut duplicate);
        }
    }
}

#[test]
fn free_duplicate_has_no_refund_and_welcome_cannot_repeat() {
    let mut world = World::default();
    world
        .transact_economy(
            |state| {
                state
                    .inventory
                    .extend(CrateKind::GeneralYard.definition()?.items);
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
    action(
        &mut world,
        EconomyAction::BuyCrate {
            kind: CrateKind::GeneralYard,
            free: true,
        },
    );
    let pending = world.economy().pending_purchase.as_ref().unwrap();
    assert!(pending.duplicate);
    assert_eq!((pending.paid, pending.refund), (0, 0));
    assert_eq!(world.economy().balance, 30);
    reveal(&mut world);
    let before = world.economy().clone();
    assert!(
        world
            .economy_action(
                EconomyAction::BuyCrate {
                    kind: CrateKind::GeneralYard,
                    free: true
                },
                |_| Ok(())
            )
            .is_err()
    );
    assert_eq!(world.economy(), &before);
}

#[test]
fn purchase_is_saved_before_reveal_and_reopening_never_pays_again() {
    let path = path();
    let mut world = World::with_seed(7);
    fund(&mut world, 500);
    let buy = EconomyAction::BuyCrate {
        kind: CrateKind::GeneralYard,
        free: false,
    };
    progress::economy_action(&path, &mut world, buy.clone()).unwrap();
    let committed = world.economy().clone();
    for _ in 0..3 {
        world = progress::load_world(&path).unwrap().unwrap();
        assert_eq!(world.economy(), &committed);
        assert!(progress::economy_action(&path, &mut world, buy.clone()).is_err());
    }
    let transaction_id = committed.pending_purchase.unwrap().transaction_id;
    let reveal = EconomyAction::Reveal { transaction_id };
    progress::economy_action(&path, &mut world, reveal.clone()).unwrap();
    assert_eq!(world.economy().balance, committed.balance);
    let mut reopened = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(world.economy(), reopened.economy());
    assert!(progress::economy_action(&path, &mut reopened, reveal).is_err());
}

#[test]
fn write_failures_rollback_wallet_inventory_rng_pity_and_reveal() {
    let path = path();
    fs::write(&path, "arquivo incompatível".as_bytes()).unwrap();
    let mut world = World::with_seed(12);
    fund(&mut world, 1000);
    let before = world.economy().clone();
    for buy in [
        EconomyAction::BuyCrate {
            kind: CrateKind::GeneralYard,
            free: false,
        },
        EconomyAction::BuyCrate {
            kind: CrateKind::GeneralYard,
            free: true,
        },
        EconomyAction::BuyDirect(ItemId::wagon(ServiceWagon::Mail)),
    ] {
        assert!(progress::economy_action(&path, &mut world, buy).is_err());
        assert_eq!(world.economy(), &before);
    }
    action(
        &mut world,
        EconomyAction::BuyCrate {
            kind: CrateKind::GeneralYard,
            free: false,
        },
    );
    let before = world.economy().clone();
    let transaction_id = before.pending_purchase.as_ref().unwrap().transaction_id;
    assert!(
        progress::economy_action(&path, &mut world, EconomyAction::Reveal { transaction_id })
            .is_err()
    );
    assert_eq!(world.economy(), &before);
    assert_eq!(fs::read(&path).unwrap(), "arquivo incompatível".as_bytes());
}

#[test]
fn arrival_income_keeps_remainders_and_does_not_pay_cargo_twice() {
    let mut world = World::with_seed(8);
    advance(&mut world, 10000);
    let state = world.service_state();
    let reward = state.cargo_delivered * 10;
    assert_eq!(
        world.economy().balance,
        30 + state.last_processed_visit + state.passengers_delivered / 4 + reward / 20
    );
    assert_eq!(
        world.economy().passenger_remainder,
        state.passengers_delivered % 4
    );
    assert_eq!(world.economy().contract_remainder, reward % 20);
    assert_eq!(
        world.economy().cargo_by_kind.values().sum::<u64>(),
        state.cargo_delivered
    );
    assert_eq!(world.economy().cargo_by_kind[CargoKind::Ore.id()], 0);
    let before = world.economy().clone();
    world.drain_travel_events().for_each(drop);
    world.set_paused(true);
    advance(&mut world, 100);
    assert_eq!(world.economy(), &before);
}

#[test]
fn machine_bonus_is_bounded_fractional_and_not_saved_as_machine_history() {
    let mut base = World::with_seed(3);
    let mut bonus = base.clone();
    for input in [f32::NAN, f32::INFINITY, -10.0, 0.0] {
        bonus.set_machine_bonus(input);
        assert_eq!(bonus.machine_bonus(), 1.0);
    }
    bonus.set_machine_bonus(9.0);
    assert_eq!(bonus.machine_bonus(), 1.3);
    advance(&mut base, 3000);
    advance(&mut bonus, 3000);
    let income = base.economy().balance - 30;
    assert_eq!(bonus.economy().balance - 30, income * 1300 / 1000);
    assert_eq!(bonus.economy().bonus_remainder, income * 1300 % 1000);
    assert_eq!(base.service_state(), bonus.service_state());
    let path = path();
    progress::save_checkpoint(&path, &bonus).unwrap();
    let reopened = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(reopened.economy(), bonus.economy());
    assert_eq!(reopened.machine_bonus(), 1.0);
}

#[test]
fn direct_purchase_equips_multiple_slots_and_survives_restart() {
    let path = path();
    let mut world = World::default();
    fund(&mut world, 1000);
    let item = ItemId::wagon(ServiceWagon::Fair);
    progress::economy_action(&path, &mut world, EconomyAction::BuyDirect(item.clone())).unwrap();
    assert_eq!(world.economy().balance, 550);
    world
        .set_service_composition([ServiceWagon::Fair; 3])
        .unwrap();
    progress::save_checkpoint(&path, &world).unwrap();
    let mut reopened = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(
        reopened.service_state().composition,
        [ServiceWagon::Fair; 3]
    );
    assert!(
        progress::economy_action(&path, &mut reopened, EconomyAction::BuyDirect(item)).is_err()
    );
    assert_eq!(world.economy(), reopened.economy());
}

#[test]
fn bounded_log_retains_monotonic_ids_and_balances() {
    let mut world = World::default();
    for _ in 0..200 {
        fund(&mut world, 1);
    }
    assert_eq!(world.economy().transactions.len(), TRANSACTION_LIMIT);
    assert_eq!(world.economy().next_transaction_id, 202);
    world.economy().validate().unwrap();
}

#[test]
fn e1b_reserved_state_roundtrips_and_failed_claim_does_not_mark_paid() {
    use cpu_express::simulation::economy::DailyOrder;
    let path = path();
    let mut world = World::default();
    progress::transact_economy(&path, &mut world, |state| {
        state.paid_missions.insert("ce.fixture-paid".into());
        state.legacy_mission_claim_pending = false;
        state.daily_orders.push(DailyOrder {
            id: 1,
            template_id: "fixture-arrivals".into(),
            baselines: [("arrivals".into(), 8)].into_iter().collect(),
            goal: 6,
            reward: 10,
            entered_active_seconds: 600.0,
            telegram: false,
        });
        state.next_order_id = 2;
        state.last_order_active_seconds = 600.0;
        state.last_order_swap_active_seconds = 1200.0;
        state.museum_stamps = 2;
        state.record("Prêmio de fixture", 10, 0)
    })
    .unwrap();
    let restored = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(restored.economy(), world.economy());
    let before = world.economy().clone();
    assert!(
        world
            .transact_economy(
                |state| {
                    state.paid_missions.insert("ce.fixture-failed".into());
                    state.record("Prêmio que não foi salvo", 25, 0)
                },
                |_| Err("Falha de gravação simulada".into())
            )
            .is_err()
    );
    assert_eq!(world.economy(), &before);
}

#[test]
fn arrival_failure_rolls_back_the_entire_domain_before_notification() {
    let path = path();
    fs::write(&path, "inválido".as_bytes()).unwrap();
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    while !world.may_arrive(1.0) {
        world.advance_expedition(&mut train, 0.0, 1.0, &config);
    }
    let snapshot = world.journey_snapshot();
    let service = world.service_state().clone();
    let economy = world.economy().clone();
    let speed = train.speed_kmh();
    assert!(
        progress::advance_checkpoint(&path, &mut world, &mut train, 0.0, 1.0, &config).is_err()
    );
    assert_eq!(world.journey_snapshot(), snapshot);
    assert_eq!(world.service_state(), &service);
    assert_eq!(world.economy(), &economy);
    assert_eq!(train.speed_kmh(), speed);
    let good = self::path();
    progress::advance_checkpoint(&good, &mut world, &mut train, 0.0, 1.0, &config).unwrap();
    let reopened = progress::load_world(&good).unwrap().unwrap();
    assert_eq!(world.economy(), reopened.economy());
    assert_eq!(world.service_state(), reopened.service_state());
}
