#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{
    error::Error,
    ffi::OsString,
    io::{self, ErrorKind},
    path::{Component, Path, PathBuf},
    time::Duration,
};

use cpu_express::app::{CpuExpressApp, LaunchOptions};
use cpu_express::diagnostics::Diagnostics;
use cpu_express::{config, platform};
use eframe::egui;

const HELP: &str = concat!(
    "CPU Express ",
    env!("CARGO_PKG_VERSION"),
    " — um trem para acompanhar o computador\n\
\n\
Uso: cpu-express.exe [opções]\n\
  --demo               CPU simulada; não consulta CPU nem RAM reais.\n\
                       Começa em 50%; combine com --cpu para outro valor.\n\
  --cpu 0..100         CPU manual, sem gerar carga no processador.\n\
                       Fora de --demo, a coleta real de métricas continua.\n\
  --config CAMINHO     Preferências JSON locais.\n\
  --diagnostics CAMINHO  Registro JSONL local, apenas quando solicitado.\n\
  --smoke-seconds N    Encerra após N segundos (0.1 a 86400).\n\
  --smoke              Equivale a --smoke-seconds 3.\n\
  --help, -h           Mostra esta ajuda sem abrir a janela.\n\
\n\
Atalhos com a janela em foco, fora dos campos de entrada:\n\
  Espaço  Pausar ou retomar a viagem.\n\
  M       Alternar HUD compacto, completo e limpo.\n\
  P       Fixar ou soltar a janela no topo.\n\
  O       Abrir ou fechar as opções.\n\
  F1      Abrir ou fechar a ajuda.\n\
  R       Reposicionar perto da barra de tarefas.\n\
  Esc     Fechar o painel aberto; não encerra o aplicativo.\n\
  Ctrl+Q  Sair. O botão Sair também fica disponível nos controles.\n\
\n\
Em DEMO, a RAM aparece como indisponível; nenhum valor real é inventado.\n\
Use 0%, 50% e 100% no painel para experimentar sem aumentar a carga.\n\
Exemplo: cpu-express.exe --demo --cpu 50 --smoke-seconds 10"
);

#[derive(Debug, Default, PartialEq)]
struct CliOptions {
    demo: bool,
    cpu_override: Option<f32>,
    diagnostics_path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    exit_after: Option<Duration>,
}

#[derive(Debug, PartialEq)]
enum CommandLine {
    Help,
    Run(CliOptions),
}

/// Parse only; no files, graphics, metrics provider or worker are opened here.
///
/// Paths retain their native encoding. Only option names and numeric values
/// need UTF-8, so a Windows path with non-ASCII characters is not round-tripped
/// through a lossy string. As in the original CLI, the last repeated value wins.
fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<CommandLine, String> {
    let mut options = CliOptions::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--demo") => options.demo = true,
            Some("--cpu") => {
                let error = "--cpu exige um número de 0 a 100 (ex.: --cpu 50)";
                let value = args.next().ok_or(error)?;
                let cpu: f32 = value.to_str().ok_or(error)?.parse().map_err(|_| error)?;
                if !cpu.is_finite() || !(0.0..=100.0).contains(&cpu) {
                    return Err("--cpu deve ser um número finito entre 0 e 100".to_owned());
                }
                options.cpu_override = Some(cpu);
            }
            Some("--diagnostics") => {
                options.diagnostics_path = Some(path_argument(&mut args, "--diagnostics")?);
            }
            Some("--config") => {
                options.config_path = Some(path_argument(&mut args, "--config")?);
            }
            Some("--smoke") => options.exit_after = Some(Duration::from_secs(3)),
            Some("--smoke-seconds") => {
                let error = "--smoke-seconds exige segundos (ex.: --smoke-seconds 3)";
                let value = args.next().ok_or(error)?;
                let seconds: f64 = value.to_str().ok_or(error)?.parse().map_err(|_| error)?;
                if !seconds.is_finite() || !(0.1..=86400.0).contains(&seconds) {
                    return Err(
                        "--smoke-seconds deve ser um número finito entre 0.1 e 86400".to_owned(),
                    );
                }
                options.exit_after = Some(Duration::from_secs_f64(seconds));
            }
            Some("--help" | "-h") => return Ok(CommandLine::Help),
            _ => return Err(format!("Opção desconhecida: {}", arg.to_string_lossy())),
        }
    }
    // Resolve this after all arguments so --demo --cpu and --cpu --demo agree.
    if options.demo && options.cpu_override.is_none() {
        options.cpu_override = Some(50.0);
    }
    Ok(CommandLine::Run(options))
}

fn path_argument(args: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<PathBuf, String> {
    let path = args
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} exige um caminho de arquivo"))?;
    // A relative path beginning with "--" remains valid when written as ./--… .
    if path.to_str().is_some_and(|value| value.starts_with("--")) {
        return Err(format!(
            "{flag} exige um caminho de arquivo antes da próxima opção"
        ));
    }
    Ok(PathBuf::from(path))
}

/// Compare destinations before the diagnostic writer can truncate a file.
///
/// Each existing prefix is canonicalized before processing the next component,
/// so relative paths, `..` and symlinked parent directories agree even when the
/// final file does not exist yet. This guard prevents accidental aliasing; it
/// does not claim to defend against concurrent filesystem changes or hard links.
fn resolved_destination(path: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let mut resolved = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(_) => {
                resolved.push(component.as_os_str());
                match std::fs::canonicalize(&resolved) {
                    Ok(canonical) => resolved = canonical,
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
            Component::Prefix(_) | Component::RootDir => resolved.push(component.as_os_str()),
        }
    }
    Ok(resolved)
}

fn ensure_separate_destinations(config_path: &Path, diagnostics_path: &Path) -> Result<(), String> {
    let resolve = |path: &Path| {
        resolved_destination(path).map_err(|error| {
            format!("Não foi possível verificar os caminhos de configuração e diagnóstico: {error}")
        })
    };
    let config = resolve(config_path)?;
    let diagnostics = resolve(diagnostics_path)?;
    #[cfg(windows)]
    let same_destination =
        config.to_string_lossy().to_lowercase() == diagnostics.to_string_lossy().to_lowercase();
    #[cfg(not(windows))]
    let same_destination = config == diagnostics;
    if same_destination {
        return Err(
            "--diagnostics deve usar um arquivo diferente da configuração; o arquivo original foi preservado."
                .to_owned(),
        );
    }
    Ok(())
}

fn open_diagnostics(
    config_path: Option<&Path>,
    diagnostics_path: Option<&Path>,
) -> Result<Option<Diagnostics>, Box<dyn Error>> {
    let Some(diagnostics_path) = diagnostics_path else {
        return Ok(None);
    };
    if let Some(config_path) = config_path {
        ensure_separate_destinations(config_path, diagnostics_path)?;
    }
    Ok(Some(Diagnostics::open(diagnostics_path)?))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("CPU Express: {error}");
        platform::show_error(&format!("CPU Express não pôde iniciar.\n\n{error}"));
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let CommandLine::Run(arguments) = parse_args(std::env::args_os().skip(1))? else {
        println!("{HELP}");
        return Ok(());
    };
    let CliOptions {
        demo,
        cpu_override,
        diagnostics_path,
        config_path,
        exit_after,
    } = arguments;
    if !cfg!(windows) && !cfg!(feature = "host-validation") {
        return Err("Esta versão é destinada ao Windows. Para validar a janela no host de desenvolvimento, use --features host-validation. O porte Linux é futuro.".into());
    }
    let config_path = config_path.or_else(config::default_path);
    let (config, warning) = config::load(config_path.as_deref());
    #[cfg(windows)]
    let _profile_lock = config_path
        .as_deref()
        .map(cpu_express::progress::lock_profile)
        .transpose()?;
    if let (Some(config), Some(diagnostics)) = (&config_path, &diagnostics_path) {
        ensure_separate_destinations(&cpu_express::progress::path_for(config), diagnostics)?;
    }
    let diagnostics = open_diagnostics(config_path.as_deref(), diagnostics_path.as_deref())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 140.0])
            .with_resizable(!config.taskbar_mode)
            .with_decorations(!config.taskbar_mode)
            .with_active(false)
            .with_taskbar(true)
            .with_app_id("cpu-express")
            .with_window_level(if config.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            }),
        renderer: eframe::Renderer::Glow,
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "CPU Express",
        options,
        Box::new(move |cc| {
            let launch_options = LaunchOptions {
                config,
                config_path,
                warning,
                cpu_override,
                exit_after,
                diagnostics,
            };
            let app = if demo {
                CpuExpressApp::new_demo(&cc.egui_ctx, launch_options)
            } else {
                CpuExpressApp::new(&cc.egui_ctx, launch_options)
            };
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CliOptions, CommandLine, ensure_separate_destinations, open_diagnostics, parse_args,
    };
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock")
                .as_nanos();
            // A fixture below cwd permits relative-path tests without mutating
            // the process-wide current directory or environment in parallel.
            let path = std::env::current_dir().unwrap().join(format!(
                "cpu-express-ux-path-test-{}-{timestamp}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("create isolated fixture");
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn parse(args: &[&str]) -> Result<CommandLine, String> {
        parse_args(args.iter().map(OsString::from))
    }

    fn options(args: &[&str]) -> CliOptions {
        let CommandLine::Run(options) = parse(args).expect("valid arguments") else {
            panic!("expected launch arguments");
        };
        options
    }

    #[test]
    fn normal_launch_keeps_real_cpu_and_demo_is_explicit() {
        assert_eq!(options(&[]), CliOptions::default());
        let demo = options(&["--demo"]);
        assert!(demo.demo);
        assert_eq!(demo.cpu_override, Some(50.0));
        let manual = options(&["--cpu", "50"]);
        assert!(!manual.demo);
        assert_eq!(manual.cpu_override, Some(50.0));
    }

    #[test]
    fn explicit_demo_cpu_is_independent_of_flag_order() {
        for value in ["0", "12.5", "50", "100"] {
            let before = options(&["--demo", "--cpu", value]);
            let after = options(&["--cpu", value, "--demo"]);
            assert_eq!(before, after);
            assert_eq!(before.cpu_override, Some(value.parse().unwrap()));
        }
    }

    #[test]
    fn original_flags_can_be_combined_with_demo() {
        let parsed = options(&[
            "--config",
            "pasta com espaços/preferências.json",
            "--diagnostics",
            "registros/sessão.jsonl",
            "--smoke-seconds",
            "2.5",
            "--cpu",
            "0",
            "--demo",
        ]);
        assert_eq!(
            parsed.config_path,
            Some(PathBuf::from("pasta com espaços/preferências.json"))
        );
        assert_eq!(
            parsed.diagnostics_path,
            Some(PathBuf::from("registros/sessão.jsonl"))
        );
        assert_eq!(parsed.exit_after, Some(Duration::from_millis(2_500)));
        assert_eq!(parsed.cpu_override, Some(0.0));
        assert!(parsed.demo);
    }

    #[test]
    fn original_smoke_alias_and_last_value_semantics_are_preserved() {
        assert_eq!(
            options(&["--smoke"]).exit_after,
            Some(Duration::from_secs(3))
        );
        assert_eq!(
            options(&["--smoke", "--smoke-seconds", "0.1"]).exit_after,
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            options(&["--smoke-seconds", "10", "--smoke"]).exit_after,
            Some(Duration::from_secs(3))
        );
        assert_eq!(
            options(&["--cpu", "0", "--cpu", "100"]).cpu_override,
            Some(100.0)
        );
    }

    #[test]
    fn numeric_boundaries_are_safe_and_invalid_cpu_is_rejected() {
        for value in ["NaN", "inf", "-inf", "-1", "100.1", "texto", ""] {
            let error = parse(&["--cpu", value]).unwrap_err();
            assert!(error.starts_with("--cpu"), "{value}: {error}");
        }
        for value in ["NaN", "inf", "-1", "0", "0.09", "86400.1", "texto", ""] {
            let error = parse(&["--smoke-seconds", value]).unwrap_err();
            assert!(error.starts_with("--smoke-seconds"), "{value}: {error}");
        }
        assert_eq!(
            options(&["--smoke-seconds", "86400"]).exit_after,
            Some(Duration::from_secs(86_400))
        );
    }

    #[test]
    fn missing_values_and_unknown_flags_report_portuguese_errors() {
        for flag in ["--cpu", "--config", "--diagnostics", "--smoke-seconds"] {
            let error = parse(&[flag]).unwrap_err();
            assert!(error.starts_with(flag));
            assert!(error.contains("exige"));
        }
        assert_eq!(
            parse(&["--surpresa"]).unwrap_err(),
            "Opção desconhecida: --surpresa"
        );
        assert!(parse(&["--config", "--demo"]).is_err());
        assert!(parse(&["--diagnostics", ""]).is_err());
    }

    #[test]
    fn help_short_circuits_before_starting_the_application() {
        assert_eq!(parse(&["--help"]).unwrap(), CommandLine::Help);
        assert_eq!(parse(&["--demo", "-h"]).unwrap(), CommandLine::Help);
        assert_eq!(
            parse(&["--config", "não-criar.json", "--help"]).unwrap(),
            CommandLine::Help
        );
    }

    #[test]
    fn diagnostics_rejects_relative_absolute_and_dot_aliases() {
        let directory = TestDirectory::new();
        let config = directory.file("preferencias.json");
        fs::write(&config, b"{\"fps\":30}").unwrap();
        fs::create_dir(directory.file("subpasta")).unwrap();
        let relative = Path::new(directory.0.file_name().unwrap()).join("preferencias.json");
        for alias in [
            relative,
            directory.file("./preferencias.json"),
            directory.file("subpasta/../preferencias.json"),
        ] {
            assert!(ensure_separate_destinations(&config, &alias).is_err());
        }
        assert!(
            ensure_separate_destinations(&config, &directory.file("diagnostico.jsonl")).is_ok()
        );
    }

    #[test]
    fn diagnostic_open_cannot_truncate_the_configuration() {
        let directory = TestDirectory::new();
        let config = directory.file("preferencias.json");
        let original = b"{\"fps\":30,\"future_field\":{\"keep\":true}}\n";
        fs::write(&config, original).unwrap();
        let result = open_diagnostics(Some(&config), Some(&config));
        assert!(result.is_err());
        assert_eq!(fs::read(&config).unwrap(), original);
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    #[test]
    fn matching_new_destinations_do_not_create_any_file() {
        let directory = TestDirectory::new();
        let config = directory.file("ainda-nao-existe.json");
        let alias = directory.file("./ainda-nao-existe.json");
        assert!(open_diagnostics(Some(&config), Some(&alias)).is_err());
        assert!(!config.exists());
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
    }

    #[test]
    fn a_distinct_diagnostic_file_preserves_config_and_records_normally() {
        let directory = TestDirectory::new();
        let config = directory.file("preferencias.json");
        let output = directory.file("diagnostico.jsonl");
        let original = b"{\"fps\":30}\n";
        fs::write(&config, original).unwrap();
        let mut diagnostics = open_diagnostics(Some(&config), Some(&output))
            .expect("separate destinations")
            .expect("requested diagnostic writer");
        diagnostics
            .record(&serde_json::json!({"event": "path_guard_test"}))
            .expect("write one record");
        drop(diagnostics);
        assert_eq!(fs::read(&config).unwrap(), original);
        assert_eq!(
            fs::read_to_string(output).unwrap(),
            "{\"event\":\"path_guard_test\"}\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_files_and_parents_cannot_alias_the_configuration() {
        use std::os::unix::fs::symlink;

        let directory = TestDirectory::new();
        let config = directory.file("preferencias.json");
        let original = b"{\"start_paused\":true}\n";
        fs::write(&config, original).unwrap();
        let alias = directory.file("diagnostico-link.jsonl");
        symlink(&config, &alias).unwrap();
        assert!(open_diagnostics(Some(&config), Some(&alias)).is_err());
        assert_eq!(fs::read(&config).unwrap(), original);

        let directory_alias = directory.file("pasta-link");
        symlink(&directory.0, &directory_alias).unwrap();
        assert!(
            ensure_separate_destinations(
                &directory.file("novo.json"),
                &directory_alias.join("novo.json")
            )
            .is_err()
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_case_alias_does_not_truncate_configuration() {
        let directory = TestDirectory::new();
        let config = directory.file("preferencias.json");
        let original = b"{\"fps\":30}\n";
        fs::write(&config, original).unwrap();
        let alias = directory.file("PREFERENCIAS.JSON");
        assert!(open_diagnostics(Some(&config), Some(&alias)).is_err());
        assert_eq!(fs::read(&config).unwrap(), original);
    }

    #[cfg(unix)]
    #[test]
    fn native_paths_are_not_lossily_reencoded() {
        use std::os::unix::ffi::OsStringExt;

        let path = OsString::from_vec(b"local-\xff.json".to_vec());
        let parsed = parse_args([OsString::from("--config"), path.clone()]).unwrap();
        let CommandLine::Run(parsed) = parsed else {
            panic!("expected launch arguments");
        };
        assert_eq!(parsed.config_path, Some(PathBuf::from(path)));
    }
}
