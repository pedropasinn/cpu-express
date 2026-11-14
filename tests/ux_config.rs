use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use cpu_express::config::{self, Config, HudMode};
use serde_json::json;

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
            "cpu-express-ux-config-test-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }

    fn file(&self) -> PathBuf {
        self.0.join("config.json")
    }

    fn entries(&self) -> usize {
        fs::read_dir(&self.0).expect("read test directory").count()
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("read config")).expect("valid saved JSON")
}

#[test]
fn visual_choices_round_trip_without_erasing_future_nested_preferences() {
    use cpu_express::rendering::{CoachStyle, RenderQuality, SceneTheme, Weather};
    let directory = TestDirectory::new();
    let path = directory.file();
    fs::write(
        &path,
        br#"{"visual":{"future_effect":{"amount":7},"scene":{"future_sky":"blue"}}}"#,
    )
    .unwrap();
    let (mut loaded, warning) = config::load(Some(&path));
    assert_eq!(warning, None);
    loaded.visual.theme = SceneTheme::Journey;
    loaded.visual.quality = RenderQuality::Low;
    loaded.visual.scene.weather = Weather::Rain;
    loaded.visual.scene.day_cycle = false;
    loaded.visual.coaches = [
        CoachStyle::Tank,
        CoachStyle::Passenger,
        CoachStyle::Observation,
    ];
    config::save(&loaded, &path).unwrap();
    assert_eq!(config::load(Some(&path)), (loaded.normalized(), None));
    let saved = read_json(&path);
    assert_eq!(saved["visual"]["future_effect"]["amount"], 7);
    assert_eq!(saved["visual"]["scene"]["future_sky"], "blue");
    assert_eq!(saved["visual"]["scene"]["weather"], "clear");
}

#[test]
fn unsupported_future_visual_mode_is_preserved_on_disk() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let original = br#"{"visual":{"theme":"future_3d"},"fps":45}"#;
    fs::write(&path, original).unwrap();
    let (loaded, warning) = config::load(Some(&path));
    assert!(warning.is_some());
    assert!(config::save(&loaded, &path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn original_seven_field_config_keeps_hidden_hud_and_acquires_safe_defaults() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let original = br#"{
        "min_speed_kmh":12.0,"max_speed_kmh":100.0,"smoothing_k":1.8,
        "fps":45,"metrics_interval_ms":1000,
        "show_metrics":false,"always_on_top":false
    }"#;
    fs::write(&path, original).expect("write original configuration");

    let (loaded, warning) = config::load(Some(&path));
    assert_eq!(warning, None);
    assert_eq!(loaded.fps, 45);
    assert!(!loaded.always_on_top);
    assert_eq!(loaded.hud_mode, HudMode::Compact);
    assert_eq!(loaded.effective_hud_mode(), HudMode::Clean);
    assert!(!loaded.pause_when_unfocused);
    assert!(!loaded.start_paused);
    assert!(loaded.auto_hide_controls);
    assert_eq!(loaded.clone().normalized(), loaded);
    assert_eq!(fs::read(&path).expect("read unchanged config"), original);

    // An explicit preference save upgrades the shape without changing the
    // legacy flag's meaning or making the hidden HUD reappear on next launch.
    config::save(&loaded, &path).expect("save compatible configuration");
    assert_eq!(config::load(Some(&path)), (loaded.normalized(), None));
}

#[test]
fn partial_new_config_uses_current_defaults_without_writing() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let original = br#"{"hud_mode":"detailed","start_paused":true}"#;
    fs::write(&path, original).expect("write partial configuration");
    let (loaded, warning) = config::load(Some(&path));
    assert_eq!(warning, None);
    assert_eq!(loaded.effective_hud_mode(), HudMode::Detailed);
    assert!(loaded.start_paused);
    assert_eq!(loaded.fps, 60);
    assert_eq!(loaded.min_speed_kmh, 12.0);
    assert_eq!(loaded.max_speed_kmh, 100.0);
    assert!(!loaded.pause_when_unfocused);
    assert!(loaded.auto_hide_controls);
    assert_eq!(fs::read(&path).expect("read unchanged config"), original);
}

#[test]
fn new_preferences_survive_an_atomic_save_and_a_second_launch() {
    let directory = TestDirectory::new();
    let path = directory.file();
    config::save(&Config::default(), &path).expect("save initial config");
    let mut changed = Config {
        fps: 60,
        smoothing_k: 0.5,
        always_on_top: false,
        pause_when_unfocused: true,
        start_paused: true,
        auto_hide_controls: false,
        ..Config::default()
    };
    changed.set_hud_mode(HudMode::Detailed);
    config::save(&changed, &path).expect("replace config");

    assert_eq!(config::load(Some(&path)), (changed, None));
    let saved = read_json(&path);
    assert_eq!(saved["hud_mode"], "detailed");
    assert_eq!(saved["show_metrics"], true);
    assert_eq!(saved["pause_when_unfocused"], true);
    assert_eq!(saved["start_paused"], true);
    assert_eq!(saved["auto_hide_controls"], false);
    assert_eq!(directory.entries(), 1, "no stale temporary file");
}

#[test]
fn cycling_hud_restores_visibility_after_a_legacy_hidden_configuration() {
    let mut preferences = Config {
        show_metrics: false,
        ..Config::default()
    };
    assert_eq!(preferences.effective_hud_mode(), HudMode::Clean);
    for expected in [HudMode::Compact, HudMode::Detailed, HudMode::Clean] {
        preferences.set_hud_mode(preferences.effective_hud_mode().next());
        assert_eq!(preferences.effective_hud_mode(), expected);
        assert_eq!(preferences.show_metrics, expected != HudMode::Clean);
    }
}

#[test]
fn either_explicit_clean_mode_or_legacy_false_hides_hud() {
    for input in [
        r#"{"show_metrics":false,"hud_mode":"detailed"}"#,
        r#"{"show_metrics":true,"hud_mode":"clean"}"#,
    ] {
        let preferences: Config = serde_json::from_str(input).expect("valid configuration");
        assert_eq!(preferences.effective_hud_mode(), HudMode::Clean);
        assert_eq!(preferences.clone().normalized(), preferences);
    }
}

#[test]
fn unknown_fields_survive_user_changes_and_subsequent_saves() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let extension = json!({
        "bioma": "café e montanhas",
        "visits": [3, 5, 8],
        "options": {"enabled": true, "label": null}
    });
    let original = json!({
        "fps": 30,
        "future_revision": 7,
        "future_ui": extension,
        "unknown_list": [null, false, "keep me", 1.25]
    });
    fs::write(
        &path,
        serde_json::to_vec(&original).expect("encode fixture"),
    )
    .expect("write future configuration");
    let (mut preferences, warning) = config::load(Some(&path));
    assert_eq!(warning, None);
    preferences.set_hud_mode(HudMode::Clean);
    preferences.pause_when_unfocused = true;
    config::save(&preferences, &path).expect("save while preserving unknown fields");
    preferences.fps = 45;
    config::save(&preferences, &path).expect("save again");

    let saved = read_json(&path);
    assert_eq!(saved["future_revision"], original["future_revision"]);
    assert_eq!(saved["future_ui"], original["future_ui"]);
    assert_eq!(saved["unknown_list"], original["unknown_list"]);
    assert_eq!(saved["fps"], 45);
    assert_eq!(saved["hud_mode"], "clean");
    assert_eq!(saved["show_metrics"], false);
    assert_eq!(config::load(Some(&path)), (preferences, None));
    assert_eq!(directory.entries(), 1);
}

#[test]
fn corrupt_or_incompatible_configs_cannot_be_overwritten_by_autosave() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let desired = Config {
        start_paused: true,
        ..Config::default()
    };
    for original in [
        b"not JSON".as_slice(),
        b"{\"fps\":30,".as_slice(),
        &[0xff, 0xfe],
        br#"{"hud_mode":"future_mode","future_state":{"keep":true}}"#,
        br#"{"pause_when_unfocused":"yes"}"#,
        br#"{"fps":30,"fps":45}"#,
        br#"{"hud_mode":"compact","hud_mode":"clean"}"#,
        b"[]",
        b"null",
    ] {
        fs::write(&path, original).expect("write invalid configuration");
        let (loaded, warning) = config::load(Some(&path));
        assert_eq!(loaded, Config::default());
        assert!(warning.is_some());
        assert!(config::save(&desired, &path).is_err());
        assert_eq!(fs::read(&path).expect("read preserved bytes"), original);
        assert_eq!(
            directory.entries(),
            1,
            "no temporary or backup side effects"
        );
    }
}

#[test]
fn valid_preference_bounds_are_repaired_without_losing_future_fields() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let original = br#"{
        "fps":0,"smoothing_k":1000,"metrics_interval_ms":1,
        "min_speed_kmh":-1,"max_speed_kmh":999,
        "hud_mode":"compact","future_flag":true
    }"#;
    fs::write(&path, original).expect("write out-of-range configuration");
    let (preferences, warning) = config::load(Some(&path));
    assert!(warning.is_some());
    assert_eq!(preferences.fps, 10);
    assert_eq!(preferences.smoothing_k, 20.0);
    assert_eq!(preferences.metrics_interval_ms, 250);
    assert_eq!(preferences.min_speed_kmh, 0.0);
    assert_eq!(preferences.max_speed_kmh, 500.0);
    assert_eq!(fs::read(&path).expect("read unchanged bytes"), original);

    config::save(&preferences, &path).expect("explicitly save repaired values");
    assert_eq!(read_json(&path)["future_flag"], true);
    assert_eq!(config::load(Some(&path)), (preferences, None));
}

#[test]
fn no_file_is_created_until_a_preference_is_explicitly_saved() {
    let directory = TestDirectory::new();
    let path = directory.file();
    let (mut preferences, warning) = config::load(Some(&path));
    assert_eq!(warning, None);
    assert!(!path.exists());
    preferences.set_hud_mode(HudMode::Clean);
    preferences.start_paused = true;
    assert!(
        !path.exists(),
        "changing in-memory preferences performs no I/O"
    );

    config::save(&preferences, &path).expect("save explicitly");
    assert_eq!(config::load(Some(&path)), (preferences, None));
    assert_eq!(directory.entries(), 1);
}

#[test]
fn retired_visual_modes_migrate_without_changing_window_or_pacing() {
    use cpu_express::rendering::{RenderQuality, SceneTheme};
    let config: Config = serde_json::from_str(r#"{"fps":30,"taskbar_mode":false,"floating_rect":[120,140,800,240],"visual":{"theme":"legacy","quality":"low","scene":{"details":false}}}"#).unwrap();
    let migrated = config.normalized();
    assert_eq!(migrated.visual.theme, SceneTheme::Journey);
    assert_eq!(migrated.visual.quality, RenderQuality::Full);
    assert!(migrated.visual.scene.details);
    assert_eq!(migrated.fps, 30);
    assert!(!migrated.taskbar_mode);
    assert_eq!(migrated.floating_rect, Some([120, 140, 800, 240]));
}
