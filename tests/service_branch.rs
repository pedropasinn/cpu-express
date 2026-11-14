use cpu_express::{
    config::Config,
    progress,
    simulation::{BranchKind, BranchPhase, BranchStop, Journey, Route, Train, World},
};
fn ready() -> World {
    let route = Route::default();
    let journey = Journey::restore_checkpoint(route.seed(), route.length_units(), 1000.0).unwrap();
    let mut world = World::default();
    world.restore_journey(journey);
    world
}
fn advance(world: &mut World, seconds: usize, parts: usize) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..seconds * parts {
        world.advance_expedition(&mut train, 50.0, 1.0 / parts as f64, &config);
    }
}
fn path() -> std::path::PathBuf {
    // Tests run in parallel and the clock is coarse: the counter keeps every file distinct.
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    std::env::temp_dir().join(format!(
        "ce-branch-{}-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}
#[test]
fn unlocked_request_pending_until_base_and_no_replacement() {
    assert!(
        World::default()
            .request_excursion(BranchKind::Serra)
            .is_err()
    );
    let mut w = ready();
    advance(&mut w, 10, 1);
    w.request_excursion(BranchKind::Litoral).unwrap();
    assert_eq!(w.excursion_snapshot().pending, Some(BranchKind::Litoral));
    assert!(w.excursion_snapshot().current.is_none());
    assert!(w.request_excursion(BranchKind::Serra).is_err());
    for _ in 0..600 {
        if w.excursion_snapshot().current.is_some() {
            break;
        }
        advance(&mut w, 1, 4);
    }
    assert_eq!(
        w.excursion_snapshot().current.unwrap().kind,
        BranchKind::Litoral
    );
    assert!(w.excursion_snapshot().pending.is_none());
}
#[test]
fn actual_detour_freezes_main_distance_but_moves_scene_and_active_time() {
    let mut w = ready();
    let original = w.journey_snapshot();
    w.request_excursion(BranchKind::Serra).unwrap();
    advance(&mut w, 90, 4);
    assert_eq!(
        w.excursion_snapshot().current.unwrap().phase,
        BranchPhase::Service
    );
    assert_eq!(
        w.excursion_snapshot().discoveries,
        vec![BranchStop::MiranteDaSerra]
    );
    assert_eq!(
        w.journey_snapshot().total_distance_units,
        original.total_distance_units
    );
    assert_eq!(
        w.journey_snapshot().active_seconds,
        original.active_seconds + 90.0
    );
    let scroll = w.scroll_px();
    advance(&mut w, 8, 4);
    assert_eq!(w.scroll_px(), scroll);
    advance(&mut w, 90, 4);
    assert!(w.excursion_snapshot().current.is_none());
    assert_eq!(
        w.journey_snapshot().stations_visited,
        original.stations_visited
    );
    assert_eq!(
        w.journey_snapshot().total_distance_units,
        original.total_distance_units
    );
    assert!(w.service_state().arrivals.is_empty());
    assert_eq!(w.service_state().credits, 0);
    advance(&mut w, 1, 1);
    assert_eq!(
        w.journey_snapshot().total_distance_units,
        original.total_distance_units + 24.0
    );
}
#[test]
fn restart_each_phase_partition_pause_and_repeat_discovery() {
    for seconds in [15, 93, 125] {
        let mut a = ready();
        a.request_excursion(BranchKind::Litoral).unwrap();
        advance(&mut a, seconds, 4);
        let file = path();
        progress::save(&file, &a).unwrap();
        let mut b = progress::load_world(&file).unwrap().unwrap();
        let before = b.excursion_snapshot().clone();
        let time = b.elapsed_seconds();
        b.set_paused(true);
        advance(&mut b, 30, 1);
        assert_eq!(&before, b.excursion_snapshot());
        assert_eq!(time, b.elapsed_seconds());
        b.set_paused(false);
        advance(&mut a, 188 - seconds, 4);
        advance(&mut b, 188 - seconds, 1);
        assert_eq!(a.service_state(), b.service_state());
        assert_eq!(
            a.journey_snapshot().total_distance_units,
            b.journey_snapshot().total_distance_units
        );
        b.request_excursion(BranchKind::Litoral).unwrap();
        advance(&mut b, 188, 4);
        assert_eq!(
            b.excursion_snapshot().discoveries,
            vec![BranchStop::CaisDasConchas]
        );
        assert_eq!(b.service_state().credits, 0);
    }
}
#[test]
fn invalid_phase_and_inconsistent_discovery_do_not_overwrite_save() {
    let mut w = ready();
    w.request_excursion(BranchKind::Serra).unwrap();
    let file = path();
    progress::save(&file, &w).unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    for phase in [
        serde_json::json!({"kind":"serra","phase":"outbound","phase_seconds":90}),
        serde_json::json!({"kind":"serra","phase":"service","phase_seconds":2}),
    ] {
        let mut changed = original.clone();
        changed["service"]["branch"]["current"] = phase;
        let bytes = serde_json::to_vec(&changed).unwrap();
        std::fs::write(&file, &bytes).unwrap();
        assert!(progress::load_world(&file).is_err());
        assert!(progress::save(&file, &w).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), bytes);
    }
}
#[test]
fn queued_trip_persists_and_base_service_finishes_before_departure() {
    let mut w = ready();
    w.set_station_wait(8.0);
    w.request_excursion(BranchKind::Serra).unwrap();
    let file = path();
    progress::save(&file, &w).unwrap();
    let mut loaded = progress::load_world(&file).unwrap().unwrap();
    advance(&mut loaded, 8, 4);
    assert_eq!(loaded.station_wait(), 0.0);
    advance(&mut loaded, 1, 4);
    assert_eq!(
        loaded.excursion_snapshot().current.unwrap().phase_seconds,
        1.0
    );
}
