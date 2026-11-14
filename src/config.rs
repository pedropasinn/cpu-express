//! User preferences and the small set of parameters used by the simulation.

use crate::rendering::{CoachStyle, RenderQuality, SceneOptions, SceneTheme};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

/// Amount of information drawn over the scenery.
///
/// The legacy `show_metrics` flag can still hide any mode; use
/// [`Config::effective_hud_mode`] when deciding what to display.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HudMode {
    Detailed,
    #[default]
    Compact,
    Clean,
}

impl HudMode {
    /// A deterministic cycle for the HUD button and its keyboard shortcut.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Compact => Self::Detailed,
            Self::Detailed => Self::Clean,
            Self::Clean => Self::Compact,
        }
    }
}

/// Saved presentation choices. These never change CPU load or journey rewards.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VisualPreferences {
    pub livery: crate::simulation::objectives::Livery,
    pub theme: SceneTheme,
    pub quality: RenderQuality,
    pub scene: SceneOptions,
    /// Ordered from the tail toward the locomotive.
    pub coaches: [CoachStyle; 3],
}

impl Default for VisualPreferences {
    fn default() -> Self {
        Self {
            livery: crate::simulation::objectives::Livery::default(),
            theme: SceneTheme::Journey,
            quality: RenderQuality::Full,
            scene: SceneOptions::default(),
            coaches: [
                CoachStyle::Observation,
                CoachStyle::Sleeper,
                CoachStyle::Freight,
            ],
        }
    }
}

/// Configuration has no platform handles or mutable global state.
///
/// Missing fields in an older configuration file receive their current default.
/// Call [`Self::normalized`] after loading user input before using the values.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    /// Embed at the left of the primary Windows taskbar; floating mode remains available.
    pub taskbar_mode: bool,
    pub taskbar_offset: i32,
    /// Physical pixels; zero follows the taskbar automatically.
    pub taskbar_width: i32,
    pub taskbar_height: i32,
    /// Physical outer rectangle [x, y, width, height] in floating mode.
    pub floating_rect: Option<[i32; 4]>,
    /// Independent menu geometry, in logical points.
    pub panel_rect: Option<[f32; 4]>,
    pub reduce_menu_motion: bool,
    pub quiet_mode: bool,
    pub min_speed_kmh: f64,
    pub max_speed_kmh: f64,
    /// Response rate in inverse seconds; larger values accelerate more quickly.
    pub smoothing_k: f64,
    pub fps: u32,
    pub metrics_interval_ms: u64,
    /// Legacy visibility flag. A false value always hides the metrics HUD.
    pub show_metrics: bool,
    pub always_on_top: bool,
    pub hud_mode: HudMode,
    /// Opt-in only: desktop companions normally keep moving without focus.
    pub pause_when_unfocused: bool,
    pub start_paused: bool,
    pub auto_hide_controls: bool,
    pub visual: VisualPreferences,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            taskbar_mode: true,
            taskbar_offset: 0,
            taskbar_width: 0,
            taskbar_height: 0,
            floating_rect: None,
            panel_rect: None,
            reduce_menu_motion: false,
            quiet_mode: false,
            min_speed_kmh: 12.0,
            max_speed_kmh: 100.0,
            smoothing_k: 1.8,
            fps: 60,
            metrics_interval_ms: 1_000,
            show_metrics: true,
            always_on_top: true,
            hud_mode: HudMode::Compact,
            pause_when_unfocused: false,
            start_paused: false,
            auto_hide_controls: true,
            visual: VisualPreferences::default(),
        }
    }
}

impl Config {
    /// Resolve old and new preferences without rewriting a legacy file.
    ///
    /// `show_metrics = false` wins over any stored mode. This preserves old
    /// JSON and Rust struct literals that only know about the visibility flag.
    /// An explicit `Clean` mode also remains clean if the legacy flag is true.
    #[must_use]
    pub fn effective_hud_mode(&self) -> HudMode {
        if self.show_metrics {
            self.hud_mode
        } else {
            HudMode::Clean
        }
    }

    /// Apply an explicit HUD choice and keep the legacy visibility in sync.
    pub fn set_hud_mode(&mut self, mode: HudMode) {
        self.hud_mode = mode;
        self.show_metrics = mode != HudMode::Clean;
    }

    /// Repair invalid numeric input and keep experimental settings reasonable.
    ///
    /// The collector additionally respects sysinfo's platform-specific minimum
    /// interval. The UI never has to trust serialized floating-point values.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        self.visual.theme = SceneTheme::Journey;
        self.visual.quality = RenderQuality::Full;
        self.visual.scene.details = true;
        self.visual.scene.clouds = true;
        self.visual.scene.day_cycle = true;
        self.visual.scene.weather = crate::rendering::Weather::Clear;
        self.taskbar_offset = self.taskbar_offset.clamp(0, 32768);
        self.taskbar_width = if self.taskbar_width == 0 {
            0
        } else {
            self.taskbar_width.clamp(160, 2400)
        };
        self.taskbar_height = if self.taskbar_height == 0 {
            0
        } else {
            self.taskbar_height.clamp(24, 240)
        };
        self.floating_rect = self.floating_rect.map(|[x, y, w, h]| {
            [
                x.clamp(-32768, 32768),
                y.clamp(-32768, 32768),
                w.clamp(320, 2400),
                h.clamp(140, 1200),
            ]
        });
        self.panel_rect = self.panel_rect.filter(|r| r.iter().all(|v| v.is_finite())).map(|[x,y,w,h]| [x.clamp(-32768.0,32768.0),y.clamp(-32768.0,32768.0),w.clamp(480.0,2400.0),h.clamp(480.0,1600.0)]);
        let defaults = Self::default();
        self.min_speed_kmh =
            finite_or(self.min_speed_kmh, defaults.min_speed_kmh).clamp(0.0, 300.0);
        self.max_speed_kmh =
            finite_or(self.max_speed_kmh, defaults.max_speed_kmh).clamp(self.min_speed_kmh, 500.0);
        self.smoothing_k = finite_or(self.smoothing_k, defaults.smoothing_k).clamp(0.05, 20.0);
        self.fps = self.fps.clamp(10, 60);
        self.metrics_interval_ms = self.metrics_interval_ms.clamp(250, 60_000);
        self
    }
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

/// An explicit environment override also supports isolated development tests.
/// Linux receives no product-specific storage integration in this release.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CPU_EXPRESS_CONFIG").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(path));
    }
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .filter(|path| !path.is_empty())
            .map(|path| PathBuf::from(path).join("CPU Express").join("config.json"))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Load preferences without ever modifying the source file.
///
/// A missing file means a first launch. Invalid or unreadable files produce a
/// warning and safe defaults. Corrupt or incompatible user data remains intact
/// even if a later preference change attempts to save it: [`save`] refuses to
/// overwrite a document it cannot safely understand.
#[must_use]
pub fn load(path: Option<&Path>) -> (Config, Option<String>) {
    let Some(path) = path else {
        return (Config::default(), None);
    };
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return (Config::default(), None),
        Err(error) => {
            return (
                Config::default(),
                Some(format!(
                    "Não foi possível ler a configuração em {}: {error}. Usando os padrões.",
                    path.display()
                )),
            );
        }
    };
    match parse_document(&text) {
        Ok((config, _)) => {
            let normalized = config.clone().normalized();
            // Intentional visual migration is not invalid user input.
            let mut comparable = config.clone();
            comparable.visual = normalized.visual.clone();
            let warning = (normalized != comparable).then(|| {
                "Alguns valores da configuração estavam fora dos limites e foram ajustados. O arquivo original foi preservado."
                    .to_owned()
            });
            (normalized, warning)
        }
        Err(error) => (
            Config::default(),
            Some(format!(
                "Configuração inválida em {}: {error}. Usando os padrões; o arquivo original foi preservado.",
                path.display()
            )),
        ),
    }
}

/// Save a complete JSON file next to the destination, then atomically replace it.
///
/// This function is only for explicit preference changes. It must not be called
/// automatically just because [`load`] returned defaults or a warning.
/// Unknown fields of a valid document are retained, including nested values.
/// Corrupt, incompatible or unreadable documents are never replaced: callers
/// receive a recoverable error and can keep using the preference in memory.
/// `rename` stays on the same filesystem and the existing file is never removed
/// first; a write/sync/rename failure therefore leaves its contents untouched.
pub fn save(config: &Config, path: &Path) -> Result<(), String> {
    if path.file_name().is_none() {
        return Err("O caminho da configuração deve indicar um arquivo.".to_owned());
    }
    // Read and validate before creating a directory or temporary file. A
    // future version's unknown fields survive, but a known field with an
    // unsupported type/value must not be silently replaced with our defaults.
    let mut document = match fs::read_to_string(path) {
        Ok(text) => parse_document(&text)
            .map(|(_, document)| document)
            .map_err(|error| {
                format!(
                    "Preferências válidas só nesta sessão: a configuração em {} está inválida ou é incompatível ({error}). O arquivo original foi preservado; corrija-o ou escolha outro caminho com --config.",
                    path.display()
                )
            })?,
        Err(error) if error.kind() == ErrorKind::NotFound => serde_json::Map::new(),
        Err(error) => {
            return Err(format!(
                "Não foi possível ler a configuração em {} antes de salvar: {error}. O arquivo original foi preservado.",
                path.display()
            ));
        }
    };
    let values = serde_json::to_value(config.clone().normalized())
        .map_err(|error| format!("Não foi possível codificar a configuração: {error}"))?;
    let serde_json::Value::Object(values) = values else {
        return Err("Não foi possível codificar a configuração como objeto JSON.".to_owned());
    };
    merge_preferences(&mut document, values);
    let mut bytes = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("Não foi possível codificar a configuração: {error}"))?;
    bytes.push(b'\n');

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Não foi possível criar a pasta da configuração {}: {error}",
            parent.display()
        )
    })?;
    let (temporary, file) = create_temporary(parent)?;
    // The closure owns the file: its handle closes on both success and failure
    // before TemporaryPath attempts cleanup (important for Windows file locks).
    let write_result = (|| {
        let mut file = file;
        file.write_all(&bytes)?;
        file.sync_all()
    })();
    write_result.map_err(|error| {
        format!(
            "Não foi possível gravar a configuração em {}: {error}",
            path.display()
        )
    })?;
    fs::rename(&temporary.0, path).map_err(|error| {
        format!(
            "Não foi possível substituir a configuração em {}: {error}. O arquivo anterior foi preservado.",
            path.display()
        )
    })?;
    Ok(())
}

// New nested presentation preferences must not erase unknown fields written
// by another compatible version. Known scalar/array values remain authoritative.
fn merge_preferences(
    document: &mut serde_json::Map<String, serde_json::Value>,
    values: serde_json::Map<String, serde_json::Value>,
) {
    for (key, value) in values {
        if let serde_json::Value::Object(new_fields) = value {
            if let Some(serde_json::Value::Object(existing)) = document.get_mut(&key) {
                merge_preferences(existing, new_fields);
            } else {
                document.insert(key, serde_json::Value::Object(new_fields));
            }
        } else {
            document.insert(key, value);
        }
    }
}

/// Parse both views from the same bytes. Parsing `Config` directly also catches
/// duplicate known keys, which would be lost by first deserializing a `Value`.
fn parse_document(
    text: &str,
) -> Result<(Config, serde_json::Map<String, serde_json::Value>), serde_json::Error> {
    let document = serde_json::from_str(text)?;
    let config = serde_json::from_str(text)?;
    Ok((config, document))
}

/// Removes our temporary file after any error; after a rename it is already gone.
struct TemporaryPath(PathBuf);

impl Drop for TemporaryPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn create_temporary(parent: &Path) -> Result<(TemporaryPath, File), String> {
    static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    for _ in 0..32 {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".cpu-express-config-{}-{timestamp}-{sequence}.tmp",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((TemporaryPath(path), file)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Não foi possível criar um arquivo temporário em {}: {error}",
                    parent.display()
                ));
            }
        }
    }
    Err("Não foi possível reservar um arquivo temporário para a configuração.".to_owned())
}
