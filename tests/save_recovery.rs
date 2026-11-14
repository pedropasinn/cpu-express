//! SAV-06/SAV-07: the profile never loses its only valid journey to one bad write.
use cpu_express::{
    config::Config,
    progress,
    simulation::{Train, World},
};
use std::{fs, path::PathBuf};

fn directory(tag: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("ce-recovery-{}-{nonce}-{tag}", std::process::id()));
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

/// Two consecutive checkpoints: `previous` holds the first, the main file the second.
fn profile(tag: &str) -> (PathBuf, PathBuf, World, World) {
    let dir = directory(tag);
    let file = dir.join("config.journey.json");
    let mut world = World::default();
    advance(&mut world, 400);
    assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
    let older = world.clone();
    advance(&mut world, 300);
    assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
    assert_ne!(older.service_state(), world.service_state());
    (dir, file, older, world)
}

fn names(dir: &PathBuf, part: &str) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap().to_string_lossy().contains(part))
        .collect()
}

#[test]
fn checkpoint_keeps_the_replaced_document_as_recovery_copy() {
    let (dir, file, older, newer) = profile("copy");
    let previous = progress::load_world(&progress::previous_path(&file))
        .unwrap()
        .unwrap();
    assert_eq!(previous.service_state(), older.service_state());
    let current = progress::load_world(&file).unwrap().unwrap();
    assert_eq!(current.service_state(), newer.service_state());
    assert!(names(&dir, ".tmp").is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn truncated_or_damaged_checkpoint_resumes_from_the_previous_valid_one() {
    let (probe_dir, probe, ..) = profile("probe");
    let valid = fs::read(&probe).unwrap();
    fs::remove_dir_all(probe_dir).unwrap();
    let mut invalid_state: serde_json::Value = serde_json::from_slice(&valid).unwrap();
    invalid_state["service"]["credits"] = serde_json::json!(999_999_999_u64);
    let damages: Vec<Vec<u8>> = vec![
        Vec::new(),
        valid[..1].to_vec(),
        valid[..valid.len() / 2].to_vec(),
        valid[..valid.len() - 1].to_vec(),
        vec![0; valid.len()],
        b"{}".to_vec(),
        serde_json::to_vec(&invalid_state).unwrap(),
    ];
    for (index, damage) in damages.iter().enumerate() {
        let (dir, file, older, _) = profile(&format!("damage-{index}"));
        let previous_bytes = fs::read(progress::previous_path(&file)).unwrap();
        fs::write(&file, damage).unwrap();
        // The plain loader keeps refusing; only the profile loader recovers.
        assert!(progress::load_world(&file).is_err());

        let (world, notice) = progress::load_profile(&file).unwrap();
        let mut world = world.unwrap();
        assert_eq!(world.service_state(), older.service_state(), "{index}");
        assert_eq!(
            world.journey_snapshot().total_distance_units,
            older.journey_snapshot().total_distance_units
        );
        let quarantined = names(&dir, "damaged-");
        assert_eq!(quarantined.len(), 1, "{index}");
        assert_eq!(&fs::read(&quarantined[0]).unwrap(), damage, "{index}");
        let notice = notice.unwrap();
        assert!(notice.contains(&*quarantined[0].file_name().unwrap().to_string_lossy()));

        // The profile is immediately consistent again and normal saves resume.
        assert_eq!(fs::read(&file).unwrap(), previous_bytes);
        assert_eq!(progress::load_profile(&file).unwrap().1, None);
        advance(&mut world, 60);
        assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
        assert_eq!(
            progress::load_world(&file)
                .unwrap()
                .unwrap()
                .service_state(),
            world.service_state()
        );
        assert_eq!(&fs::read(&quarantined[0]).unwrap(), damage);
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn newer_version_is_never_set_aside_or_downgraded() {
    let (dir, file, _, world) = profile("future");
    let mut future: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    future["version"] = serde_json::json!(4);
    future["service"]["future_field"] = serde_json::json!({"kept": true});
    let bytes = serde_json::to_vec_pretty(&future).unwrap();
    fs::write(&file, &bytes).unwrap();
    let previous = fs::read(progress::previous_path(&file)).unwrap();
    let before = fs::read_dir(&dir).unwrap().count();

    assert!(progress::load_profile(&file).is_err());
    assert!(progress::save_checkpoint(&file, &world).is_err());
    assert_eq!(fs::read(&file).unwrap(), bytes);
    assert_eq!(fs::read(progress::previous_path(&file)).unwrap(), previous);
    assert!(names(&dir, "damaged-").is_empty());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), before);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn damage_without_a_valid_recovery_copy_stays_untouched() {
    // No recovery copy at all.
    let dir = directory("alone");
    let file = dir.join("config.journey.json");
    fs::write(&file, b"{broken").unwrap();
    assert!(progress::load_profile(&file).is_err());
    assert!(progress::save_checkpoint(&file, &World::default()).is_err());
    assert_eq!(fs::read(&file).unwrap(), b"{broken");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    fs::remove_dir_all(dir).unwrap();

    // Both documents damaged: neither is moved, replaced or deleted.
    let (dir, file, ..) = profile("both");
    fs::write(&file, b"main damaged").unwrap();
    fs::write(progress::previous_path(&file), b"copy damaged").unwrap();
    assert!(progress::load_profile(&file).is_err());
    assert_eq!(fs::read(&file).unwrap(), b"main damaged");
    assert_eq!(
        fs::read(progress::previous_path(&file)).unwrap(),
        b"copy damaged"
    );
    assert!(names(&dir, "damaged-").is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_checkpoint_is_a_new_profile_and_does_not_resurrect_the_copy() {
    let (dir, file, ..) = profile("missing");
    fs::remove_file(&file).unwrap();
    let (world, notice) = progress::load_profile(&file).unwrap();
    assert!(world.is_none() && notice.is_none());
    assert!(names(&dir, "damaged-").is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn interrupted_writer_leaves_only_an_inert_temporary_file() {
    let (dir, file, _, mut world) = profile("abrupt");
    let valid = fs::read(&file).unwrap();
    // A process killed between writing the temporary file and the rename.
    let orphan = file.with_extension("99999.0.tmp");
    fs::write(&orphan, &valid[..valid.len() / 3]).unwrap();
    let (loaded, notice) = progress::load_profile(&file).unwrap();
    assert_eq!(loaded.unwrap().service_state(), world.service_state());
    assert_eq!(notice, None);
    advance(&mut world, 30);
    assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
    assert_eq!(fs::read(&orphan).unwrap(), &valid[..valid.len() / 3]);
    assert_eq!(names(&dir, ".tmp").len(), 1);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn blocked_recovery_copy_is_reported_without_losing_the_save() {
    let dir = directory("blocked-copy");
    let file = dir.join("config.journey.json");
    let mut world = World::default();
    progress::save_checkpoint(&file, &world).unwrap();
    // A directory with the copy's name makes its replacement fail.
    fs::create_dir(progress::previous_path(&file)).unwrap();
    advance(&mut world, 200);
    let warning = progress::save_checkpoint(&file, &world).unwrap();
    assert!(warning.is_some());
    assert_eq!(
        progress::load_world(&file)
            .unwrap()
            .unwrap()
            .service_state(),
        world.service_state()
    );
    assert!(names(&dir, ".tmp").is_empty());
    fs::remove_dir_all(dir).unwrap();
}

/// Another process holding the file (antivirus, sync client, permissions) makes the
/// replacement fail: the old checkpoint stays intact and a later attempt succeeds.
#[cfg(windows)]
#[test]
fn failed_replacement_keeps_the_valid_checkpoint_and_a_later_retry_succeeds() {
    use std::os::windows::fs::OpenOptionsExt;
    let (dir, file, _, mut world) = profile("sharing");
    let valid = fs::read(&file).unwrap();
    advance(&mut world, 120);
    // FILE_SHARE_READ only: reads work, replacing or deleting does not.
    let holder = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&file)
        .unwrap();
    for _ in 0..3 {
        assert!(progress::save_checkpoint(&file, &world).is_err());
        assert_eq!(fs::read(&file).unwrap(), valid);
        assert!(names(&dir, ".tmp").is_empty());
    }
    drop(holder);
    assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
    assert_eq!(
        progress::load_world(&file)
            .unwrap()
            .unwrap()
            .service_state(),
        world.service_state()
    );
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn read_only_checkpoint_is_not_replaced_or_set_aside() {
    let (dir, file, ..) = profile("readonly");
    let valid = fs::read(&file).unwrap();
    let mut permissions = fs::metadata(&file).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&file, permissions.clone()).unwrap();
    let mut world = progress::load_profile(&file).unwrap().0.unwrap();
    advance(&mut world, 90);
    assert!(progress::save_checkpoint(&file, &world).is_err());
    assert_eq!(fs::read(&file).unwrap(), valid);
    assert!(names(&dir, ".tmp").is_empty() && names(&dir, "damaged-").is_empty());
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    fs::set_permissions(&file, permissions).unwrap();
    assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
    fs::remove_dir_all(dir).unwrap();
}

/// The profile lock prevents this in the product. Even without it, racing writers may
/// lose a write but can never leave a document that fails validation.
#[test]
fn racing_writers_never_leave_an_invalid_checkpoint() {
    let dir = directory("race");
    let file = dir.join("config.journey.json");
    let mut worlds = Vec::new();
    let mut world = World::default();
    for _ in 0..4 {
        advance(&mut world, 150);
        worlds.push(world.clone());
    }
    progress::save_checkpoint(&file, &worlds[0]).unwrap();
    let handles: Vec<_> = worlds
        .into_iter()
        .map(|world| {
            let file = file.clone();
            std::thread::spawn(move || {
                for _ in 0..25 {
                    // A lost race is an ordinary reported failure.
                    let _ = progress::save_checkpoint(&file, &world);
                    if let Ok(bytes) = fs::read(&file) {
                        assert!(!bytes.is_empty());
                    }
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    assert!(progress::load_world(&file).unwrap().is_some());
    let copy = progress::previous_path(&file);
    assert!(progress::load_world(&copy).unwrap().is_some());
    assert!(names(&dir, ".tmp").is_empty());
    fs::remove_dir_all(dir).unwrap();
}

/// PERF-05 (domain part): a day of unattended active time, never draining the visual
/// event queue, keeps every persisted and in-memory collection inside its limit.
#[test]
fn a_day_of_unattended_travel_keeps_state_and_checkpoint_bounded() {
    let dir = directory("day");
    let file = dir.join("config.journey.json");
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    let mut largest = 0;
    for second in 0..86_400_u32 {
        world.advance_expedition(&mut train, 35.0, 1.0, &config);
        if second % 3_600 == 3_599 {
            let service = world.service_state();
            assert!(service.arrivals.len() <= 32 && service.passengers.len() <= 12);
            assert!(service.passenger_count() <= service.capacity().0);
            assert!(world.journey_snapshot().pending_events <= 64);
            assert!(world.smoke().len() <= 48);
            assert_eq!(progress::save_checkpoint(&file, &world).unwrap(), None);
            largest = largest.max(fs::metadata(&file).unwrap().len());
        }
    }
    assert!(world.journey_snapshot().stations_visited > 500);
    assert!(world.journey_snapshot().dropped_events > 0);
    assert!(largest < 16_384, "{largest}");
    // One main document, one recovery copy, nothing accumulating beside them.
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
    let reopened = progress::load_profile(&file).unwrap().0.unwrap();
    assert_eq!(reopened.service_state(), world.service_state());
    fs::remove_dir_all(dir).unwrap();
}
