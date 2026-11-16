use cpu_express::{
    config::Config,
    progress,
    simulation::{Train, World, economy::EconomyAction, line_session::LineSession},
};
use std::{fs, path::PathBuf};

fn path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let folder = std::env::temp_dir().join(format!(
        "cpu-e3-playable-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&folder).unwrap();
    folder.join("config.journey.json")
}
fn advance(world: &mut World, seconds: usize) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..seconds {
        world.advance_expedition(&mut train, 45.0, 1.0, &config);
    }
}
fn until(world: &mut World, predicate: impl Fn(&World) -> bool) {
    for _ in 0..20_000 {
        if predicate(world) {
            return;
        }
        advance(world, 1);
    }
    panic!("Condição não atingida: {:?}", world.travel_view());
}
fn save(world: &World, path: &std::path::Path) -> Result<(), String> {
    progress::save_checkpoint(path, world).map(|_| ())
}

#[test]
fn full_real_round_trip_shared_income_discovery_and_unique_missions() {
    let mut world = World::with_seed(42);
    world.switch_line("curitiba-paranagua", |_| Ok(())).unwrap();
    let mut expected = LineSession::playable(
        cpu_express::simulation::catalog::Catalog::builtin()
            .as_ref()
            .unwrap()
            .line("curitiba-paranagua")
            .unwrap(),
        42,
    )
    .unwrap();
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut visited = Vec::new();
    let mut previous = 0;
    for _ in 0..6000 {
        world.advance_expedition(&mut train, 0.0, 1.0, &config);
        expected.advance(1.0);
        let s = world.active_session().unwrap();
        assert_eq!(s.progress(), expected.progress());
        if s.progress().stations_visited != previous {
            previous = s.progress().stations_visited;
            visited.push(s.position().at_station.unwrap());
        }
        if previous == 26 {
            break;
        }
    }
    assert_eq!(visited, (1..14).chain((0..13).rev()).collect::<Vec<_>>());
    assert_eq!(
        world
            .active_session()
            .unwrap()
            .progress()
            .discovered_stations
            .len(),
        14
    );
    assert_eq!(world.active_session().unwrap().progress().completed_runs, 2);
    assert_eq!(
        world.service_view().passengers_delivered,
        expected.service().passengers_delivered
    );
    assert!(world.service_view().passengers_delivered > 0);
    assert!(world.service_view().contracts_completed > 0);
    assert!(world.economy().balance > 30);
    assert_eq!(world.journey_snapshot().stations_visited, 0);
    assert_eq!(world.economic_snapshot().stations_visited, 26);
    for id in [
        "line.curitiba-paranagua.stations",
        "line.curitiba-paranagua.ascent",
    ] {
        let before = world.economy().balance;
        let reward = world
            .missions()
            .into_iter()
            .find(|m| m.id == id)
            .unwrap()
            .reward;
        assert!(
            world
                .economy_action(EconomyAction::ClaimMission(id.into()), |_| Err(
                    "disco".into()
                ))
                .is_err()
        );
        assert_eq!(world.economy().balance, before);
        world
            .economy_action(EconomyAction::ClaimMission(id.into()), |_| Ok(()))
            .unwrap();
        assert_eq!(world.economy().balance, before + reward);
        assert!(
            world
                .economy_action(EconomyAction::ClaimMission(id.into()), |_| Ok(()))
                .is_err()
        );
    }
}

#[test]
fn close_boarding_switch_save_reopen_return_preserves_lines_and_global_orders() {
    let path = path("roundtrip");
    let mut world = World::with_seed(42);
    advance(&mut world, 750);
    world.set_boarding_closed(true, |w| save(w, &path)).unwrap();
    let boarded = world.service_view().passengers_boarded;
    until(&mut world, World::can_leave_line);
    assert_eq!(world.service_view().passengers_boarded, boarded);
    let aurora = world.line_progress()["aurora-v1"].clone();
    let aurora_service = world.service_state().clone();
    let economy = world.economy().clone();
    let visits = world.economic_snapshot().stations_visited;
    let time = world.economic_snapshot().active_seconds;
    assert!(
        world
            .switch_line("curitiba-paranagua", |_| Err("falha".into()))
            .is_err()
    );
    assert_eq!(world.active_line_id(), "aurora-v1");
    assert_eq!(world.economy(), &economy);
    assert_eq!(world.line_progress().len(), 1);
    world
        .switch_line("curitiba-paranagua", |w| save(w, &path))
        .unwrap();
    world = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(world.active_line_id(), "curitiba-paranagua");
    assert_eq!(world.economy(), &economy);
    assert_eq!(world.economic_snapshot().stations_visited, visits);
    assert_eq!(world.economic_snapshot().active_seconds, time);
    assert_eq!(world.service_state(), &aurora_service);
    assert_eq!(world.line_progress()["aurora-v1"], aurora);
    world
        .set_boarding_closed(false, |w| save(w, &path))
        .unwrap();
    advance(&mut world, 900);
    save(&world, &path).unwrap();
    let service = world.service_view().clone();
    let economy = world.economy().clone();
    world = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(world.service_view(), &service);
    assert_eq!(world.economy(), &economy);
    assert!(!world.can_leave_line());
    assert!(world.switch_line("aurora-v1", |_| Ok(())).is_err());
    world.set_boarding_closed(true, |w| save(w, &path)).unwrap();
    until(&mut world, World::can_leave_line);
    let real = world.line_progress()["curitiba-paranagua"].clone();
    let totals = world.economic_snapshot();
    let economy = world.economy().clone();
    world.switch_line("aurora-v1", |w| save(w, &path)).unwrap();
    world = progress::load_profile(&path).unwrap().0.unwrap();
    assert_eq!(world.active_line_id(), "aurora-v1");
    assert_eq!(world.line_progress()["aurora-v1"], aurora);
    assert_eq!(world.line_progress()["curitiba-paranagua"], real);
    assert_eq!(world.economy(), &economy);
    assert_eq!(
        world.economic_snapshot().stations_visited,
        totals.stations_visited
    );
    assert_eq!(
        world.economic_snapshot().active_seconds,
        totals.active_seconds
    );
    world
        .switch_line("curitiba-paranagua", |w| save(w, &path))
        .unwrap();
    assert_eq!(world.line_progress()["curitiba-paranagua"], real);
}

#[test]
fn real_arrival_failure_rolls_back_wallet_service_and_train_and_pause_freezes_both_lines() {
    let mut world = World::with_seed(42);
    world.switch_line("curitiba-paranagua", |_| Ok(())).unwrap();
    advance(&mut world, 319);
    let before = world.line_progress();
    let service = world.service_view().clone();
    let economy = world.economy().clone();
    let config = Config::default();
    let mut train = Train::new(&config);
    let bad = path("failure");
    fs::write(&bad, b"arquivo protegido").unwrap();
    assert!(
        progress::advance_checkpoint(&bad, &mut world, &mut train, 50.0, 1.0, &config).is_err()
    );
    assert_eq!(world.line_progress(), before);
    assert_eq!(world.service_view(), &service);
    assert_eq!(world.economy(), &economy);
    world.set_paused(true);
    advance(&mut world, 100);
    assert_eq!(world.line_progress(), before);
    world.set_paused(false);
    world.advance_expedition(&mut train, 20.0, 30.0, &config);
    assert_eq!(world.line_progress(), before);
}

#[test]
fn v5_migration_has_exact_backup_and_v6_rejects_unknown_identity_without_overwrite() {
    let path = path("migration");
    let fixture = include_bytes!("fixtures/economy/v5-new.json");
    fs::write(&path, fixture).unwrap();
    let mut world = progress::load_world(&path).unwrap().unwrap();
    let old = world.economy().clone();
    world
        .switch_line("curitiba-paranagua", |w| save(w, &path))
        .unwrap();
    assert_eq!(
        fs::read(path.with_extension("v5.backup.json")).unwrap(),
        fixture
    );
    assert_eq!(world.economy(), &old);
    let bytes = fs::read(&path).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["version"], 6);
    json["lines"]["curitiba-paranagua"]["discovered_stations"] = serde_json::json!(["inventada"]);
    let corrupt = serde_json::to_vec(&json).unwrap();
    fs::write(&path, &corrupt).unwrap();
    assert!(progress::load_world(&path).is_err());
    assert!(save(&world, &path).is_err());
    assert_eq!(fs::read(&path).unwrap(), corrupt);
}

#[test]
fn static_v6_fixture_reopens_in_motion_without_replaying_income() {
    let path = path("v6-fixture");
    fs::write(&path, include_bytes!("fixtures/economy/v6-real-line.json")).unwrap();
    let world = progress::load_world(&path).unwrap().unwrap();
    assert_eq!(world.active_line_id(), "curitiba-paranagua");
    assert_eq!(world.travel_view().next_name, "Cadeado");
    assert!(world.service_view().passenger_count() > 0);
    let economy = world.economy().clone();
    let service = world.service_view().clone();
    let lines = world.line_progress();
    save(&world, &path).unwrap();
    let reopened = progress::load_profile(&path).unwrap().0.unwrap();
    assert_eq!(reopened.economy(), &economy);
    assert_eq!(reopened.service_view(), &service);
    assert_eq!(reopened.line_progress(), lines);
}

#[test]
fn real_line_machine_bonus_changes_only_income_and_holding_terminal_freezes_clock() {
    let mut base = World::with_seed(42);
    base.switch_line("curitiba-paranagua", |_| Ok(())).unwrap();
    let mut bonus = base.clone();
    bonus.set_machine_bonus(1.3);
    advance(&mut base, 2300);
    advance(&mut bonus, 2300);
    assert_eq!(base.line_progress(), bonus.line_progress());
    assert_eq!(base.service_view(), bonus.service_view());
    assert!(bonus.economy().balance > base.economy().balance);
    base.set_boarding_closed(true, |_| Ok(())).unwrap();
    until(&mut base, World::can_leave_line);
    let lines = base.line_progress();
    let income = base.economy().clone();
    advance(&mut base, 100);
    assert_eq!(base.line_progress(), lines);
    assert_eq!(base.economy(), &income);
    base.set_boarding_closed(false, |_| Ok(())).unwrap();
    advance(&mut base, 10);
    assert_ne!(base.line_progress(), lines);
}
