use cpu_express::{
    progress,
    simulation::{BranchKind, Journey, Route, World},
};
fn directory() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ce-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn explicit_import_preserves_corrupt_destination_and_branch_state() {
    let dir = directory();
    let source = dir.join("source.json");
    let destination = dir.join("journey.json");
    let mut w = World::default();
    let route = Route::default();
    w.restore_journey(
        Journey::restore_checkpoint(route.seed(), route.length_units(), 1000.0).unwrap(),
    );
    w.request_excursion(BranchKind::Serra).unwrap();
    progress::save(&source, &w).unwrap();
    let original = b"corrupt\x00save";
    std::fs::write(&destination, original).unwrap();
    let imported = progress::import_world(&destination, &source).unwrap();
    assert_eq!(imported.service_state(), w.service_state());
    assert_eq!(
        progress::load_world(&destination)
            .unwrap()
            .unwrap()
            .service_state(),
        w.service_state()
    );
    let backups: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.to_string_lossy().contains("before-import"))
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(std::fs::read(&backups[0]).unwrap(), original);
    progress::import_world(&destination, &source).unwrap();
    assert_eq!(std::fs::read(&backups[0]).unwrap(), original);
    assert_eq!(
        std::fs::read_dir(&dir)
            .unwrap()
            .filter(|p| p
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("before-import"))
            .count(),
        2
    );
}
#[test]
fn same_file_is_noop_and_invalid_source_leaves_destination_untouched() {
    let dir = directory();
    let source = dir.join("source.json");
    let destination = dir.join("journey.json");
    let old = progress::Checkpoint::capture(World::default().journey_snapshot());
    let original = serde_json::to_vec(&old).unwrap();
    std::fs::write(&source, &original).unwrap();
    progress::import_world(&source, &source).unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), original);
    std::fs::write(&destination, b"destination").unwrap();
    std::fs::write(&source, b"{}").unwrap();
    assert!(progress::import_world(&destination, &source).is_err());
    assert_eq!(std::fs::read(&destination).unwrap(), b"destination");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
}
#[test]
fn import_v1_converts_to_current_without_modifying_source() {
    let dir = directory();
    let source = dir.join("source.json");
    let destination = dir.join("new/sub/journey.json");
    let old = progress::Checkpoint::capture(World::default().journey_snapshot());
    let original = serde_json::to_vec(&old).unwrap();
    std::fs::write(&source, &original).unwrap();
    progress::import_world(&destination, &source).unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), original);
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
    assert_eq!(saved["version"], 2);
}
