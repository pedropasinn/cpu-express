//! Exercise the public help command without graphics or operating-system metrics.
//!
//! Invalid arguments are covered by the pure parser tests in `src/main.rs`:
//! spawning them here would intentionally open a native error dialog on Windows.

use std::{
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn help_and_short_alias_describe_demo_and_current_shortcuts() {
    for flag in ["--help", "-h"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cpu-express"))
            .arg(flag)
            .output()
            .expect("run the help command");
        assert!(output.status.success(), "{flag}: {:?}", output.stderr);
        let help = String::from_utf8(output.stdout).expect("UTF-8 help text");
        for expected in [
            "--demo",
            "--cpu",
            "--config",
            "--diagnostics",
            "--smoke-seconds",
            "--smoke",
            "não consulta CPU nem RAM reais",
            "Espaço",
            "HUD compacto, completo e limpo",
            "F1",
            "Esc",
            "não encerra",
            "Ctrl+Q",
        ] {
            assert!(help.contains(expected), "{flag}: missing {expected}");
        }
    }
}

#[test]
fn help_leaves_explicit_config_and_diagnostics_paths_untouched() {
    let root = unique_path();
    let config = root.join("pasta com espaços").join("preferências.json");
    let diagnostics = root.join("diagnóstico.jsonl");
    assert!(!root.exists());
    let output = Command::new(env!("CARGO_BIN_EXE_cpu-express"))
        .arg("--demo")
        .arg("--cpu")
        .arg("0")
        .arg("--config")
        .arg(&config)
        .arg("--diagnostics")
        .arg(&diagnostics)
        .arg("--smoke-seconds")
        .arg("0.1")
        .arg("--help")
        .output()
        .expect("run help with launch options");
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(
        !root.exists(),
        "--help must not initialize files or folders"
    );
}

fn unique_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cpu-express-ux-cli-{}-{timestamp}",
        std::process::id()
    ))
}
