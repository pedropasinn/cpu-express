use cpu_express::{
    config::Config,
    progress,
    simulation::{
        ServiceWagon, Train, World,
        economy::{CrateKind, EconomyAction, EconomyState, ItemId, Rarity},
        economy_goals::ORDER_TEMPLATES,
    },
};

fn advance(w: &mut World, seconds: usize) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..seconds {
        w.advance_expedition(&mut train, 0.0, 1.0, &config);
    }
}
fn action(w: &mut World, a: EconomyAction) {
    w.economy_action(a, |_| Ok(())).unwrap();
}
fn path() -> std::path::PathBuf {
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "ce-e1b-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p.join("journey.json")
}

#[test]
fn all_five_pools_have_explicit_ids_valid_bands_and_independent_guarantees() {
    let sizes = [29, 20, 7, 10, 10];
    let prices = [150, 225, 225, 450, 188];
    for ((kind, size), price) in CrateKind::ALL.into_iter().zip(sizes).zip(prices) {
        let d = kind.definition().unwrap();
        d.validate().unwrap();
        assert_eq!((d.items.len(), d.price), (size, price));
        if matches!(kind, CrateKind::PlatformComfort | CrateKind::PaintAtelier) {
            assert_eq!(d.weights[3], 0);
        }
        for seed in 0..20 {
            let mut world = World::with_seed(seed);
            world
                .transact_economy(
                    |s| {
                        s.record("Teste", 500, 0)?;
                        s.crate_misses.insert(kind, d.guarantee - 1);
                        Ok(())
                    },
                    |_| Ok(()),
                )
                .unwrap();
            action(&mut world, EconomyAction::BuyCrate { kind, free: false });
            let p = world.economy().pending_purchase.as_ref().unwrap();
            assert!(p.rarity >= Rarity::Rare && d.items.contains(&p.item));
            assert_eq!(world.economy().crate_misses[&kind], 0);
            for other in CrateKind::ALL {
                if other != kind {
                    assert_eq!(world.economy().crate_misses[&other], 0);
                }
            }
            let pth = path();
            progress::save_checkpoint(&pth, &world).unwrap();
            let reopened = progress::load_world(&pth).unwrap().unwrap();
            assert_eq!(world.economy(), reopened.economy());
        }
    }
    let union: std::collections::BTreeSet<_> = CrateKind::ALL
        .into_iter()
        .flat_map(|k| k.definition().unwrap().items)
        .collect();
    assert_eq!(
        union,
        EconomyState::collection_items().into_iter().collect()
    );
}

#[test]
fn mission_claim_is_atomic_single_and_durable() {
    let mut world = World::default();
    advance(&mut world, 4000);
    let claim = EconomyAction::ClaimMission("ce.stations-4".into());
    let before = world.economy().clone();
    assert!(
        world
            .economy_action(claim.clone(), |_| Err("Disco indisponível".into()))
            .is_err()
    );
    assert_eq!(world.economy(), &before);
    let p = path();
    progress::economy_action(&p, &mut world, claim.clone()).unwrap();
    assert_eq!(world.economy().balance, before.balance + 20);
    let mut reopened = progress::load_world(&p).unwrap().unwrap();
    assert!(progress::economy_action(&p, &mut reopened, claim).is_err());
    assert_eq!(world.economy(), reopened.economy());
    assert!(
        world
            .economy_action(EconomyAction::ClaimMission("ce.ore-24".into()), |_| Ok(()))
            .is_err()
    );
}

#[test]
fn legacy_missions_settle_on_load_with_exact_backup_and_notice_once() {
    for version in 1..=4 {
        let bytes = std::fs::read(format!("tests/fixtures/economy/v{version}.json")).unwrap();
        let p = path();
        std::fs::write(&p, &bytes).unwrap();
        let before = progress::load_world(&p).unwrap().unwrap();
        let reward: u64 = before
            .economy()
            .missions(before.journey_snapshot(), before.service_state())
            .iter()
            .filter(|m| m.claimable())
            .map(|m| m.reward)
            .sum();
        let (world, notice) = progress::load_profile(&p).unwrap();
        let world = world.unwrap();
        assert!(notice.unwrap().contains("Marcos antigos"));
        assert!(!world.economy().legacy_mission_claim_pending);
        assert_eq!(world.economy().balance, before.economy().balance + reward);
        assert_eq!(
            std::fs::read(p.with_extension(format!("v{version}.backup.json"))).unwrap(),
            bytes
        );
        let (again, notice) = progress::load_profile(&p).unwrap();
        assert!(notice.is_none());
        assert_eq!(world.economy(), again.unwrap().economy());
    }
}

#[test]
fn failed_legacy_write_keeps_loaded_journey_and_pending_claim() {
    let bytes = std::fs::read("tests/fixtures/economy/v4.json").unwrap();
    let p = path();
    std::fs::write(&p, &bytes).unwrap();
    // An unrelated existing backup must block migration without replacing the journey.
    std::fs::write(p.with_extension("v4.backup.json"), b"backup diferente").unwrap();
    let before = progress::load_world(&p).unwrap().unwrap();
    let (after, notice) = progress::load_profile(&p).unwrap();
    let after = after.unwrap();
    assert!(notice.unwrap().contains("ainda não foram pagos"));
    assert_eq!(after.journey_snapshot(), before.journey_snapshot());
    assert_eq!(after.economy(), before.economy());
    assert!(after.economy().legacy_mission_claim_pending);
    assert_eq!(std::fs::read(&p).unwrap(), bytes);
}

#[test]
fn orders_use_active_time_baselines_slots_and_single_payment_after_restart() {
    let mut world = World::default();
    advance(&mut world, 599);
    assert!(world.economy().daily_orders.is_empty());
    world.set_paused(true);
    advance(&mut world, 1000);
    assert!(world.economy().daily_orders.is_empty());
    world.set_paused(false);
    advance(&mut world, 1800);
    assert_eq!(world.economy().daily_orders.len(), 3);
    let mut times: Vec<_> = world
        .economy()
        .daily_orders
        .iter()
        .map(|o| o.entered_active_seconds)
        .collect();
    times.sort_by(f64::total_cmp);
    assert!(times.windows(2).all(|t| t[1] - t[0] >= 600.0));
    world.request_telegram();
    advance(&mut world, 700);
    assert_eq!(world.economy().daily_orders.len(), 4);
    assert_eq!(
        world
            .economy()
            .daily_orders
            .iter()
            .filter(|o| o.telegram)
            .count(),
        1
    );
    world.request_telegram();
    advance(&mut world, 6000);
    assert_eq!(world.economy().daily_orders.len(), 4);
    let o = world
        .economy()
        .daily_orders
        .iter()
        .find(|o| {
            o.claimable(
                world.economy(),
                world.journey_snapshot(),
                world.service_state(),
            )
        })
        .unwrap()
        .clone();
    let before = world.economy().clone();
    let claim = EconomyAction::ClaimOrder { order_id: o.id };
    assert!(
        world
            .economy_action(claim.clone(), |_| Err("Falha".into()))
            .is_err()
    );
    assert_eq!(world.economy(), &before);
    let p = path();
    progress::economy_action(&p, &mut world, claim.clone()).unwrap();
    assert_eq!(world.economy().balance, before.balance + o.reward);
    assert_eq!(world.economy().daily_orders.len(), 3);
    let mut reopened = progress::load_world(&p).unwrap().unwrap();
    assert!(reopened.economy_action(claim, |_| Ok(())).is_err());
    assert_eq!(world.economy(), reopened.economy());
}

#[test]
fn owned_specialist_must_fit_occupied_composition_to_enable_order() {
    use cpu_express::simulation::{PassengerGroup, StationId};
    let mut world = World::default();
    world
        .transact_economy(
            |s| {
                s.inventory.insert(ItemId::wagon(ServiceWagon::Gondola));
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
    let mut service = world.service_state().clone();
    service.composition = [ServiceWagon::Passenger; 3];
    service.passengers = vec![PassengerGroup {
        origin: StationId::Aurora,
        destination: StationId::Pines,
        count: 72,
    }];
    let ore = ORDER_TEMPLATES.iter().find(|t| t.id == "ore").unwrap();
    assert!(!world.economy().order_eligible(ore, &service));
    service.passengers[0].count = 48;
    assert!(world.economy().order_eligible(ore, &service));
    let chilled = ORDER_TEMPLATES.iter().find(|t| t.id == "chilled").unwrap();
    assert!(!world.economy().order_eligible(chilled, &service));
}

#[test]
fn museum_requires_full_collection_and_persists_monotonic_numbers_atomically() {
    let mut world = World::default();
    assert!(
        world
            .economy_action(EconomyAction::BuyMuseumStamp, |_| Ok(()))
            .is_err()
    );
    world
        .transact_economy(
            |s| {
                s.inventory.extend(EconomyState::collection_items());
                s.record("Teste", 500, 0)?;
                Ok(())
            },
            |_| Ok(()),
        )
        .unwrap();
    let before = world.economy().clone();
    assert!(
        world
            .economy_action(EconomyAction::BuyMuseumStamp, |_| Err("Falha".into()))
            .is_err()
    );
    assert_eq!(world.economy(), &before);
    let p = path();
    for _ in 0..2 {
        progress::economy_action(&p, &mut world, EconomyAction::BuyMuseumStamp).unwrap();
    }
    let reopened = progress::load_world(&p).unwrap().unwrap();
    assert_eq!(reopened.economy().museum_stamps, 2);
    assert_eq!(reopened.economy().balance, before.balance - 240);
}
