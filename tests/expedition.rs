use cpu_express::{
    config::Config,
    progress,
    simulation::{Journey, Train, World},
};

fn run(cpu: f32, hz: usize, seconds: usize) -> World {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..seconds * hz {
        world.advance_expedition(&mut train, cpu, 1.0 / hz as f64, &config);
    }
    world
}

#[test]
fn timetable_and_rewards_do_not_depend_on_cpu_or_fps() {
    let reference = run(0.0, 30, 1300).journey_snapshot();
    for (cpu, hz) in [(50.0, 60), (100.0, 144), (0.0, 10)] {
        let sample = run(cpu, hz, 1300).journey_snapshot();
        assert_eq!(sample.stations_visited, reference.stations_visited);
        assert_eq!(sample.achievements, reference.achievements);
        assert!((sample.total_distance_units - reference.total_distance_units).abs() < 1e-5);
    }
}

#[test]
fn stop_freezes_wheels_and_pause_freezes_service() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    while world.station_wait() == 0.0 {
        world.advance_expedition(&mut train, 100.0, 0.1, &config);
    }
    let phase = train.wheel_phase();
    let scroll = world.scroll_px();
    world.advance_expedition(&mut train, 100.0, 1.0, &config);
    assert_eq!(phase, train.wheel_phase());
    assert_eq!(scroll, world.scroll_px());
    assert_eq!(train.speed_kmh(), 0.0);
    let waiting = world.station_wait();
    world.set_paused(true);
    world.advance_expedition(&mut train, 100.0, 1.0, &config);
    assert_eq!(waiting, world.station_wait());
}

#[test]
fn checkpoint_restores_route_rewards_and_stop_without_replaying_events() {
    let mut world = run(50.0, 30, 600);
    world.set_station_wait(4.25);
    let path = std::env::temp_dir().join(format!("ce-expedition-{}.json", std::process::id()));
    progress::save(&path, &world).unwrap();
    let (restored, wait) = progress::load(&path).unwrap().unwrap();
    let before = world.journey_snapshot();
    let after = restored.snapshot();
    assert_eq!(wait, 4.25);
    assert_eq!(before.stations_visited, after.stations_visited);
    assert_eq!(before.achievements, after.achievements);
    assert!((before.segment_progress - after.segment_progress).abs() < 1e-8);
    assert_eq!(after.pending_events, 0);
    std::fs::write(&path, b"{broken").unwrap();
    assert!(progress::save(&path, &world).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_or_future_checkpoints_are_rejected() {
    for value in [f64::NAN, f64::INFINITY, -1.0, 1e15] {
        assert!(Journey::restore_checkpoint(1, value, 10.0).is_err());
    }
    let mut checkpoint = progress::Checkpoint::capture(World::default().journey_snapshot());
    checkpoint.version = 99;
    assert!(checkpoint.restore().is_err());
}

#[cfg(windows)]
#[test]
fn only_one_writer_can_open_a_profile_and_release_allows_reopening() {
    let path = std::env::temp_dir().join(format!("ce-writer-{}.json", std::process::id()));
    let first = progress::lock_profile(&path).unwrap();
    assert!(progress::lock_profile(&path).is_err());
    drop(first);
    let second = progress::lock_profile(&path).unwrap();
    drop(second);
    std::fs::remove_file(path.with_extension("profile.lock")).unwrap();
}

#[test]
fn orphan_temporary_is_preserved_and_repeated_checkpoints_replace_valid_save() {
    let path = std::env::temp_dir().join(format!("ce-collision-{}.json", std::process::id()));
    let orphan = path.with_extension(format!("{}.0.tmp", std::process::id()));
    std::fs::write(&orphan, b"orphan").unwrap();
    let mut world = World::default();
    progress::save(&path, &world).unwrap();
    world.set_station_wait(3.0);
    progress::save(&path, &world).unwrap();
    assert_eq!(progress::load(&path).unwrap().unwrap().1, 3.0);
    assert_eq!(std::fs::read(&orphan).unwrap(), b"orphan");
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(orphan).unwrap();
}
