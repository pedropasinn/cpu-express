use cpu_express::{
    config::Config,
    progress::{Checkpoint, WorldCheckpoint},
    simulation::{
        BranchStop, Journey, ServiceState, ServiceWagon, Train, World,
        objectives::{MISSIONS, MissionCounter},
        passenger_contracts::PassengerCharter,
    },
};

#[test]
fn milestones_survive_v4_roundtrip_without_replaying_transactions() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..6000 {
        world.advance_expedition(&mut train, 40.0, 1.0, &config);
    }
    let progress = MISSIONS.map(|m| m.progress(world.journey_snapshot(), world.service_state()));
    assert!(progress.iter().filter(|p| p.complete).count() >= 8);
    let saved = WorldCheckpoint {
        version: 4,
        catalog_version: 1,
        line: "aurora-v1".into(),
        journey: Checkpoint::capture(world.journey_snapshot()),
        service: world.service_state().clone(),
        economy: None,
        lines: None,
    };
    let bytes = serde_json::to_vec(&saved).unwrap();
    let restored: WorldCheckpoint = serde_json::from_slice(&bytes).unwrap();
    let mut reopened = World::default();
    reopened.restore_journey(restored.journey.restore().unwrap());
    reopened.restore_service(restored.service).unwrap();
    let state = reopened.service_state().clone();
    for _ in 0..3 {
        assert_eq!(
            progress,
            MISSIONS.map(|m| m.progress(reopened.journey_snapshot(), reopened.service_state()))
        );
        reopened.advance_expedition(&mut train, 100.0, 0.0, &config);
        assert_eq!(state, *reopened.service_state());
    }
}

#[test]
fn every_mission_has_unique_id_and_correct_durable_threshold() {
    let snapshot = Journey::default().snapshot();
    for (index, mission) in MISSIONS.iter().enumerate() {
        assert!(!MISSIONS[..index].iter().any(|other| other.id == mission.id));
        for value in [mission.goal - 1, mission.goal, mission.goal + 1] {
            let mut journey = snapshot;
            let mut service = ServiceState::migrated(snapshot);
            match mission.counter {
                MissionCounter::Passengers => service.passengers_delivered = value,
                MissionCounter::Cargo => service.cargo_delivered = value,
                MissionCounter::Contracts => service.contracts_completed = value,
                MissionCounter::Stations => {
                    journey.discovered_stations = [false; 4];
                    for entry in journey.discovered_stations.iter_mut().take(value as usize) {
                        *entry = true;
                    }
                }
                MissionCounter::Branches => {
                    service.branch.discoveries =
                        [BranchStop::MiranteDaSerra, BranchStop::CaisDasConchas]
                            .into_iter()
                            .take(value as usize)
                            .collect()
                }
                MissionCounter::ServicePostcards => {
                    service.collection = [
                        cpu_express::simulation::ServiceReward::PassengerPostcard,
                        cpu_express::simulation::ServiceReward::CargoPostcard,
                    ]
                    .into_iter()
                    .take(value as usize)
                    .collect()
                }
                MissionCounter::Liveries => {
                    service.owned_rewards = cpu_express::simulation::objectives::Livery::ALL
                        .into_iter()
                        .take(value as usize)
                        .collect()
                }
            }
            let p = mission.progress(journey, &service);
            assert_eq!(p.complete, value >= mission.goal, "{}", mission.id);
            assert_eq!(p.current, value.min(mission.goal));
        }
    }
}

#[test]
fn discovery_is_distinct_and_collection_does_not_count_equipped_stock() {
    let mut journey = Journey::default().snapshot();
    journey.stations_visited = 50;
    let mut service = ServiceState::migrated(journey);
    service.composition = [ServiceWagon::Passenger; 3];
    let mission = MISSIONS
        .iter()
        .find(|m| m.counter == MissionCounter::Stations)
        .unwrap();
    assert_eq!(mission.progress(journey, &service).current, 1);
    let postcards = MISSIONS
        .iter()
        .find(|m| m.counter == MissionCounter::ServicePostcards)
        .unwrap();
    assert_eq!(postcards.progress(journey, &service).current, 0);
}

#[test]
fn charter_preview_accounts_for_occupied_seats_and_specialty_without_mutating_service() {
    let mut service = ServiceState::migrated(Journey::default().snapshot());
    service.composition = [
        ServiceWagon::Dining,
        ServiceWagon::Freight,
        ServiceWagon::Freight,
    ];
    assert!(PassengerCharter::DiningTrip.preview(&service).fits());
    assert_eq!(
        PassengerCharter::SchoolVisit
            .preview(&service)
            .missing_seats,
        4
    );
    assert_eq!(
        PassengerCharter::ScenicParty
            .preview(&service)
            .missing_wagon,
        Some(ServiceWagon::Observation)
    );
    service
        .passengers
        .push(cpu_express::simulation::PassengerGroup {
            origin: cpu_express::simulation::StationId::Aurora,
            destination: cpu_express::simulation::StationId::Pines,
            count: 5,
        });
    let before = service.clone();
    let preview = PassengerCharter::DiningTrip.preview(&service);
    assert_eq!(preview.missing_seats, 3);
    assert!(!preview.fits());
    for charter in PassengerCharter::ALL {
        let _ = charter.preview(&service);
    }
    assert_eq!(before, service);
}
