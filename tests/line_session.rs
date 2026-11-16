//! E3 integration seam: no profile, network, renderer or currency mutation.
use cpu_express::{
    config::Config,
    simulation::{
        AdvanceOutcome, ContractStatus, ServiceState, Train, World,
        catalog::{Catalog, ContentKind, Layout, LineDef},
        itinerary::{DEFAULT_UNITS_PER_KM, Itinerary},
        line_session::{CatalogStationId, LineSession},
        world::LineProgress,
    },
};

fn line() -> &'static LineDef {
    Catalog::builtin()
        .as_ref()
        .unwrap()
        .line("curitiba-paranagua")
        .unwrap()
}
fn inspection() -> LineSession {
    LineSession::for_inspection(
        line(),
        Itinerary::from_line(line(), DEFAULT_UNITS_PER_KM).unwrap(),
        42,
    )
    .unwrap()
}
fn to_visit(session: &mut LineSession, visit: u64) {
    for _ in 0..20_000 {
        if session.position().stations_visited >= visit {
            return;
        }
        session.advance(1.0);
    }
    panic!("arrival not reached");
}

#[test]
fn fourteen_stations_both_directions_deliver_and_restore_without_replay() {
    assert_eq!(LineSession::SERVICE_KIND, ContentKind::Fictional);
    assert!(LineSession::playable(line(), 42).is_ok());
    let mut session = inspection();
    assert!(session.can_leave_line());
    for visit in 1..=52 {
        to_visit(&mut session, visit);
        let station = &session.service().arrivals.last().unwrap().station;
        let expected = session.itinerary().station_at_visit(visit);
        assert_eq!(station.line, line().id);
        assert_eq!(station.station, line().stations[expected].id);
        let progress =
            serde_json::from_str(&serde_json::to_string(session.progress()).unwrap()).unwrap();
        let service =
            serde_json::from_str(&serde_json::to_string(session.service()).unwrap()).unwrap();
        let mut restored = inspection();
        restored.restore(progress, service).unwrap();
        assert_eq!(session.service(), restored.service());
        let (a, b) = (session.progress(), restored.progress());
        assert_eq!(a.seed, b.seed);
        assert_eq!(a.stations_visited, b.stations_visited);
        assert_eq!(a.completed_runs, b.completed_runs);
        assert_eq!(a.discovered_stations, b.discovered_stations);
        // serde_json without float_roundtrip may differ in the last binary digit.
        assert!((a.distance - b.distance).abs() < 1e-9);
        assert!((a.active_seconds - b.active_seconds).abs() < 1e-9);
        assert!((a.station_wait - b.station_wait).abs() < 1e-9);
        assert!(!restored.can_leave_line());
        let credits = restored.service().credits;
        restored.advance(0.25);
        assert_eq!(restored.service().credits, credits);
        assert!(
            session
                .service()
                .passengers
                .iter()
                .all(|p| p.origin != p.destination)
        );
        if visit == 13 {
            assert!(!session.position().forward);
        }
        if visit == 26 {
            assert!(session.position().forward);
        }
    }
    assert_eq!(session.progress().discovered_stations.len(), 14);
    assert!(session.service().passengers_delivered > 100);
    assert!(session.service().contracts_completed > 0);
}

#[test]
fn generic_clock_and_service_transactions_match_aurora() {
    let line = Catalog::builtin()
        .as_ref()
        .unwrap()
        .line("aurora-v1")
        .unwrap();
    for seed in [0, 7, 42, u64::MAX] {
        let mut session = LineSession::playable(line, seed).unwrap();
        let mut world = World::with_seed(seed);
        let config = Config::default();
        let mut train = Train::new(&config);
        for _ in 0..1800 {
            session.advance(0.5);
            world.advance_expedition(&mut train, 0.0, 0.5, &config);
            let j = world.journey_snapshot();
            assert!((session.progress().distance - j.total_distance_units).abs() < 1e-6);
            assert_eq!(session.progress().stations_visited, j.stations_visited);
            assert_eq!(
                session.service().passengers_delivered,
                world.service_state().passengers_delivered
            );
            assert_eq!(
                session.service().cargo_delivered,
                world.service_state().cargo_delivered
            );
            assert_eq!(session.service().credits, world.service_state().credits);
        }
    }
}

#[test]
fn generic_domain_handles_twenty_nine_stops_and_rejects_bad_restoration_atomically() {
    let mut synthetic = line().clone();
    synthetic.id = "fixture-fictional-29".into();
    synthetic.kind = ContentKind::Fictional;
    synthetic.layout = Layout::Shuttle;
    synthetic.stations = (0..29)
        .map(|i| {
            let mut station = line().stations[0].clone();
            station.id = format!("s{i}");
            station.architecture = None;
            station
        })
        .collect();
    synthetic.segments = (0..28)
        .map(|i| {
            let mut segment = line().segments[0].clone();
            segment.id = format!("leg{i}");
            segment.from = format!("s{i}");
            segment.to = format!("s{}", i + 1);
            segment.base_units = Some(48);
            segment
        })
        .collect();
    let mut session = LineSession::for_inspection(
        &synthetic,
        Itinerary::from_line(&synthetic, 480.0).unwrap(),
        42,
    )
    .unwrap();
    to_visit(&mut session, 56);
    assert_eq!(session.progress().discovered_stations.len(), 29);
    session
        .restore(session.progress().clone(), session.service().clone())
        .unwrap();
    let baseline = session.progress().clone();
    let service = session.service().clone();
    for which in 0..6 {
        let mut bad = baseline.clone();
        match which {
            0 => bad.distance = f64::NAN,
            1 => bad.discovered_stations.clear(),
            2 => bad.stations_visited += 1,
            3 => bad.station_wait = 9.0,
            4 => bad.completed_runs += 1,
            _ => bad.seed += 1,
        }
        assert!(session.restore(bad, service.clone()).is_err());
        assert_eq!(session.progress(), &baseline);
        assert_eq!(session.service(), &service);
    }
    let mut bad = service.clone();
    bad.arrivals.last_mut().unwrap().station.line = "other-line".into();
    assert!(session.restore(baseline.clone(), bad).is_err());
    let mut bad = service.clone();
    bad.passengers[0].destination.line = "other-line".into();
    assert!(session.restore(baseline, bad).is_err());
    assert_eq!(session.service(), &service);
}

#[test]
fn pause_invalid_steps_and_manual_return_keep_the_same_active_clock() {
    let mut session = inspection();
    session.set_automatic_contracts(false);
    let original = session.progress().clone();
    for dt in [0.0, -1.0, f64::NAN, f64::INFINITY, 60.0] {
        assert!(!matches!(
            session.advance(dt),
            AdvanceOutcome::Advanced { .. }
        ));
        assert_eq!(session.progress(), &original);
    }
    session.set_paused(true);
    assert_eq!(session.advance(1.0), AdvanceOutcome::Paused);
    assert_eq!(session.progress(), &original);
    session.set_paused(false);
    to_visit(&mut session, 1);
    assert_eq!(
        session.service().contract.as_ref().unwrap().status,
        ContractStatus::Offered
    );
    session.accept_contract().unwrap();
    assert_eq!(
        session.service().contract.as_ref().unwrap().status,
        ContractStatus::Loaded
    );
    session.advance(1.0);
    assert_eq!(session.progress().distance, 7680.0);
    assert_eq!(session.progress().station_wait, 3.0);
    session.cancel_contract().unwrap();
    assert_eq!(session.service().cargo_count(), 0);
    assert_eq!(session.service().contracts_completed, 0);
    assert_eq!(session.service().credits, 0);
    assert!(!session.can_leave_line()); // Passengers remain aboard.
}

#[test]
fn fixture_uses_stable_qualified_ids_and_v5_line_progress() {
    #[derive(serde::Deserialize)]
    struct Fixture {
        progress: LineProgress,
        service: ServiceState<CatalogStationId>,
    }
    let saved: Fixture =
        serde_json::from_str(include_str!("fixtures/e3-line-inspection.json")).unwrap();
    let mut session = inspection();
    session.restore(saved.progress, saved.service).unwrap();
    let mut expected = inspection();
    to_visit(&mut expected, 1);
    assert_eq!(session.progress(), expected.progress());
    assert_eq!(session.service(), expected.service());
    for _ in 0..5000 {
        session.advance(0.5);
        expected.advance(0.5);
    }
    assert_eq!(session.progress(), expected.progress());
    assert_eq!(session.service(), expected.service());
}

#[test]
fn departures_require_empty_services_and_the_right_platform() {
    let mut session = inspection();
    let mut service = session.service().clone();
    service.composition = [cpu_express::simulation::ServiceWagon::Freight; 3];
    service.automatic_contracts = false;
    session
        .restore(session.progress().clone(), service)
        .unwrap();
    to_visit(&mut session, 1);
    assert!(!session.can_leave_line()); // An intermediate shuttle platform.
    to_visit(&mut session, 13);
    assert!(session.can_leave_line());
    // Accepting an offer from another platform must also block departure.
    session.accept_contract().unwrap();
    assert_eq!(
        session.service().contract.as_ref().unwrap().status,
        ContractStatus::Accepted
    );
    assert!(!session.can_leave_line());
    session.cancel_contract().unwrap();
    assert!(session.can_leave_line());

    let aurora = Catalog::builtin()
        .as_ref()
        .unwrap()
        .line("aurora-v1")
        .unwrap();
    let mut circuit = LineSession::playable(aurora, 42).unwrap();
    let mut service = circuit.service().clone();
    service.composition = [cpu_express::simulation::ServiceWagon::Freight; 3];
    service.automatic_contracts = false;
    circuit
        .restore(circuit.progress().clone(), service)
        .unwrap();
    to_visit(&mut circuit, 1);
    assert!(circuit.can_leave_line());
    while circuit.progress().station_wait > 0.0 {
        circuit.advance(0.1);
    }
    circuit.advance(0.1);
    assert!(!circuit.can_leave_line());
}

#[test]
fn frame_partition_does_not_skip_stops_or_change_services() {
    let mut whole = inspection();
    let mut frames = inspection();
    for _ in 0..5000 {
        whole.advance(1.0);
    }
    for _ in 0..300_000 {
        frames.advance(1.0 / 60.0);
    }
    assert_eq!(whole.service(), frames.service());
    assert_eq!(
        whole.progress().discovered_stations,
        frames.progress().discovered_stations
    );
    assert!((whole.progress().distance - frames.progress().distance).abs() < 1e-5);
    assert!((whole.progress().active_seconds - frames.progress().active_seconds).abs() < 1e-5);
    frames
        .restore(frames.progress().clone(), frames.service().clone())
        .unwrap();
}
