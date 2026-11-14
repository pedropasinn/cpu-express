use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use cpu_express::config::{self, Config};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock")
            .as_nanos();
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "cpu-express-config-test-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn names_in(path: &Path) -> Vec<String> {
    let mut names = fs::read_dir(path)
        .expect("read test directory")
        .map(|entry| {
            entry
                .expect("read test entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

#[test]
fn missing_or_unspecified_config_uses_defaults_without_writing() {
    let directory = TestDirectory::new();
    let missing = directory.path().join("missing.json");
    assert_eq!(config::load(None), (Config::default(), None));
    assert_eq!(config::load(Some(&missing)), (Config::default(), None));
    assert!(!missing.exists());
    assert!(names_in(directory.path()).is_empty());
}

#[test]
fn round_trip_creates_parents_and_replaces_the_previous_complete_file() {
    let directory = TestDirectory::new();
    let parent = directory.path().join("preferences");
    let path = parent.join("config.json");
    let first = Config::default();
    config::save(&first, &path).expect("save initial preferences");
    assert_eq!(config::load(Some(&path)), (first, None));

    let changed = Config {
        min_speed_kmh: 20.0,
        max_speed_kmh: 88.0,
        show_metrics: false,
        always_on_top: false,
        fps: 45,
        ..Config::default()
    };
    config::save(&changed, &path).expect("replace preferences");
    assert_eq!(config::load(Some(&path)), (changed, None));
    assert_eq!(names_in(&parent), ["config.json"]);
}

#[test]
fn partial_config_uses_defaults_for_missing_fields() {
    let directory = TestDirectory::new();
    let path = directory.path().join("partial.json");
    let original = br#"{"fps":45,"show_metrics":false}"#;
    fs::write(&path, original).expect("write partial fixture");
    let (loaded, warning) = config::load(Some(&path));
    assert_eq!(
        loaded,
        Config {
            fps: 45,
            show_metrics: false,
            ..Config::default()
        }
    );
    assert_eq!(warning, None);
    assert_eq!(fs::read(&path).expect("read fixture"), original);
}

#[test]
fn malformed_config_is_preserved_and_reports_a_warning() {
    let directory = TestDirectory::new();
    let path = directory.path().join("broken.json");
    for original in [
        b"not JSON".as_slice(),
        b"{\"fps\":30,".as_slice(),
        &[0xff, 0xfe],
    ] {
        fs::write(&path, original).expect("write corrupt fixture");
        let (loaded, warning) = config::load(Some(&path));
        assert_eq!(loaded, Config::default());
        assert!(warning.is_some());
        assert_eq!(fs::read(&path).expect("read preserved fixture"), original);
    }
    assert_eq!(names_in(directory.path()), ["broken.json"]);
}

#[test]
fn numeric_values_are_normalized_without_implicitly_rewriting_the_file() {
    let directory = TestDirectory::new();
    let path = directory.path().join("limits.json");
    let original = br#"{"min_speed_kmh":-50,"max_speed_kmh":-30,"smoothing_k":0,"fps":5000,"metrics_interval_ms":1}"#;
    fs::write(&path, original).expect("write bounds fixture");
    let (loaded, warning) = config::load(Some(&path));
    assert_eq!(loaded.min_speed_kmh, 0.0);
    assert_eq!(loaded.max_speed_kmh, 0.0);
    assert_eq!(loaded.smoothing_k, 0.05);
    assert_eq!(loaded.fps, 60);
    assert_eq!(loaded.metrics_interval_ms, 250);
    assert!(warning.is_some());
    assert_eq!(fs::read(&path).expect("read unchanged fixture"), original);
}

#[test]
fn save_normalizes_programmatic_invalid_floats_before_json_encoding() {
    let directory = TestDirectory::new();
    let path = directory.path().join("config.json");
    let invalid = Config {
        min_speed_kmh: f64::NAN,
        max_speed_kmh: f64::INFINITY,
        smoothing_k: f64::NEG_INFINITY,
        ..Config::default()
    };
    config::save(&invalid, &path).expect("save repaired config");
    assert_eq!(config::load(Some(&path)), (Config::default(), None));
}

#[test]
fn non_directory_parent_fails_gracefully_without_modifying_existing_data() {
    let directory = TestDirectory::new();
    let parent_file = directory.path().join("a-file");
    fs::write(&parent_file, "keep this content").expect("write existing data");
    let target = parent_file.join("config.json");
    assert!(config::save(&Config::default(), &target).is_err());
    assert_eq!(
        fs::read_to_string(&parent_file).expect("read existing data"),
        "keep this content"
    );
    assert_eq!(names_in(directory.path()), ["a-file"]);
}

#[test]
fn failed_final_rename_cleans_up_only_its_own_temporary_file() {
    let directory = TestDirectory::new();
    let destination = directory.path().join("existing-directory");
    fs::create_dir(&destination).expect("create invalid file destination");
    fs::write(destination.join("keep.txt"), "preserve me").expect("write existing data");
    assert!(config::save(&Config::default(), &destination).is_err());
    assert_eq!(names_in(directory.path()), ["existing-directory"]);
    assert_eq!(
        fs::read_to_string(destination.join("keep.txt")).expect("read preserved data"),
        "preserve me"
    );
}

#[test]
fn window_layout_defaults_roundtrips_and_clamps_untrusted_geometry() {
    let legacy: Config = serde_json::from_str("{}").unwrap();
    assert_eq!(
        (
            legacy.taskbar_width,
            legacy.taskbar_height,
            legacy.floating_rect
        ),
        (0, 0, None)
    );
    let layout = Config {
        taskbar_offset: 900,
        taskbar_width: 480,
        taskbar_height: 40,
        floating_rect: Some([-1200, 80, 900, 260]),
        ..Config::default()
    };
    let encoded = serde_json::to_string(&layout).unwrap();
    assert_eq!(
        serde_json::from_str::<Config>(&encoded)
            .unwrap()
            .normalized(),
        layout
    );
    let invalid = Config {
        taskbar_offset: -10,
        taskbar_width: i32::MAX,
        taskbar_height: -4,
        floating_rect: Some([i32::MIN, i32::MAX, -1, 99999]),
        ..Config::default()
    }
    .normalized();
    assert_eq!(
        (
            invalid.taskbar_offset,
            invalid.taskbar_width,
            invalid.taskbar_height
        ),
        (0, 32768, 24)
    );
    assert_eq!(invalid.floating_rect, Some([-32768, 32768, 320, 1200]));
}
