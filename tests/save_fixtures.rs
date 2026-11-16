//! SAV-03: static rc7 documents migrate explicitly, without inventing service.
use cpu_express::{
    config::{self, Config, HudMode},
    progress,
    simulation::{StationId, Train, World, objectives::Livery},
};
use std::{fs, path::PathBuf};

struct Fixture {
    name: &'static str,
    bytes: &'static [u8],
    visits: u64,
    at_station: Option<StationId>,
    wait: f64,
    liveries: usize,
}

const FIXTURES: [Fixture; 4] = [
    Fixture {
        name: "new-profile",
        bytes: include_bytes!("fixtures/rc7/new-profile.journey.json"),
        visits: 0,
        at_station: Some(StationId::Aurora),
        wait: 0.0,
        liveries: 1,
    },
    Fixture {
        name: "mid-segment",
        bytes: include_bytes!("fixtures/rc7/mid-segment.journey.json"),
        visits: 1,
        at_station: None,
        wait: 0.0,
        liveries: 2,
    },
    Fixture {
        name: "station-stop",
        bytes: include_bytes!("fixtures/rc7/station-stop.journey.json"),
        visits: 2,
        at_station: Some(StationId::Ridge),
        wait: 5.25,
        liveries: 2,
    },
    Fixture {
        name: "rewards-unlocked",
        bytes: include_bytes!("fixtures/rc7/rewards-unlocked.journey.json"),
        visits: 49,
        at_station: None,
        wait: 0.0,
        liveries: 10,
    },
];

fn directory(tag: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ce-fixture-{}-{nonce}-{tag}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn advance(world: &mut World, seconds: usize) {
    let config = Config::default();
    let mut train = Train::new(&config);
    for _ in 0..seconds * 2 {
        world.advance_expedition(&mut train, 20.0, 0.5, &config);
    }
}

#[test]
fn rc7_fixtures_load_without_invented_service_and_without_touching_the_file() {
    for fixture in &FIXTURES {
        let dir = directory(fixture.name);
        let file = dir.join("config.journey.json");
        fs::write(&file, fixture.bytes).unwrap();
        let (world, notice) = progress::load_profile(&file).unwrap();
        let world = world.unwrap();
        assert_eq!(notice, None, "{}", fixture.name);
        assert_eq!(fs::read(&file).unwrap(), fixture.bytes, "{}", fixture.name);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1, "{}", fixture.name);

        let journey = world.journey_snapshot();
        assert_eq!(journey.stations_visited, fixture.visits, "{}", fixture.name);
        assert_eq!(journey.at_station, fixture.at_station, "{}", fixture.name);
        assert_eq!(world.station_wait(), fixture.wait, "{}", fixture.name);
        assert_eq!(journey.pending_events, 0, "{}", fixture.name);

        let service = world.service_state();
        assert!(service.passengers.is_empty() && service.contract.is_none());
        assert!(service.arrivals.is_empty() && service.collection.is_empty());
        assert_eq!(
            (
                service.passengers_boarded,
                service.passengers_delivered,
                service.cargo_delivered,
                service.contracts_completed,
                service.credits,
            ),
            (0, 0, 0, 0, 0),
            "{}",
            fixture.name
        );
        assert!(service.automatic_contracts);
        assert_eq!(service.last_processed_visit, fixture.visits);
        assert_eq!(service.owned_rewards.len(), fixture.liveries);
        for livery in Livery::ALL {
            assert_eq!(service.owns(livery), livery.available(journey));
        }
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn migrated_fixture_reopened_twice_keeps_the_same_domain_and_exact_backup() {
    for fixture in &FIXTURES {
        let dir = directory(fixture.name);
        let file = dir.join("config.journey.json");
        fs::write(&file, fixture.bytes).unwrap();
        let first = progress::load_profile(&file).unwrap().0.unwrap();
        assert_eq!(progress::save_checkpoint(&file, &first).unwrap(), None);

        let backup = file.with_extension("v1.backup.json");
        assert_eq!(fs::read(&backup).unwrap(), fixture.bytes);
        // The replaced document is also the recovery copy of this first save.
        assert_eq!(
            fs::read(progress::previous_path(&file)).unwrap(),
            fixture.bytes
        );
        let migrated = fs::read(&file).unwrap();
        let document: serde_json::Value = serde_json::from_slice(&migrated).unwrap();
        assert_eq!(document["version"], 5);
        assert_eq!(document["line"], "aurora-v1");
        assert_eq!(document["catalog_version"], 1);

        let second = progress::load_profile(&file).unwrap().0.unwrap();
        assert_eq!(progress::save_checkpoint(&file, &second).unwrap(), None);
        let third = progress::load_profile(&file).unwrap().0.unwrap();
        for reopened in [&second, &third] {
            assert_eq!(reopened.service_state(), first.service_state());
            assert_eq!(reopened.journey_snapshot(), first.journey_snapshot());
            assert_eq!(reopened.station_wait(), first.station_wait());
        }
        // Saving an unchanged domain is byte-stable and never rewrites the backup.
        assert_eq!(fs::read(&file).unwrap(), migrated);
        assert_eq!(fs::read(&backup).unwrap(), fixture.bytes);
        assert_eq!(fs::read(progress::previous_path(&file)).unwrap(), migrated);
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn migrated_stop_finishes_without_replaying_the_arrival_already_made_in_rc7() {
    let fixture = &FIXTURES[2];
    let dir = directory("stop-replay");
    let file = dir.join("config.journey.json");
    fs::write(&file, fixture.bytes).unwrap();
    let mut world = progress::load_profile(&file).unwrap().0.unwrap();
    // The remaining 5.25 s of platform time pass without a new transaction.
    advance(&mut world, 5);
    assert!(world.station_wait() > 0.0);
    assert!(world.service_state().arrivals.is_empty());
    assert_eq!(world.service_state().passengers_boarded, 0);
    assert_eq!(world.journey_snapshot().stations_visited, 2);

    // The next station is the first one served by the 0.4 rules, exactly once.
    while world.journey_snapshot().stations_visited == 2 {
        advance(&mut world, 1);
    }
    let service = world.service_state().clone();
    assert_eq!(service.arrivals.len(), 1);
    assert_eq!(service.arrivals[0].visit, 3);
    assert_eq!(service.arrivals[0].station, StationId::Harbor);
    progress::save_checkpoint(&file, &world).unwrap();
    let mut reopened = progress::load_profile(&file).unwrap().0.unwrap();
    assert_eq!(reopened.service_state(), &service);
    advance(&mut reopened, 4);
    assert_eq!(reopened.service_state(), &service);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn migrated_fixtures_keep_the_timetable_deterministic() {
    for fixture in &FIXTURES {
        let dir = directory(fixture.name);
        let file = dir.join("config.journey.json");
        fs::write(&file, fixture.bytes).unwrap();
        let mut a = progress::load_profile(&file).unwrap().0.unwrap();
        let mut b = progress::load_profile(&file).unwrap().0.unwrap();
        advance(&mut a, 700);
        // Same active time with a save and reopen in the middle of the trip.
        advance(&mut b, 333);
        progress::save_checkpoint(&file, &b).unwrap();
        let mut b = progress::load_profile(&file).unwrap().0.unwrap();
        advance(&mut b, 367);
        assert_eq!(a.service_state(), b.service_state(), "{}", fixture.name);
        assert_eq!(
            a.journey_snapshot().stations_visited,
            b.journey_snapshot().stations_visited
        );
        assert!(a.service_state().passengers_boarded > 0);
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn rc7_preferences_load_with_defaults_for_new_fields_and_stay_untouched() {
    let bytes: &[u8] = include_bytes!("fixtures/rc7/old-preferences.config.json");
    let dir = directory("preferences");
    let file = dir.join("config.json");
    fs::write(&file, bytes).unwrap();
    let (loaded, warning) = config::load(Some(&file));
    assert_eq!(warning, None);
    assert_eq!(fs::read(&file).unwrap(), bytes);

    // Choices made in rc7 survive.
    assert!(!loaded.taskbar_mode);
    assert_eq!(loaded.floating_rect, Some([40, 600, 640, 180]));
    assert_eq!(loaded.fps, 30);
    assert_eq!(loaded.hud_mode, HudMode::Detailed);
    assert_eq!(loaded.visual.livery, Livery::Midnight);

    // Fields introduced by 0.4 take their documented defaults.
    let defaults = Config::default();
    assert_eq!(loaded.sound_volume, 0.0);
    assert!(!loaded.quiet_mode && !loaded.reduce_menu_motion);
    assert_eq!(loaded.weather_location, defaults.weather_location);
    assert_eq!(loaded.visual.engine, defaults.visual.engine);
    assert!(loaded.visual.favorite_consists.is_empty());
    assert_eq!(loaded.panel_rect, None);

    // The saved livery is usable only with the journey that earned it.
    let earned = directory("preferences-journey").join("config.journey.json");
    fs::write(&earned, FIXTURES[3].bytes).unwrap();
    let world = progress::load_profile(&earned).unwrap().0.unwrap();
    assert!(world.service_state().owns(loaded.visual.livery));
    let fresh = World::default();
    assert!(!fresh.service_state().owns(loaded.visual.livery));

    // An explicit preference change keeps every rc7 key in the document.
    let mut changed = loaded.clone();
    changed.quiet_mode = true;
    config::save(&changed, &file).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    for key in original.as_object().unwrap().keys() {
        assert!(saved.get(key).is_some(), "{key}");
    }
    assert_eq!(saved["quiet_mode"], true);
    assert_eq!(saved["visual"]["livery"], "midnight");
    fs::remove_dir_all(dir).unwrap();
    fs::remove_dir_all(earned.parent().unwrap()).unwrap();
}
