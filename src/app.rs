//! Desktop integration: presentation consumes metrics and journey snapshots.

use crate::{
    config::{self, Config, HudMode},
    diagnostics::Diagnostics,
    local_weather::LocalWeather,
    metrics::{
        ManualProvider, MetricStatus, MetricsProvider, MetricsSample, MetricsWorker,
        SampleFreshness,
    },
    platform,
    rendering::{CoachStyle, HEIGHT, SceneContext, SceneRenderer, WIDTH},
    simulation::{Achievement, JourneySnapshot, Train, TravelEvent, World},
};
use crate::{
    ui_game::{self as game, Icon},
    ui_stock::StockGallery,
};
mod branches;
mod journey;
mod menus;
mod missions;
mod scenery;
mod services;
mod storage;
mod workshop;

use eframe::egui;
#[cfg(windows)]
use raw_window_handle::HasWindowHandle;
use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

const INK: egui::Color32 = egui::Color32::from_rgb(245, 237, 211);
const PANEL: egui::Color32 = egui::Color32::from_rgb(16, 28, 31);
const CARD: egui::Color32 = egui::Color32::from_rgb(27, 43, 46);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(242, 197, 112);

fn section(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    game::section_frame(ui, title, |ui| {
        game::apply_style(ui);
        content(ui);
    });
    ui.add_space(8.0);
}

fn stock_card(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    name: &str,
    state: &str,
    selected: bool,
    unlocked: bool,
) -> egui::Response {
    crate::ui_stock::stock_card(
        ui,
        name,
        &crate::ui_stock::StockCard {
            texture,
            name,
            state,
            selected,
            locked_reason: (!unlocked).then_some(state),
        },
    )
    .response
}

fn achievement_name(achievement: Achievement) -> &'static str {
    match achievement {
        Achievement::FirstStation => "Primeira estação",
        Achievement::FirstCircuit => "Primeiro circuito",
        Achievement::LongJourney => "Longa jornada",
    }
}

fn service_wagon(style: CoachStyle) -> crate::simulation::ServiceWagon {
    style.into()
}
fn coach_style(style: crate::simulation::ServiceWagon) -> CoachStyle {
    style.into()
}

fn coach_name(style: CoachStyle) -> &'static str {
    style.name()
}

/// Kept source-compatible with the baseline and its examples.
pub struct LaunchOptions {
    pub config: Config,
    pub config_path: Option<PathBuf>,
    pub warning: Option<String>,
    pub cpu_override: Option<f32>,
    pub exit_after: Option<Duration>,
    pub diagnostics: Option<Diagnostics>,
}

/// Local commands shared by buttons and shortcuts. No global keyboard hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    TogglePause,
    CycleHud,
    TogglePin,
    ToggleControls,
    ToggleHelp,
    Reposition,
    Minimize,
    HideToTray,
    ClosePanel,
    Quit,
}

/// Text/slider input owns keys. Escape only closes a panel; it never exits.
#[must_use]
pub fn shortcut_action(
    key: egui::Key,
    modifiers: egui::Modifiers,
    keyboard_in_use: bool,
    panel_open: bool,
) -> Option<AppAction> {
    if key == egui::Key::Escape {
        return panel_open.then_some(AppAction::ClosePanel);
    }
    if keyboard_in_use {
        return None;
    }
    if key == egui::Key::Q && modifiers.ctrl && !modifiers.alt && !modifiers.shift {
        return Some(AppAction::Quit);
    }
    if modifiers.ctrl || modifiers.alt || modifiers.command || modifiers.shift {
        return None;
    }
    match key {
        egui::Key::Space => Some(AppAction::TogglePause),
        egui::Key::M => Some(AppAction::CycleHud),
        egui::Key::P => Some(AppAction::TogglePin),
        egui::Key::O => Some(AppAction::ToggleControls),
        egui::Key::F1 => Some(AppAction::ToggleHelp),
        egui::Key::R => Some(AppAction::Reposition),
        _ => None,
    }
}

/// Relative timestamps make pause/resume tests independent of sleeping or a window.
#[derive(Clone, Debug)]
pub struct SimulationClock {
    last: Duration,
    suspended: bool,
}

impl SimulationClock {
    #[must_use]
    pub fn new(now: Duration) -> Self {
        Self {
            last: now,
            suspended: false,
        }
    }

    pub fn set_suspended(&mut self, suspended: bool, now: Duration) {
        if suspended != self.suspended {
            self.last = now;
            self.suspended = suspended;
        }
    }

    pub fn take_step(&mut self, now: Duration) -> f64 {
        let delta = now.saturating_sub(self.last);
        self.last = now;
        if self.suspended {
            0.0
        } else {
            delta.as_secs_f64().min(0.25)
        }
    }
}

/// Read-only integration state; contains no platform handles or mutable providers.
#[derive(Clone, Debug)]
pub struct SessionSnapshot {
    pub demo_mode: bool,
    pub paused: bool,
    pub effective_paused: bool,
    pub hud_mode: HudMode,
    pub controls_open: bool,
    pub help_open: bool,
    pub cpu_percent: f32,
    pub ram_percent: Option<f32>,
    pub speed_kmh: f64,
    pub world_elapsed_seconds: f64,
    pub scroll_px: f64,
    pub journey: JourneySnapshot,
    pub rendered_frames: u64,
    pub rendered_frames_hidden: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PanelTab {
    #[default]
    Drive,
    Journey,
    Appearance,
    Behavior,
    Shop,
    Collection,
}

// This factory is lazy: the demo branch cannot construct SysinfoProvider.
fn launch_worker(
    demo: bool,
    interval: Duration,
    cpu: f32,
    real_factory: impl FnOnce(Duration) -> io::Result<MetricsWorker>,
) -> io::Result<MetricsWorker> {
    if demo {
        MetricsWorker::spawn_with_provider(interval, ManualProvider::new(cpu))
    } else {
        real_factory(interval)
    }
}

pub struct CpuExpressApp {
    config: Config,
    config_path: Option<PathBuf>,
    preferences_dirty: bool,
    progress_path: Option<PathBuf>,
    progress_after: Instant,
    progress_notice: Option<String>,
    travel_log: std::collections::VecDeque<String>,
    save_after: Instant,
    notification: Option<String>,
    preferences_notice: Option<String>,
    show_controls: bool,
    welcome: bool,
    welcome_taskbar: bool,
    show_help: bool,
    panel_tab: PanelTab,
    menu: menus::MenuState,
    appearance_page: u8,
    journey_page: u8,
    selected_station: Option<crate::simulation::StationId>,
    coach_slot: usize,
    stock_gallery: Option<StockGallery>,
    equipment_notice: Option<(String, Instant)>,
    seen_rewards: (usize, usize),
    highlight_livery: Option<crate::simulation::objectives::Livery>,
    pending_dock: bool,
    applied_window_mode: Option<bool>,
    layout_test_step: Option<u8>,
    render_surface_size: [f32; 2],
    next_taskbar_check: Instant,
    next_strip_raise: Instant,
    next_panel_save: Instant,
    /// Frames painted since the panel was opened; zero while it is closed.
    panel_frames: u8,
    taskbar_failures: u8,
    strip_gesture: Option<platform::StripGesture>,
    slider_geometry: Option<[i32; 3]>,
    native_status: serde_json::Value,
    validation_captures: [bool; 2],
    #[cfg(windows)]
    desktop: Option<platform::DesktopController>,
    #[cfg(windows)]
    native_scale: Option<f32>,
    was_hidden: bool,
    focused: bool,
    paused: bool,
    demo_mode: bool,
    clock: SimulationClock,
    train: Train,
    world: World,
    renderer: SceneRenderer,
    scene_dirty: bool,
    local_weather: LocalWeather,
    weather_draft: crate::local_weather::WeatherLocation,
    travel_notice: Option<(String, Instant)>,
    texture: Option<egui::TextureHandle>,
    next_frame: Instant,
    opened: Instant,
    exit_after: Option<Duration>,
    cpu_override: Option<f32>,
    manual_value: f32,
    metrics: MetricsSample,
    worker: Option<MetricsWorker>,
    diagnostics: Option<Diagnostics>,
    next_diagnostic: Instant,
    samples: u64,
    closing: bool,
    frames: u64,
    frame_timings: crate::frame_timings::FrameTimings,
    audio: crate::audio::AudioPlayer,
    frames_hidden: u64,
}

impl CpuExpressApp {
    pub fn new(ctx: &egui::Context, options: LaunchOptions) -> Self {
        Self::create(ctx, options, false)
    }

    /// Starts a CPU-only manual provider without opening real CPU/RAM counters.
    pub fn new_demo(ctx: &egui::Context, options: LaunchOptions) -> Self {
        Self::create(ctx, options, true)
    }

    /// Prepare a bounded, deterministic populated demonstration for visual review.
    /// Never advances a real profile or writes files. Product time remains wall-clock driven.
    #[doc(hidden)]
    pub fn prepare_demo_review(
        &mut self,
        seconds: u32,
        excursion: Option<crate::simulation::BranchKind>,
    ) {
        if !self.demo_mode {
            return;
        }
        self.world.set_paused(false);
        for _ in 0..seconds.min(3600) {
            self.world
                .advance_expedition(&mut self.train, 50.0, 1.0, &self.config);
        }
        if let Some(kind) = excursion
            && self.world.request_excursion(kind).is_ok()
        {
            for _ in 0..600 {
                if self.world.excursion_snapshot().current.is_some() {
                    break;
                }
                self.world
                    .advance_expedition(&mut self.train, 50.0, 1.0, &self.config);
            }
            for _ in 0..75 {
                self.world
                    .advance_expedition(&mut self.train, 50.0, 1.0, &self.config);
            }
        }
        self.consume_travel_events();
        self.travel_notice = None;
        self.world.set_paused(self.suspended());
        self.scene_dirty = true;
    }

    /// Deterministic dispatch states for headless review; never touches a real profile.
    #[doc(hidden)]
    pub fn prepare_demo_contract_review(&mut self, returning: bool, incompatible: bool) {
        if !self.demo_mode || self.world.service_state().contract.is_some() {
            return;
        }
        self.world.set_automatic_contracts(false);
        if incompatible {
            let _ = self
                .world
                .set_service_composition([crate::simulation::ServiceWagon::Passenger; 3]);
        }
        self.world.set_paused(false);
        for _ in 0..600 {
            self.world
                .advance_expedition(&mut self.train, 50.0, 1.0, &self.config);
            if self.world.service_state().contract.is_some() {
                break;
            }
        }
        if returning && self.world.accept_cargo_contract().is_ok() {
            for _ in 0..10 {
                self.world
                    .advance_expedition(&mut self.train, 50.0, 1.0, &self.config);
            }
            let _ = self.world.cancel_cargo_contract();
        }
        self.consume_travel_events();
        self.travel_notice = None;
        self.world.set_paused(self.suspended());
        self.scene_dirty = true;
    }

    fn create(ctx: &egui::Context, options: LaunchOptions, demo_mode: bool) -> Self {
        ctx.set_theme(egui::Theme::Dark);
        let mut style = (*ctx.global_style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = PANEL;
        style.visuals.window_fill = PANEL;
        style.visuals.override_text_color = Some(INK);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(53, 103, 96);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, INK);
        style.visuals.widgets.inactive.bg_fill = CARD;
        style.visuals.widgets.inactive.weak_bg_fill = CARD;
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(46, 73, 74);
        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(46, 73, 74);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(57, 99, 92);
        style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(57, 99, 92);
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(5.0, 3.0);
        style.spacing.interact_size.y = 22.0;
        // Unlike the floating default, this leaves a visible scroll affordance
        // even before the mouse enters the compact options/help panel.
        style.spacing.scroll = egui::style::ScrollStyle::solid();
        ctx.set_global_style(style);
        let mut config = options.config.normalized();
        let welcome = !demo_mode && config::first_launch(options.config_path.as_deref());
        let welcome_taskbar = config.taskbar_mode;
        let manual_value = options.cpu_override.unwrap_or(50.0);
        let manual_value = if manual_value.is_finite() {
            manual_value.clamp(0.0, 100.0)
        } else {
            50.0
        };
        let cpu_override = if demo_mode || options.cpu_override.is_some() {
            Some(manual_value)
        } else {
            None
        };
        let mut metrics = if demo_mode {
            ManualProvider::new(manual_value).sample()
        } else {
            MetricsSample {
                cpu_percent: None,
                ram_percent: None,
                used_memory_bytes: 0,
                total_memory_bytes: 0,
                measured_at: Instant::now(),
                status: MetricStatus::WarmingUp,
            }
        };
        let worker = match launch_worker(
            demo_mode,
            Duration::from_millis(config.metrics_interval_ms),
            manual_value,
            MetricsWorker::spawn,
        ) {
            Ok(worker) => Some(worker),
            Err(error) => {
                metrics.status = MetricStatus::Unavailable(error.to_string());
                None
            }
        };
        let opened = Instant::now();
        let mut clock = SimulationClock::new(Duration::ZERO);
        clock.set_suspended(config.start_paused, Duration::ZERO);
        let mut world = World::default();
        let progress_path = if demo_mode || options.exit_after.is_some() {
            None
        } else {
            options
                .config_path
                .as_deref()
                .map(crate::progress::path_for)
        };
        let mut progress_notice = None;
        let mut recovery_notice = None;
        if let Some(path) = &progress_path {
            match crate::progress::load_profile(path) {
                Ok((saved_world, notice)) => {
                    if let Some(saved_world) = saved_world {
                        world = saved_world;
                    }
                    recovery_notice = notice;
                }
                Err(error) => progress_notice = Some(error),
            }
        }
        // The service transaction owns occupied capacity. A stale preference file must
        // never discard a loaded consignment or passengers after restart.
        if world
            .set_service_composition(config.visual.coaches.map(service_wagon))
            .is_err()
        {
            config.visual.coaches = world.service_snapshot().composition.map(coach_style);
        }
        if !demo_mode {
            use crate::rendering::EngineStyle;
            if !world
                .economy()
                .owns(&crate::simulation::economy::ItemId::Locomotive(
                    config.visual.engine.id().into(),
                ))
            {
                config.visual.engine = EngineStyle::Steam;
            }
        }
        let mut local_weather = LocalWeather::new(
            !std::env::var("CPU_EXPRESS_OFFLINE").is_ok_and(|v| v == "1")
                && ((!demo_mode && options.exit_after.is_none())
                    || std::env::var("CPU_EXPRESS_LIVE_WEATHER").is_ok_and(|v| v == "1")),
        );
        let weather_draft = config.weather_location.clone();
        if let Err(error) = local_weather.set_location(weather_draft.clone()) {
            local_weather.error = Some(error);
        }
        if !demo_mode
            && options.exit_after.is_none()
            && let Some(path) = &options.config_path
            && let Err(error) = local_weather.set_cache_path(path.with_extension("weather.json"))
        {
            local_weather.cache_error = Some(error);
        }
        let seen_rewards = (
            world.service_snapshot().owned_rewards.len(),
            world.service_snapshot().collection.len(),
        );
        world.set_paused(config.start_paused);
        Self {
            train: Train::new(&config),
            paused: config.start_paused,
            config,
            config_path: options.config_path,
            preferences_dirty: false,
            progress_path,
            progress_after: opened + Duration::from_secs(30),
            progress_notice,
            travel_log: std::collections::VecDeque::with_capacity(32),
            save_after: opened,
            notification: match (options.warning, recovery_notice) {
                (Some(warning), Some(recovery)) => Some(format!("{warning}\n{recovery}")),
                (warning, recovery) => warning.or(recovery),
            },
            preferences_notice: None,
            show_controls: welcome || std::env::var_os("CPU_EXPRESS_OPEN_PANEL").is_some(),
            welcome,
            welcome_taskbar,
            show_help: false,
            panel_tab: PanelTab::default(),
            menu: menus::MenuState::default(),
            appearance_page: 0,
            journey_page: 0,
            selected_station: None,
            coach_slot: 0,
            stock_gallery: None,
            equipment_notice: None,
            seen_rewards,
            highlight_livery: None,
            pending_dock: true,
            applied_window_mode: None,
            layout_test_step: (options.exit_after.is_some()
                && std::env::var_os("CPU_EXPRESS_LAYOUT_TEST").is_some())
            .then_some(0),
            render_surface_size: [0.0; 2],
            next_taskbar_check: opened,
            next_strip_raise: opened,
            next_panel_save: opened + Duration::from_secs(2),
            panel_frames: 0,
            taskbar_failures: 0,
            strip_gesture: None,
            slider_geometry: None,
            native_status: serde_json::Value::Null,
            validation_captures: [false; 2],
            #[cfg(windows)]
            desktop: None,
            #[cfg(windows)]
            native_scale: None,
            was_hidden: false,
            focused: false,
            demo_mode,
            clock,
            world,
            renderer: SceneRenderer::new(),
            scene_dirty: true,
            local_weather,
            weather_draft,
            travel_notice: None,
            texture: None,
            next_frame: opened,
            opened,
            exit_after: options.exit_after,
            cpu_override,
            manual_value,
            metrics,
            worker,
            diagnostics: options.diagnostics,
            next_diagnostic: opened,
            samples: 0,
            closing: false,
            frames: 0,
            frame_timings: Default::default(),
            audio: crate::audio::AudioPlayer::new(),
            frames_hidden: 0,
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            demo_mode: self.demo_mode,
            paused: self.paused,
            effective_paused: self.suspended(),
            hud_mode: self.config.effective_hud_mode(),
            controls_open: self.show_controls,
            help_open: self.show_help,
            cpu_percent: self.cpu_percent(),
            ram_percent: self.ram_percent(),
            speed_kmh: self.train.speed_kmh(),
            world_elapsed_seconds: self.world.elapsed_seconds(),
            scroll_px: self.world.scroll_px(),
            journey: self.world.journey_snapshot(),
            rendered_frames: self.frames,
            rendered_frames_hidden: self.frames_hidden,
        }
    }

    /// Buttons and shortcuts share these commands; native actions use one owner.
    pub fn dispatch(&mut self, action: AppAction, ctx: &egui::Context) {
        match action {
            AppAction::TogglePause => {
                self.paused = !self.paused;
                self.sync_clock();
                self.next_frame = Instant::now();
            }
            AppAction::CycleHud => {
                self.config
                    .set_hud_mode(self.config.effective_hud_mode().next());
                self.preferences_changed();
            }
            AppAction::TogglePin => {
                #[cfg(windows)]
                if let Some(desktop) = &mut self.desktop {
                    let result = desktop.set_topmost(!desktop.always_on_top());
                    self.note_result(result);
                    ctx.request_repaint();
                    return;
                }
                self.config.always_on_top = !self.config.always_on_top;
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    if self.config.always_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    },
                ));
                self.preferences_changed();
            }
            AppAction::ToggleControls => {
                self.show_controls = !self.show_controls;
                self.show_help = false;
            }
            AppAction::ToggleHelp => {
                self.show_help = !self.show_help;
                self.show_controls = false;
            }
            AppAction::Reposition => self.pending_dock = true,
            AppAction::Minimize => {
                #[cfg(windows)]
                if let Some(desktop) = &mut self.desktop {
                    let result = desktop.minimize();
                    self.note_result(result);
                    ctx.request_repaint();
                    return;
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            AppAction::HideToTray =>
            {
                #[cfg(windows)]
                if let Some(desktop) = &mut self.desktop {
                    let result = desktop.hide();
                    self.note_result(result);
                }
            }
            AppAction::ClosePanel => {
                self.show_controls = false;
                self.show_help = false;
            }
            AppAction::Quit => {
                #[cfg(windows)]
                if let Some(desktop) = &mut self.desktop {
                    let result = desktop.close();
                    self.note_result(result);
                    ctx.request_repaint();
                    return;
                }
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        ctx.request_repaint();
    }

    fn suspended(&self) -> bool {
        self.paused || self.was_hidden || (self.config.pause_when_unfocused && !self.focused)
    }

    fn sync_clock(&mut self) {
        self.world.set_paused(self.suspended());
        self.clock
            .set_suspended(self.suspended(), self.opened.elapsed());
    }

    fn metrics_stale(&self) -> bool {
        let limit = self.worker.as_ref().map_or_else(
            || Duration::from_millis(self.config.metrics_interval_ms.saturating_mul(3).max(3_000)),
            MetricsWorker::stale_after,
        );
        self.metrics.freshness(limit) == SampleFreshness::Stale
    }

    fn add_notice(&mut self, message: String) {
        self.notification = Some(match self.notification.take() {
            Some(previous) if !previous.contains(&message) => format!("{previous}\n{message}"),
            Some(previous) => previous,
            None => message,
        });
    }

    fn note_result(&mut self, result: Result<(), String>) {
        if let Err(error) = result {
            self.add_notice(error);
        }
    }

    fn tray_available(&self) -> bool {
        #[cfg(windows)]
        {
            self.desktop
                .as_ref()
                .is_some_and(platform::DesktopController::tray_available)
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    fn cpu_percent(&self) -> f32 {
        let cpu = self.cpu_override.unwrap_or_else(|| {
            if self.metrics_stale() || matches!(self.metrics.status, MetricStatus::WarmingUp) {
                0.0
            } else {
                self.metrics.cpu_percent.unwrap_or(0.0)
            }
        });
        if cpu.is_finite() {
            cpu.clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    fn ram_percent(&self) -> Option<f32> {
        if self.demo_mode || self.metrics_stale() || self.metrics.memory_bytes().is_none() {
            return None;
        }
        self.metrics
            .ram_percent
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 100.0))
    }

    fn telemetry_notice(&self) -> Option<&'static str> {
        if self.demo_mode {
            return None;
        }
        if self.metrics_stale() {
            Some("Leitura atrasada")
        } else if matches!(self.metrics.status, MetricStatus::WarmingUp) {
            Some("Lendo CPU…")
        } else if self.metrics.cpu_percent.is_none_or(|cpu| !cpu.is_finite()) {
            if self.ram_percent().is_some() {
                Some("CPU indisponível")
            } else {
                Some("Sem métricas")
            }
        } else if matches!(self.metrics.status, MetricStatus::Unavailable(_)) {
            Some("Leitura parcial")
        } else {
            None
        }
    }

    fn cpu_label(&self) -> String {
        if self.demo_mode {
            format!("DEMO · CPU {:.1}%", self.cpu_percent())
        } else if self.cpu_override.is_some() {
            format!("SIMULADO · CPU {:.1}%", self.cpu_percent())
        } else if !self.metrics_stale()
            && !matches!(self.metrics.status, MetricStatus::WarmingUp)
            && self.metrics.cpu_percent.is_some_and(f32::is_finite)
        {
            format!("CPU {:.1}%", self.cpu_percent())
        } else {
            self.telemetry_notice()
                .unwrap_or("CPU indisponível")
                .to_owned()
        }
    }

    fn record_diagnostics(&mut self, final_record: bool) {
        if self.diagnostics.is_none() || (!final_record && Instant::now() < self.next_diagnostic) {
            return;
        }
        let status = match &self.metrics.status {
            MetricStatus::WarmingUp => "warming_up",
            MetricStatus::Ready => "ready",
            MetricStatus::Unavailable(_) => "unavailable",
        };
        let mode = if self.demo_mode {
            "demo"
        } else if self.cpu_override.is_some() {
            "manual"
        } else {
            "real"
        };
        let journey = self.world.journey_snapshot();
        let record = serde_json::json!({
            "event": if final_record { "shutdown" } else { "sample" },
            "elapsed_seconds": self.opened.elapsed().as_secs_f64(),
            "mode": mode,
            "local_weather": self.local_weather.diagnostics(),
            "frame_work": self.frame_timings.diagnostics(),
            "cpu_percent": if self.demo_mode { self.cpu_override } else { self.metrics.cpu_percent },
            "effective_cpu_percent": self.cpu_percent(), "ram_percent": self.ram_percent(),
            "used_memory_bytes": self.metrics.used_memory_bytes, "total_memory_bytes": self.metrics.total_memory_bytes,
            "metric_status": status, "metric_age_seconds": self.metrics.measured_at.elapsed().as_secs_f64(),
            "metrics_stale": self.metrics_stale(), "target_kmh": self.train.target_speed_kmh(),
            "speed_kmh": self.train.speed_kmh(), "rendered_frames": self.frames,
            "rendered_frames_visible": self.frames.saturating_sub(self.frames_hidden),
            "rendered_frames_hidden": self.frames_hidden,
            "received_samples": self.samples, "smoke_particles": self.world.smoke().len(), "hidden": self.was_hidden,
            "paused": self.paused, "effective_paused": self.suspended(),
            "world_elapsed_seconds": self.world.elapsed_seconds(), "scroll_px": self.world.scroll_px(),
            "journey_distance_units": journey.total_distance_units,
            "journey_active_seconds": journey.active_seconds,
            "journey_segment": journey.route_segment.index,
            "next_station": self.destination_name(),
            "segment_progress": journey.segment_progress,
            "completed_laps": journey.completed_laps,
            "stations_visited": journey.stations_visited,
            "dropped_travel_events": journey.dropped_events,
            "native_window": self.native_status,
            "requested_taskbar_mode": self.config.taskbar_mode,
            "render_surface_size": self.render_surface_size,
            "station_wait_seconds": self.world.station_wait(),
            "services": self.world.service_snapshot(),
        });
        if let Some(diagnostics) = &mut self.diagnostics
            && let Err(error) = diagnostics.record(&record)
        {
            eprintln!("diagnostics disabled: {error}");
            self.notification = Some(format!("Não foi possível gravar o diagnóstico: {error}"));
            self.diagnostics = None;
        }
        self.next_diagnostic = Instant::now() + Duration::from_secs(1);
    }

    fn step_frame(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        if self.was_hidden
            || (now + Duration::from_millis(1) < self.next_frame
                && self.texture.is_some()
                && !self.scene_dirty)
        {
            return;
        }
        self.sync_clock();
        let dt = self.clock.take_step(self.opened.elapsed());
        if !self.suspended() {
            let cpu = self.cpu_percent();
            if let Some(path) = &self.progress_path {
                match crate::progress::advance_checkpoint(
                    path,
                    &mut self.world,
                    &mut self.train,
                    cpu,
                    dt,
                    &self.config,
                ) {
                    Ok(Some(warning)) => self.progress_notice = Some(warning),
                    Ok(None) => {}
                    Err(error) => {
                        self.progress_notice = Some(error.clone());
                        self.notification = Some(format!("Viagem pausada: {error}"));
                        self.paused = true;
                        self.world.set_paused(true);
                    }
                }
            } else {
                self.world
                    .advance_expedition(&mut self.train, cpu, dt, &self.config);
            }
            self.consume_travel_events();
        }
        // Pausing freezes particles and wheels and reuses the existing GPU texture.
        // An explicit visual edit can redraw once without advancing the journey.
        if !self.suspended() || self.texture.is_none() || self.scene_dirty {
            if self.scene_dirty {
                self.renderer.set_theme(self.config.visual.theme);
                self.renderer.set_quality(self.config.visual.quality);
                self.renderer.set_options(self.config.visual.scene);
                self.renderer.set_coaches(self.config.visual.coaches);
                self.renderer.set_engine(self.config.visual.engine);
            }
            self.renderer
                .set_scene_context(Some(SceneContext::from_journey(
                    &self.world.journey_snapshot(),
                )));
            self.update_scenery_adapter();
            let livery = self.config.visual.livery;
            self.renderer.set_livery(
                if self.world.service_snapshot().owns(livery) || self.demo_mode {
                    livery
                } else {
                    crate::simulation::objectives::Livery::Aurora
                },
            );
            self.renderer
                .set_local_atmosphere(self.local_weather.atmosphere());
            self.renderer
                .set_local_lighting(self.local_weather.lighting_phase());
            self.renderer.set_weather_detail(
                self.local_weather
                    .last_good
                    .as_ref()
                    .and_then(|s| s.wind_speed_kmh),
                self.local_weather
                    .last_good
                    .as_ref()
                    .and_then(|s| s.precipitation_mm),
            );
            let arrival = self.world.service_snapshot().arrivals.last();
            self.renderer
                .set_station_service(Some(crate::rendering::StationService {
                    progress: (1.0 - self.world.station_wait() / 8.0) as f32,
                    boarding: arrival.map_or(0, |a| a.boarded),
                    alighting: arrival.map_or(0, |a| a.alighted),
                }));
            let image = egui::ColorImage::new(
                [self.renderer.width(), HEIGHT],
                self.renderer.render(&self.train, &self.world).to_vec(),
            );
            if let Some(texture) = &mut self.texture {
                texture.set(image, egui::TextureOptions::NEAREST);
            } else {
                self.texture = Some(ctx.load_texture(
                    "original-train-scene",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            }
            self.frame_timings.record(now.elapsed());
            self.frames += 1;
            if self.was_hidden {
                self.frames_hidden += 1;
            }
            self.scene_dirty = false;
        }
        let interval = if self.suspended() {
            Duration::from_millis(250)
        } else {
            Duration::from_secs_f64(1.0 / f64::from(self.config.fps))
        };
        self.next_frame = crate::frame_pacing::next_deadline(self.next_frame, now, interval);
    }

    fn preferences_changed(&mut self) {
        self.scene_dirty = true;
        self.preferences_dirty = true;
        self.save_after = Instant::now() + Duration::from_millis(400);
    }

    fn play_cue(&mut self, cue: crate::audio::AudioCue) {
        self.audio.play(
            cue,
            if self.config.quiet_mode {
                0.0
            } else {
                self.config.sound_volume
            },
        );
    }

    fn consume_travel_events(&mut self) {
        let mut messages = Vec::new();
        let mut reward = false;
        let state = self.world.service_snapshot();
        for livery in state.owned_rewards.iter().skip(self.seen_rewards.0) {
            messages.push(format!("Pintura conquistada: {}", livery.name()));
            reward = true;
        }
        for item in state.collection.iter().skip(self.seen_rewards.1) {
            messages.push(
                match item {
                    crate::simulation::service::ServiceReward::PassengerPostcard => {
                        "Postal conquistado · Elétrica da Serra liberada"
                    }
                    crate::simulation::service::ServiceReward::CargoPostcard => {
                        "Postal conquistado · Diesel do Porto liberada"
                    }
                }
                .to_owned(),
            );
            reward = true;
        }
        self.seen_rewards = (state.owned_rewards.len(), state.collection.len());

        while let Some(event) = self.world.pop_travel_event() {
            let message = match event {
                TravelEvent::StationArrived { station, .. } => {
                    let arrival = self.world.service_snapshot().arrivals.last();
                    Some(arrival.map_or_else(
                        || format!("Chegada: {}", station.name()),
                        |a| {
                            format!(
                                "{} · {} embarcaram · {} desembarcaram · +{} pontos",
                                station.name(),
                                a.boarded,
                                a.alighted,
                                a.earned
                            )
                        },
                    ))
                }
                TravelEvent::CircuitCompleted { completed_laps, .. } => {
                    Some(format!("Circuito {completed_laps} concluído"))
                }
                TravelEvent::AchievementUnlocked { achievement, .. } => {
                    reward = true;
                    Some(format!("Conquista: {}", achievement_name(achievement)))
                }
                TravelEvent::StationDeparted { .. } => None,
            };
            if let Some(message) = message {
                if self.travel_log.len() == 32 {
                    self.travel_log.pop_front();
                }
                self.travel_log.push_back(message.clone());
                messages.push(message);
            }
        }
        if !messages.is_empty() {
            self.play_cue(if reward {
                crate::audio::AudioCue::Reward
            } else {
                crate::audio::AudioCue::Arrival
            });
            self.travel_notice = Some((
                messages.join(" · "),
                Instant::now() + Duration::from_secs(7),
            ));
        }
    }

    fn persist_preferences(&mut self, force: bool) {
        if self.welcome {
            return;
        }
        if !self.preferences_dirty || (!force && Instant::now() < self.save_after) {
            return;
        }
        let result = self
            .config_path
            .as_deref()
            .map(|path| config::save(&self.config, path));
        self.preferences_dirty = matches!(&result, Some(Err(_)));
        self.save_after = Instant::now() + Duration::from_secs(5);
        self.preferences_notice = match result {
            Some(Ok(())) => None,
            Some(Err(error)) => Some(format!("Alterações não salvas. {error}")),
            None => Some(
                "Preferências só nesta sessão: caminho de configuração indisponível.".to_owned(),
            ),
        };
        // A failed write generates one recoverable message, never a per-frame retry.
    }

    fn persist_progress(&mut self, force: bool) {
        if !force && Instant::now() < self.progress_after {
            return;
        }
        self.progress_after = Instant::now() + Duration::from_secs(30);
        if let Some(path) = &self.progress_path {
            // The notice carries either a failed save or a stale recovery copy.
            self.progress_notice = match crate::progress::save_checkpoint(path, &self.world) {
                Ok(warning) => warning,
                Err(error) => Some(error),
            };
        }
    }

    fn notification_message(&self) -> Option<String> {
        match (&self.notification, &self.preferences_notice) {
            (Some(general), Some(preferences)) => Some(format!("{general}\n{preferences}")),
            (Some(message), None) | (None, Some(message)) => Some(message.clone()),
            (None, None) => None,
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let in_use = ctx.egui_wants_keyboard_input();
        let panel_open = self.show_controls || self.show_help;
        let events = ctx.input(|input| input.events.clone());
        for event in events {
            if let egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
                && let Some(action) = shortcut_action(key, modifiers, in_use, panel_open)
            {
                ctx.input_mut(|input| {
                    input.consume_key(modifiers, key);
                });
                self.dispatch(action, ctx);
                break;
            }
        }
    }

    fn toolbar_visible(&self, ui: &egui::Ui, bounds: egui::Rect) -> bool {
        !self.config.auto_hide_controls
            || self.focused
            || self.show_controls
            || self.show_help
            || ui.input(|input| {
                input
                    .pointer
                    .hover_pos()
                    .is_some_and(|pos| bounds.contains(pos))
            })
    }

    fn badge(ui: &mut egui::Ui, rect: egui::Rect, text: &str, warning: bool, hint: &str) {
        let fill = if warning {
            egui::Color32::from_rgb(92, 62, 30)
        } else {
            PANEL
        };
        ui.painter().rect_filled(rect, 4.0, fill);
        ui.put(
            rect,
            egui::Label::new(egui::RichText::new(text).size(10.5).color(INK)),
        )
        .on_hover_text(hint);
    }

    fn header(&mut self, ui: &mut egui::Ui, bounds: egui::Rect, visible: bool) {
        let narrow = bounds.width() < 600.0;
        if visible {
            let strip =
                egui::Rect::from_min_size(bounds.left_top(), egui::vec2(bounds.width(), 32.0));
            ui.painter().rect_filled(strip, 0.0, PANEL);
            if !narrow {
                ui.painter().text(
                    bounds.left_top() + egui::vec2(10.0, 16.0),
                    egui::Align2::LEFT_CENTER,
                    "CPU EXPRESS",
                    egui::FontId::monospace(11.0),
                    INK,
                );
            }
        }
        let mut x = if visible && !narrow { 112.0 } else { 8.0 };
        if self.demo_mode || self.cpu_override.is_some() {
            let text = if self.demo_mode {
                "DEMO"
            } else if narrow {
                "SIM"
            } else {
                "SIMULADO"
            };
            let width = if self.demo_mode {
                47.0
            } else if narrow {
                38.0
            } else {
                70.0
            };
            Self::badge(
                ui,
                egui::Rect::from_min_size(
                    bounds.left_top() + egui::vec2(x, 6.0),
                    egui::vec2(width, 21.0),
                ),
                text,
                true,
                if self.demo_mode {
                    "CPU simulada. Nenhum contador real de CPU ou RAM é consultado."
                } else {
                    "O trem usa a CPU manual. A RAM e a coleta em segundo plano continuam reais."
                },
            );
            x += width + 5.0;
        }
        if self.suspended() {
            Self::badge(
                ui,
                egui::Rect::from_min_size(
                    bounds.left_top() + egui::vec2(x, 6.0),
                    egui::vec2(58.0, 21.0),
                ),
                "PAUSADO",
                false,
                if self.paused {
                    "Viagem pausada. Use Espaço ou Seguir para retomar."
                } else {
                    "Pausa automática: a janela está sem foco ou minimizada."
                },
            );
            x += 63.0;
        }
        if let Some(notice) = self.telemetry_notice() {
            // A separate row avoids collisions when multiple notices are visible.
            Self::badge(
                ui,
                egui::Rect::from_min_size(
                    bounds.left_top() + egui::vec2(8.0, 35.0),
                    egui::vec2(119.0, 21.0),
                ),
                notice,
                true,
                "A janela continua funcionando. A coleta tenta se recuperar; dados antigos não aparecem como leitura atual.",
            );
        }
        if let Some(message) = self.notification_message() {
            let rect = egui::Rect::from_min_size(
                bounds.left_top() + egui::vec2(x.min(285.0), 6.0),
                egui::vec2(24.0, 21.0),
            );
            if ui
                .put(
                    rect,
                    egui::Button::new("!").fill(egui::Color32::from_rgb(92, 62, 30)),
                )
                .on_hover_text(format!("{message}\nClique para dispensar este aviso."))
                .clicked()
            {
                self.notification = None;
                self.preferences_notice = None;
            }
        }
        if !visible || self.show_controls || self.show_help {
            return;
        }
        let mut left = bounds.right() - if narrow { 177.0 } else { 303.0 };
        let mut button =
            |ui: &mut egui::Ui, label: &str, width: f32, selected: bool, hint: &str| {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(left, bounds.top() + 4.0),
                    egui::vec2(width, 24.0),
                );
                left += width + 4.0;
                ui.put(rect, egui::Button::new(label).selected(selected))
                    .on_hover_text(hint)
                    .clicked()
            };
        let mut action = None;
        if button(
            ui,
            if self.paused { "Seguir" } else { "Pausar" },
            58.0,
            self.paused,
            "Pausar/retomar a viagem (Espaço)",
        ) {
            action = Some(AppAction::TogglePause);
        }
        if narrow {
            if button(
                ui,
                "Opções",
                59.0,
                false,
                "Controles, viagem, visual e janela (O)",
            ) {
                action = Some(AppAction::ToggleControls);
            }
            if button(ui, "Sair", 40.0, false, "Encerrar CPU Express (Ctrl+Q)") {
                action = Some(AppAction::Quit);
            }
            if let Some(action) = action {
                self.dispatch(action, ui.ctx());
            }
            return;
        }
        if button(
            ui,
            "HUD",
            38.0,
            false,
            "Alternar compacto, completo e limpo (M)",
        ) {
            action = Some(AppAction::CycleHud);
        }
        if button(ui, "Opções", 59.0, false, "CPU simulada e preferências (O)") {
            action = Some(AppAction::ToggleControls);
        }
        if button(ui, "?", 24.0, false, "Ajuda e atalhos (F1)") {
            action = Some(AppAction::ToggleHelp);
        }
        if button(
            ui,
            "P",
            24.0,
            self.config.always_on_top,
            "Sempre no topo (P)",
        ) {
            action = Some(AppAction::TogglePin);
        }
        if button(
            ui,
            "−",
            24.0,
            false,
            "Minimizar; restaure pela barra de tarefas",
        ) {
            action = Some(AppAction::Minimize);
        }
        if button(ui, "Sair", 40.0, false, "Encerrar CPU Express (Ctrl+Q)") {
            action = Some(AppAction::Quit);
        }
        if let Some(action) = action {
            self.dispatch(action, ui.ctx());
        }
    }

    fn status_bar(&self, ui: &mut egui::Ui, bounds: egui::Rect) {
        let mode = self.config.effective_hud_mode();
        if mode == HudMode::Clean {
            return;
        }
        // Keep the whole status readable to the left of the overlay. A detailed
        // HUD temporarily uses the compact layout while the panel is open.
        let overlay_open = self.show_controls || self.show_help;
        let detailed = mode == HudMode::Detailed && !overlay_open && bounds.width() >= 600.0;
        let available = if overlay_open {
            bounds.width() - 373.0_f32.min(bounds.width())
        } else {
            bounds.width()
        };
        if available < 250.0 {
            return;
        }
        let width = if detailed {
            bounds.width()
        } else {
            300.0_f32.min(available)
        };
        let strip = egui::Rect::from_min_size(
            bounds.left_bottom() - egui::vec2(0.0, 24.0),
            egui::vec2(width, 24.0),
        );
        ui.painter().rect_filled(strip, 0.0, PANEL);
        ui.painter().text(
            strip.left_center() + egui::vec2(10.0, 0.0),
            egui::Align2::LEFT_CENTER,
            self.cpu_label(),
            egui::FontId::monospace(11.0),
            INK,
        );
        if detailed {
            let ram = if self.demo_mode {
                "RAM indisponível no demo".to_owned()
            } else {
                self.ram_percent()
                    .map_or("RAM --".to_owned(), |ram| format!("RAM {ram:.1}%"))
            };
            ui.painter().text(
                strip.left_center() + egui::vec2(245.0, 0.0),
                egui::Align2::LEFT_CENTER,
                ram,
                egui::FontId::monospace(11.0),
                INK,
            );
        }
        let speed = if self.suspended() {
            "Pausado".to_owned()
        } else {
            format!("{:.1} km/h", self.train.speed_kmh())
        };
        ui.painter().text(
            strip.right_center() - egui::vec2(10.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            speed,
            egui::FontId::monospace(11.0),
            ACCENT,
        );
        let mut details = if self.demo_mode {
            "Demonstração sem consultar CPU/RAM reais. A RAM não é simulada.".to_owned()
        } else if self.ram_percent().is_some() && self.metrics.total_memory_bytes > 0 {
            format!(
                "RAM: {:.2} / {:.2} GiB",
                self.metrics.used_memory_bytes as f64 / 1_073_741_824.0,
                self.metrics.total_memory_bytes as f64 / 1_073_741_824.0
            )
        } else {
            "Sem leitura recente de RAM.".to_owned()
        };
        details.push_str("\nA velocidade é fictícia. Simular CPU não gera carga artificial.");
        if let MetricStatus::Unavailable(error) = &self.metrics.status {
            details.push_str(&format!("\nColeta: {error}"));
        }
        ui.interact(
            strip,
            egui::Id::unique("metrics-details"),
            egui::Sense::HOVER,
        )
        .on_hover_text(details);
    }

    fn journey_indicator(&self, ui: &mut egui::Ui, bounds: egui::Rect) {
        if self.show_controls || self.show_help {
            return;
        }
        let journey = self.world.journey_snapshot();
        if self.config.effective_hud_mode() == HudMode::Compact && bounds.width() >= 550.0 {
            let rect = egui::Rect::from_min_max(
                egui::pos2(bounds.left() + 306.0, bounds.bottom() - 24.0),
                bounds.right_bottom(),
            );
            ui.painter().rect_filled(rect, 0.0, PANEL);
            ui.painter().text(
                rect.left_center() + egui::vec2(8.0, -2.0),
                egui::Align2::LEFT_CENTER,
                format!(
                    "Para {} · {:.0}%",
                    self.destination_name(),
                    f64::from(self.travel_progress()) * 100.0
                ),
                egui::FontId::proportional(11.0),
                INK,
            );
            let track = egui::Rect::from_min_size(
                rect.left_bottom() + egui::vec2(8.0, -4.0),
                egui::vec2(rect.width() - 16.0, 2.0),
            );
            ui.painter()
                .rect_filled(track, 0.0, egui::Color32::from_rgb(60, 79, 70));
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    track.min,
                    egui::vec2(track.width() * self.travel_progress(), 2.0),
                ),
                0.0,
                ACCENT,
            );
            ui.interact(rect, egui::Id::unique("journey-progress"), egui::Sense::hover())
                .on_hover_text(format!("{} até {}\nFaltam {:.0} unidades fictícias.\n{} visitas · {} circuitos. Abra Opções / Viagem para detalhes.",
                    journey.route_segment.from.name(), self.destination_name(), journey.distance_to_next_station_units, journey.stations_visited, journey.completed_laps));
        }
        if let Some((message, until)) = &self.travel_notice
            && Instant::now() < *until
            && !self.config.quiet_mode
        {
            let width = 300.0_f32.min(bounds.width() - 16.0);
            let rect = egui::Rect::from_min_size(
                egui::pos2(bounds.right() - width - 8.0, bounds.top() + 35.0),
                egui::vec2(width, 21.0),
            );
            Self::badge(
                ui,
                rect,
                message,
                false,
                "Abra Viagem para acompanhar objetivos e salvamento.",
            );
        }
    }

    fn scene_preview(&self, ui: &mut egui::Ui, compact: bool) {
        let ppp = ui.ctx().pixels_per_point();
        let scale = (ui.available_width() * ppp / WIDTH as f32)
            .floor()
            .clamp(1.0, if compact { 1.0 } else { 3.0 });
        let preview_width = self.renderer.width().min(WIDTH) as f32;
        let size = egui::vec2(preview_width, HEIGHT as f32) * scale / ppp;
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), size.y + 16.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, game::EDGE);
        painter.rect_stroke(
            rect.shrink(2.0),
            2.0,
            egui::Stroke::new(2.0, game::GOLD),
            egui::StrokeKind::Inside,
        );
        if let Some(texture) = &self.texture {
            let minimum = (rect.center() - size * 0.5) * ppp;
            let image = egui::Rect::from_min_size(
                egui::pos2(minimum.x.round(), minimum.y.round()) / ppp,
                size,
            );
            painter.image(
                texture.id(),
                image,
                egui::Rect::from_min_max(
                    egui::Pos2::ZERO,
                    egui::pos2(preview_width / self.renderer.width() as f32, 1.0),
                ),
                egui::Color32::WHITE,
            );
        }
        ui.add_space(8.0);
    }

    fn compact_drive_controls(&mut self, ui: &mut egui::Ui) {
        let metric_color = if ui.visuals().dark_mode {
            ACCENT
        } else {
            game::MUTED
        };
        if self.demo_mode {
            ui.label(
                egui::RichText::new("DEMO · CPU simulada; RAM indisponível")
                    .size(11.0)
                    .color(metric_color),
            );
        } else {
            let mut manual = self.cpu_override.is_some();
            if ui
                .checkbox(&mut manual, "Simular CPU, sem gerar carga")
                .changed()
            {
                self.cpu_override = manual.then_some(self.manual_value);
            }
        }
        let manual = self.cpu_override.is_some();
        ui.add_enabled_ui(manual, |ui| {
            ui.horizontal(|ui| {
                ui.label("CPU:");
                for value in [0.0, 50.0, 100.0] {
                    if ui
                        .selectable_label(
                            (self.manual_value - value).abs() < 0.05,
                            format!("{value:.0}%"),
                        )
                        .clicked()
                    {
                        self.manual_value = value;
                        self.cpu_override = Some(value);
                    }
                }
            });
            if ui
                .add(
                    egui::Slider::new(&mut self.manual_value, 0.0..=100.0)
                        .suffix(" %")
                        .fixed_decimals(1)
                        .step_by(0.1),
                )
                .changed()
            {
                self.cpu_override = Some(self.manual_value);
            }
        });
        ui.horizontal(|ui| {
            if ui
                .button(if self.paused {
                    "Seguir (Espaço)"
                } else {
                    "Pausar (Espaço)"
                })
                .clicked()
            {
                self.dispatch(AppAction::TogglePause, ui.ctx());
            }
            if ui.button("Reposicionar (R)").clicked() {
                self.dispatch(AppAction::Reposition, ui.ctx());
            }
        });
        if let Some(notice) = self.telemetry_notice() {
            ui.label(egui::RichText::new(notice).color(metric_color));
            if let MetricStatus::Unavailable(error) = &self.metrics.status {
                ui.label(egui::RichText::new(error).size(11.0));
            }
        }
        ui.label(
            egui::RichText::new("0 / 50 / 100%: alvos 12 / 56 / 100 km/h nos padrões.").size(10.5),
        );
    }

    fn drive_controls(&mut self, ui: &mut egui::Ui) {
        // Embedded backends have only the tiny scene window for their panel.
        if ui.ctx().embed_viewports() {
            self.compact_drive_controls(ui);
            return;
        }
        game::apply_style(ui);
        let journey = self.world.journey_snapshot();
        if self.world.excursion_snapshot().current.is_some() {
            self.branch_controls(ui);
        }
        // The train first, then where it is going and the one action of this screen.
        self.scene_preview(ui, false);
        ui.add_space(6.0);
        let art = |ui: &mut egui::Ui, sprite: game::Sprite, size: f32| {
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
            game::sprite(ui.painter(), rect, sprite);
        };
        let waiting = self.world.station_wait();
        let seconds = if waiting > 0.0 {
            waiting.ceil()
        } else {
            (journey.distance_to_next_station_units / 24.0).ceil()
        };
        // Active seconds only: the clock stands still while paused.
        let situation = if self.world.excursion_snapshot().current.is_some() {
            "Em excursão pelo ramal".to_owned()
        } else if self.paused {
            format!("Pausado · faltam {seconds:.0} s")
        } else if waiting > 0.0 {
            format!("Na plataforma · partida em {seconds:.0} s")
        } else {
            format!("Chegada em {seconds:.0} s")
        };
        let mut toggle = false;
        egui::Frame::new()
            .fill(game::CARD)
            .stroke(egui::Stroke::new(2.0, game::WOOD_DARK))
            .corner_radius(4)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (tile, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(56.0), egui::Sense::hover());
                    ui.painter().rect_filled(tile, 4, game::PAPER);
                    ui.painter().rect_stroke(
                        tile,
                        4,
                        egui::Stroke::new(2.0, game::WOOD),
                        egui::StrokeKind::Inside,
                    );
                    game::sprite(ui.painter(), tile.shrink(4.0), game::Sprite::Estacao);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(
                            egui::RichText::new("Próxima parada")
                                .size(12.0)
                                .color(game::MUTED),
                        );
                        ui.label(
                            egui::RichText::new(self.destination_name())
                                .size(20.0)
                                .strong()
                                .color(game::INK),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        toggle = game::button(
                            ui,
                            "drive-toggle",
                            if self.paused { Icon::Play } else { Icon::Pause },
                            if self.paused { "Retomar" } else { "Pausar" },
                            true,
                            egui::vec2(170.0, 48.0),
                        )
                        .on_hover_text("Espaço também pausa e retoma. O progresso fica preservado.")
                        .clicked();
                    });
                });
                ui.add_space(4.0);
                game::progress(
                    ui,
                    "next-stop",
                    self.travel_progress(),
                    &situation,
                    game::GREEN,
                );
            });
        if toggle {
            self.dispatch(AppAction::TogglePause, ui.ctx());
        }
        let speed = if self.suspended() {
            "0 km/h".to_owned()
        } else {
            format!("{:.0} km/h", self.train.speed_kmh())
        };
        let cpu = if self.demo_mode
            || self.cpu_override.is_some()
            || (self.metrics.cpu_percent.is_some() && !self.metrics_stale())
        {
            format!("{:.0}%", self.cpu_percent())
        } else {
            "-".to_owned()
        };
        let ram = self
            .ram_percent()
            .map_or_else(|| "-".to_owned(), |value| format!("{value:.0}%"));
        // One line of gauges instead of three cards.
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (sprite, value, name) in [
                (game::Sprite::Locomotiva, speed, "Velocidade do trem"),
                (
                    game::Sprite::Cpu,
                    cpu,
                    if self.demo_mode {
                        "Processador (simulado na demonstração)"
                    } else {
                        "Uso do processador: muda o ritmo visual, não a viagem"
                    },
                ),
                (game::Sprite::Memoria, ram, "Uso de memória"),
            ] {
                art(ui, sprite, 32.0);
                game::pixel_label(ui, &value, 2.0, game::INK).on_hover_text(name);
                ui.add_space(14.0);
            }
        });
        ui.add_space(6.0);
        // Shortcut to services carrying the real occupancy.
        let service = self.world.service_snapshot();
        let (seats, hold) = service.capacity();
        let (aboard, cargo) = (service.passenger_count(), service.cargo_count());
        let mut open_services = false;
        ui.horizontal(|ui| {
            art(ui, game::Sprite::Passageiro, 32.0);
            game::pixel_label(ui, &format!("{aboard}/{seats}"), 2.0, game::INK)
                .on_hover_text("Passageiros a bordo / lugares");
            ui.add_space(10.0);
            art(ui, game::Sprite::Caixote, 32.0);
            game::pixel_label(ui, &format!("{cargo}/{hold}"), 2.0, game::INK)
                .on_hover_text("Carga a bordo / espaço de carga");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                open_services = game::button(
                    ui,
                    "open-services",
                    Icon::Ticket,
                    "Serviços",
                    false,
                    egui::vec2(170.0, 36.0),
                )
                .clicked();
            });
        });
        if open_services {
            self.panel_tab = PanelTab::Journey;
            self.journey_page = 3;
        }
        ui.add_space(6.0);
        self.arrival_ticket(ui, true);
        if let Some(notice) = self.telemetry_notice() {
            ui.colored_label(game::BLUE, notice);
        }
        egui::CollapsingHeader::new("Simulação e diagnóstico")
            .default_open(self.demo_mode)
            .show(ui, |ui| {
                if self.demo_mode {
                    ui.colored_label(game::BLUE, "DEMO · CPU simulada; RAM indisponível");
                } else {
                    let mut manual = self.cpu_override.is_some();
                    if ui
                        .checkbox(&mut manual, "Simular CPU, sem gerar carga")
                        .changed()
                    {
                        self.cpu_override = manual.then_some(self.manual_value);
                    }
                }
                ui.add_enabled_ui(self.cpu_override.is_some(), |ui| {
                    ui.horizontal(|ui| {
                        for value in [0.0, 50.0, 100.0] {
                            if ui
                                .selectable_label(
                                    (self.manual_value - value).abs() < 0.05,
                                    format!("{value:.0}%"),
                                )
                                .clicked()
                            {
                                self.manual_value = value;
                                self.cpu_override = Some(value);
                            }
                        }
                    });
                    if ui
                        .add(
                            egui::Slider::new(&mut self.manual_value, 0.0..=100.0)
                                .suffix(" %")
                                .text("CPU simulada"),
                        )
                        .changed()
                    {
                        self.cpu_override = Some(self.manual_value);
                    }
                });
                if let MetricStatus::Unavailable(error) = &self.metrics.status {
                    ui.colored_label(game::BLUE, error);
                }
            });
    }

    fn desktop_absent(&self) -> bool {
        #[cfg(windows)]
        {
            self.desktop.is_none()
        }
        #[cfg(not(windows))]
        {
            true
        }
    }

    fn welcome_controls(&mut self, ui: &mut egui::Ui) {
        game::apply_style(ui);
        self.scene_preview(ui, true);
        ui.heading("Uma pequena ferrovia para acompanhar seu dia");
        ui.label("1. Escolha onde seu trem aparece.");
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.welcome_taskbar, true, "Faixa na barra de tarefas");
            ui.radio_value(&mut self.welcome_taskbar, false, "Janela flutuante");
        });
        ui.label("2. Viaje, transporte e recolha moedas em Missões.");
        ui.label("3. Clique no trem para voltar à central; Esc fecha o painel.");
        ui.separator();
        ui.label(format!(
            "Clima: {}. Altere em Ajustes > Clima.",
            self.config.weather_location.city
        ));
        ui.label("As leituras da máquina não saem do computador.");
        ui.checkbox(
            &mut self.config.use_input_activity,
            "Permitir atividade agregada do teclado e mouse",
        );
        ui.small("Sem ler teclas. Preferência para a Sala de máquinas; coleta ainda indisponível nesta etapa.");
        ui.label("O clima consulta o serviço de meteorologia. Não há progresso offline.");
        if let Some(notice) = self.notification_message() {
            ui.colored_label(game::RED, notice);
        }

        if ui.button("Começar viagem").clicked() {
            match self.local_weather.set_location(self.weather_draft.clone()) {
                Ok(()) => {
                    self.config.weather_location = self.weather_draft.clone();
                    self.config.taskbar_mode = self.welcome_taskbar;
                    self.welcome = false;
                    self.pending_dock = true;
                    self.next_taskbar_check = Instant::now();
                    self.preferences_changed();
                    self.persist_preferences(true);
                    self.local_weather.refresh(ui.ctx());
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
                Err(error) => self.add_notice(error),
            }
        }
    }

    fn taskbar_failure(&mut self) {
        self.taskbar_failures = self.taskbar_failures.saturating_add(1);
        if self.taskbar_failures >= 3 {
            self.config.taskbar_mode = false;
            self.welcome_taskbar = false;
            self.strip_gesture = None;
            self.slider_geometry = None;
            self.pending_dock = true;
            self.next_taskbar_check = Instant::now();
            self.show_controls = true;
            self.add_notice("Não foi possível confirmar a faixa na barra. O trem foi transferido para uma janela flutuante; você pode tentar a barra novamente em Janela.".to_owned());
            #[cfg(windows)]
            if let Some(desktop) = &mut self.desktop {
                let result = desktop.show();
                self.note_result(result);
            }
        }
    }

    fn commit_strip_geometry(&mut self, [offset, width, height]: [i32; 3]) {
        self.config.taskbar_offset = offset;
        self.config.taskbar_width = width;
        self.config.taskbar_height = height;
        self.preferences_changed();
    }

    #[cfg(windows)]
    fn manipulate_strip(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame, bounds: egui::Rect) {
        use platform::{StripGesture, StripHandle, strip_handle};
        let response = ui.interact(
            bounds,
            egui::Id::unique("strip-manipulation"),
            egui::Sense::click_and_drag(),
        );
        let handle = ui.input(|i| i.pointer.hover_pos()).map(|p| {
            strip_handle(
                [p.x - bounds.left(), p.y - bounds.top()],
                [bounds.width(), bounds.height()],
                ui.ctx().pixels_per_point(),
                self.config.taskbar_fit_height,
            )
        });
        if response.hovered() || self.strip_gesture.is_some() {
            ui.ctx().set_cursor_icon(
                match self.strip_gesture.as_ref().map(|g| g.handle).or(handle) {
                    Some(StripHandle::Left | StripHandle::Right) => {
                        egui::CursorIcon::ResizeHorizontal
                    }
                    Some(StripHandle::Top) => egui::CursorIcon::ResizeVertical,
                    _ if self.strip_gesture.is_some() => egui::CursorIcon::Grabbing,
                    _ => egui::CursorIcon::Grab,
                },
            );
        }
        let pressed = ui.input(|i| i.pointer.primary_pressed());
        if (self.strip_gesture.is_some() || (pressed && response.is_pointer_button_down_on()))
            && let Ok((area, placement, pointer)) = platform::strip_environment(frame)
        {
            if self.strip_gesture.is_none() {
                self.strip_gesture = Some(StripGesture::new(
                    placement,
                    area,
                    handle.unwrap_or(StripHandle::Move),
                    pointer,
                ));
            }
            let gesture = self.strip_gesture.as_mut().expect("gesto iniciado");
            gesture.update(pointer, self.config.taskbar_fit_height);
            let p = gesture.current;
            if gesture.moved && p != placement {
                let result = self
                    .desktop
                    .as_mut()
                    .expect("desktop disponível")
                    .place_strip(
                        frame,
                        p.x - area.left,
                        p.width,
                        p.height,
                        self.config.taskbar_fit_height,
                        self.config.taskbar_rounded,
                    );
                self.note_result(result);
            }
        }
        let had_gesture = self.strip_gesture.is_some();
        if !ui.input(|i| i.pointer.primary_down())
            && let Some(gesture) = self.strip_gesture.take()
        {
            if gesture.moved {
                self.commit_strip_geometry([
                    gesture.current.x - gesture.area.left,
                    gesture.current.width,
                    gesture.current.height,
                ]);
                self.persist_preferences(true);
            } else if response.hovered() {
                self.dispatch(AppAction::ToggleControls, ui.ctx());
            }
        }
        if !had_gesture && response.clicked() {
            // Pressão e soltura podem chegar no mesmo frame, sem estado intermediário.
            self.dispatch(AppAction::ToggleControls, ui.ctx());
        }
        if response.secondary_clicked() {
            self.dispatch(AppAction::TogglePause, ui.ctx());
        }
    }

    fn behavior_controls(&mut self, ui: &mut egui::Ui) {
        if self.menu.window == 0 {
            let mut placement_changed = ui
                .checkbox(&mut self.config.taskbar_mode, "Na área da barra de tarefas")
                .changed();
            if self.config.taskbar_mode {
                let bar_width = self.native_status["taskbar_width"]
                    .as_i64()
                    .filter(|v| *v > 0)
                    .unwrap_or(1920) as i32;
                let bar_height = self.native_status["taskbar_height"]
                    .as_i64()
                    .filter(|v| (24..=240).contains(v))
                    .unwrap_or(60) as i32;
                let mut geometry = self.slider_geometry.unwrap_or([
                    self.config.taskbar_offset,
                    self.config.taskbar_width,
                    self.config.taskbar_height,
                ]);
                let effective_width = if geometry[1] == 0 {
                    (bar_height * 32 / 7).min(bar_width / 3)
                } else {
                    geometry[1].min(bar_width)
                };
                let mut changed = ui
                    .add(
                        egui::Slider::new(
                            &mut geometry[0],
                            0..=(bar_width - effective_width).max(0),
                        )
                        .text("Posição horizontal (px)"),
                    )
                    .changed();
                let mut automatic = geometry[1] == 0;
                if ui.checkbox(&mut automatic, "Largura automática").changed() {
                    geometry[1] = if automatic {
                        0
                    } else {
                        effective_width.max(160)
                    };
                    changed = true;
                }
                if !automatic {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut geometry[1], 160.min(bar_width)..=bar_width)
                                .text("Largura (px)"),
                        )
                        .changed();
                }
                placement_changed |= ui
                    .checkbox(
                        &mut self.config.taskbar_fit_height,
                        "Encaixar na altura da barra",
                    )
                    .changed();
                if !self.config.taskbar_fit_height {
                    if geometry[2] == 0 {
                        geometry[2] = bar_height.clamp(24, 240);
                    }
                    changed |= ui
                        .add(egui::Slider::new(&mut geometry[2], 24..=240).text("Altura (px)"))
                        .changed();
                }
                placement_changed |= ui
                    .checkbox(&mut self.config.taskbar_rounded, "Cantos arredondados")
                    .changed();
                if changed || self.slider_geometry.is_some() {
                    let actual_width = if geometry[1] == 0 {
                        (bar_height * 32 / 7)
                            .min(bar_width / 3)
                            .max(160.min(bar_width))
                    } else {
                        geometry[1].clamp(160.min(bar_width), bar_width)
                    };
                    geometry[0] = geometry[0].clamp(0, bar_width - actual_width);
                    if ui.input(|i| i.pointer.primary_down()) {
                        self.slider_geometry = Some(geometry);
                        if changed {
                            self.pending_dock = true;
                            self.next_taskbar_check = Instant::now();
                        }
                    } else {
                        self.slider_geometry = None;
                        self.commit_strip_geometry(geometry);
                        placement_changed = true;
                    }
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
                game::hint(
                    ui,
                    "Arraste a faixa ou suas laterais. Altura livre: arraste a borda superior; o excedente cobre a área acima da barra. Cantos arredondados requerem Windows 11.",
                );
            } else {
                let rect = self.config.floating_rect.get_or_insert([40, 40, 640, 180]);
                ui.horizontal(|ui| {
                    ui.label("X / Y (px)");
                    placement_changed |= ui
                        .add(egui::DragValue::new(&mut rect[0]).range(-32768..=32768))
                        .changed();
                    placement_changed |= ui
                        .add(egui::DragValue::new(&mut rect[1]).range(-32768..=32768))
                        .changed();
                });
                placement_changed |= ui
                    .add(egui::Slider::new(&mut rect[2], 320..=2400).text("Largura (px)"))
                    .changed();
                placement_changed |= ui
                    .add(egui::Slider::new(&mut rect[3], 140..=1200).text("Altura (px)"))
                    .changed();
                game::hint(
                    ui,
                    "Arraste pelo título da janela e redimensione pelas bordas. Posição e tamanho ficam salvos.",
                );
            }
            if ui.button("Restaurar posição e tamanho").clicked() {
                self.slider_geometry = None;
                self.config.taskbar_fit_height = true;
                self.config.taskbar_offset = 0;
                self.config.taskbar_width = 0;
                self.config.taskbar_height = 0;
                self.config.floating_rect = Some([40, 40, 640, 180]);
                placement_changed = true;
            }
            if placement_changed {
                self.pending_dock = true;
                self.next_taskbar_check = Instant::now();
                self.preferences_changed();
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
            game::hint(
                ui,
                "Clique na faixa para abrir o painel; botão direito pausa.",
            );
        }
        if self.menu.window == 1 {
            let mut pinned = self.config.always_on_top;
            if ui.checkbox(&mut pinned, "Sempre no topo (P)").changed() {
                self.dispatch(AppAction::TogglePin, ui.ctx());
            }
            let mut changed = ui
                .checkbox(
                    &mut self.config.pause_when_unfocused,
                    "Pausar ao trabalhar em outra janela",
                )
                .changed();
            changed |= ui
                .checkbox(
                    &mut self.config.start_paused,
                    "Iniciar pausado na próxima abertura",
                )
                .changed();
            if changed {
                self.preferences_changed();
                self.sync_clock();
            }
            ui.horizontal(|ui| {
                if ui
                    .button("Minimizar")
                    .on_hover_text("Restaurar pela barra de tarefas")
                    .clicked()
                {
                    self.dispatch(AppAction::Minimize, ui.ctx());
                }
                if ui
                .add_enabled(
                    self.tray_available(),
                    egui::Button::new("Ocultar na bandeja"),
                )
                .on_hover_text(if self.tray_available() {
                    "Restaurar pelo ícone do CPU Express na área de notificação do Windows."
                } else {
                    "A bandeja não está disponível. Use Minimizar e restaure pela barra de tarefas."
                })
                .clicked()
            {
                self.dispatch(AppAction::HideToTray, ui.ctx());
            }
            });
            ui.label(
                egui::RichText::new(
                    "A coleta continua durante a pausa; a paisagem fica congelada.",
                )
                .size(11.0),
            );
        }
    }

    fn help_contents(&self, ui: &mut egui::Ui) {
        for (key, action) in [
            ("Espaço", "Pausar / retomar a viagem"),
            ("M", "HUD: compacto, completo e limpo"),
            ("P", "Alternar sempre no topo"),
            ("O / F1", "Abrir opções / ajuda"),
            ("R", "Reposicionar perto da barra de tarefas"),
            ("Esc", "Fechar painel; nunca encerra o app"),
            ("Ctrl+Q", "Sair (ou use o botão Sair)"),
        ] {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [62.0, 18.0],
                    egui::Label::new(egui::RichText::new(key).monospace().color(game::BLUE)),
                );
                ui.label(egui::RichText::new(action).size(11.0));
            });
        }
        ui.separator();
        ui.label("Arraste uma área livre da paisagem. Os atalhos funcionam com foco na janela e cedem ao editar campos.");
        ui.label("Simular CPU não aquece o processador de propósito. DEMO não lê CPU nem RAM reais. A velocidade é uma unidade do jogo.");
        if let Some(message) = self.notification_message() {
            ui.separator();
            ui.label(egui::RichText::new(message).color(game::BLUE));
        }
    }

    fn overlay(&mut self, ctx: &egui::Context, bounds: egui::Rect) {
        if self.welcome && self.show_controls {
            egui::Window::new("Sua primeira viagem").show(ctx, |ui| self.welcome_controls(ui));
            return;
        }
        if !self.show_controls && !self.show_help {
            return;
        }
        let help = self.show_help;
        let width = 365.0_f32.min(bounds.width() - 8.0);
        let height = (bounds.height() - 8.0).max(90.0);
        let mut close = false;
        egui::Window::new("CPU Express — painel")
            .id(egui::Id::unique("cpu-express-panel"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .movable(false)
            .fixed_pos(egui::pos2(bounds.right() - width - 4.0, bounds.top() + 4.0))
            .fixed_size(egui::vec2(width - 16.0, height - 16.0))
            .constrain_to(bounds)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(77, 101, 88)))
                    .corner_radius(6)
                    .inner_margin(7),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(if help {
                        "Ajuda e atalhos"
                    } else if self.demo_mode {
                        "Opções · DEMO"
                    } else if self.cpu_override.is_some() {
                        "Opções · SIM"
                    } else {
                        "Opções"
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close = ui.button("Fechar · Esc").clicked();
                        if ui
                            .button("Sair")
                            .on_hover_text("Encerrar o aplicativo (Ctrl+Q)")
                            .clicked()
                        {
                            self.dispatch(AppAction::Quit, ctx);
                        }
                    });
                });
                if !help {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.panel_tab, PanelTab::Drive, "Trem");
                        ui.selectable_value(&mut self.panel_tab, PanelTab::Journey, "Viagem");
                        ui.selectable_value(&mut self.panel_tab, PanelTab::Appearance, "Visual");
                        ui.selectable_value(&mut self.panel_tab, PanelTab::Behavior, "Janela");
                    });
                }
                let max_height = if help { height - 46.0 } else { height - 72.0 };
                let salt = if help {
                    "help-scroll"
                } else {
                    match self.panel_tab {
                        PanelTab::Drive => "drive-scroll",
                        PanelTab::Journey => "journey-scroll",
                        PanelTab::Appearance => "appearance-scroll",
                        PanelTab::Behavior => "behavior-scroll",
                        PanelTab::Shop => "shop-scroll",
                        PanelTab::Collection => "collection-scroll",
                    }
                };
                egui::ScrollArea::vertical()
                    .id_salt(salt)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .max_height(max_height.max(30.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(width - 34.0);
                        if help {
                            self.help_contents(ui);
                        } else {
                            match self.panel_tab {
                                PanelTab::Drive => self.drive_controls(ui),
                                PanelTab::Journey => self.journey_controls(ui),
                                PanelTab::Appearance => self.appearance_controls(ui),
                                PanelTab::Behavior => self.behavior_controls(ui),
                                PanelTab::Shop => self.shop_preview(ui),
                                PanelTab::Collection => self.postcards_page(ui),
                            }
                        }
                        // Tab may focus a control below the 60-point viewport.
                        // Bring that focused control into view before using it.
                        if let Some(id) = ui.memory(|memory| memory.focused())
                            && let Some(response) = ui.ctx().read_response(id)
                            && response.gained_focus()
                            && ui.min_rect().intersects(response.rect)
                        {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                    });
            });
        if close {
            self.dispatch(AppAction::ClosePanel, ctx);
        }
    }

    fn floating_panel(&mut self, ctx: &egui::Context, bounds: egui::Rect) {
        if !self.show_controls && !self.show_help {
            self.panel_frames = 0;
            return;
        }
        if ctx.embed_viewports() {
            self.overlay(ctx, bounds);
            return;
        }
        // The native window is created hidden and revealed once its content was
        // painted; otherwise an empty title bar flashes before the panel appears.
        let revealed = self.panel_frames >= 2;
        self.panel_frames = self.panel_frames.saturating_add(1);
        if !revealed {
            ctx.request_repaint();
        }
        game::set_reduce_motion(ctx, self.config.reduce_menu_motion);
        let mut saved = self.config.panel_rect.unwrap_or([24.0, 24.0, 704.0, 676.0]);
        saved[2] = 704.0;
        saved[3] = 676.0;
        let native_scale = ctx
            .input(|i| i.viewport().native_pixels_per_point)
            .unwrap_or(1.0);
        let fitted = platform::clamp_panel_rect(saved, native_scale);
        let scale = menus::panel_scale([fitted[2], fitted[3]]);
        ctx.set_zoom_factor(scale);
        let builder = egui::ViewportBuilder::default()
            .with_title(platform::PANEL_TITLE)
            .with_inner_size([704.0, 676.0])
            .with_min_inner_size([704.0, 676.0])
            .with_max_inner_size([704.0, 676.0])
            .with_position(egui::pos2(fitted[0] / scale, fitted[1] / scale))
            .with_resizable(false)
            .with_taskbar(false)
            .with_decorations(false)
            .with_visible(revealed)
            .with_window_level(egui::WindowLevel::AlwaysOnTop);
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("journey-panel"),
            builder,
            |root_ui, _class| {
                let ctx = &root_ui.ctx().clone();
                self.shortcuts(ctx);
                if ctx.input(|i| i.viewport().close_requested() || i.key_pressed(egui::Key::Escape))
                {
                    self.show_controls = false;
                    self.show_help = false;
                    if self.slider_geometry.take().is_some() {
                        self.pending_dock = true;
                        self.next_taskbar_check = Instant::now();
                    }
                }
                if Instant::now() >= self.next_panel_save {
                    self.next_panel_save = Instant::now() + Duration::from_secs(1);
                    let rect = ctx.input(|i| {
                        let v = i.viewport();
                        if v.minimized == Some(true) {
                            return None;
                        }
                        v.outer_rect.zip(v.inner_rect).map(|(outer, inner)| {
                            let _ = inner;
                            [outer.left() * scale, outer.top() * scale, 704.0, 676.0]
                        })
                    });
                    if let Some(rect) = rect
                        && rect[2] >= 480.0
                        && rect[3] >= 480.0
                        && self.config.panel_rect != Some(rect)
                    {
                        self.config.panel_rect = Some(rect);
                        self.preferences_changed();
                    }
                }
                // Zoom changes pixels per point, preserving one layout on small monitors.
                menus::synchronize_panel_size(ctx);
                self.central_menu(root_ui);
                self.capture_for_validation(ctx, 1);
            },
        );
    }

    fn capture_for_validation(&mut self, ctx: &egui::Context, index: usize) {
        if self.validation_captures[index] || self.opened.elapsed() < Duration::from_secs(3) {
            return;
        }
        let Some(directory) = std::env::var_os("CPU_EXPRESS_CAPTURE_DIR") else {
            return;
        };
        if let Some(step) = self.layout_test_step
            && self.opened.elapsed().as_secs_f64() < f64::from(step) * 4.0 + 2.0
        {
            return;
        }
        self.validation_captures[index] = true;
        let name = if index == 0 { "faixa" } else { "painel" };
        let filename = self.layout_test_step.map_or_else(
            || format!("{name}.ppm"),
            |step| format!("{name}-stage-{step}.ppm"),
        );
        let path = PathBuf::from(directory).join(filename);
        // Explicit opt-in framebuffer capture of this app only, never the desktop.
        ctx.request_screenshot(move |image| {
            use std::io::Write;
            let result = (|| -> std::io::Result<()> {
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)?;
                let mut output = std::io::BufWriter::new(file);
                write!(output, "P6\n{} {}\n255\n", image.size[0], image.size[1])?;
                for pixel in &image.pixels {
                    output.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
                }
                output.flush()
            })();
            if let Err(error) = result {
                eprintln!("capture: {error}");
            }
        });
    }
}

impl eframe::App for CpuExpressApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.local_weather.poll(ctx) {
            self.scene_dirty = true;
        }
        if self.local_weather.enabled {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        if self.closing {
            return;
        }
        // A native handle is absent in the headless egui harness. Initialize
        // the tray only for a real window, once, without stealing keyboard focus.
        #[cfg(windows)]
        if self.desktop.is_none() && frame.window_handle().is_ok() {
            let mut desktop = platform::DesktopController::new(ctx, self.config.always_on_top);
            let result = desktop.enable_tray();
            self.desktop = Some(desktop);
            self.note_result(result);
        }
        // Explicit smoke-only transition driver; never enabled for a normal profile.
        if let Some(step) = self.layout_test_step
            && step < 4
            && self.opened.elapsed().as_secs() >= u64::from(step + 1) * 4
        {
            self.layout_test_step = Some(step + 1);
            self.config.taskbar_mode = step % 2 == 1;
            self.config.floating_rect = Some([120, 140, 800, 240]);
            self.pending_dock = true;
            self.next_taskbar_check = Instant::now();
            self.validation_captures = [false; 2];
        }
        #[cfg(windows)]
        if self.pending_dock
            && self.desktop.is_some()
            && self.applied_window_mode != Some(self.config.taskbar_mode)
        {
            // Viewport commands run after this frame. Apply geometry only on
            // the next frame, after winit has updated decorations and client area.
            self.strip_gesture = None;
            if let Some(desktop) = &mut self.desktop {
                let result = desktop
                    .set_surface_topmost(self.config.taskbar_mode || self.config.always_on_top);
                self.note_result(result);
            }
            self.applied_window_mode = Some(self.config.taskbar_mode);
            ctx.send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                egui::ViewportCommand::Decorations(!self.config.taskbar_mode),
            );
            ctx.send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                egui::ViewportCommand::Resizable(!self.config.taskbar_mode),
            );
            ctx.request_repaint();
            return;
        }
        if self.pending_dock
            && self.strip_gesture.is_none()
            && Instant::now() >= self.next_taskbar_check
        {
            self.pending_dock = false;
            #[cfg(windows)]
            let result = if self.config.taskbar_mode
                && let Some(desktop) = &mut self.desktop
            {
                desktop.place_strip(
                    frame,
                    self.slider_geometry
                        .map_or(self.config.taskbar_offset, |g| g[0]),
                    self.slider_geometry
                        .map_or(self.config.taskbar_width, |g| g[1]),
                    self.slider_geometry
                        .map_or(self.config.taskbar_height, |g| g[2]),
                    self.config.taskbar_fit_height,
                    self.config.taskbar_rounded,
                )
            } else if self.desktop.is_some() {
                platform::place_floating(
                    frame,
                    self.config.floating_rect.unwrap_or([40, 40, 640, 180]),
                )
            } else {
                self.desktop.as_mut().map_or_else(
                    || platform::dock_window(frame),
                    platform::DesktopController::dock,
                )
            };
            #[cfg(not(windows))]
            let result = platform::dock_window(frame);
            if result.is_err() {
                self.pending_dock = true;
                self.next_taskbar_check = Instant::now() + Duration::from_secs(2);
                if self.config.taskbar_mode {
                    self.taskbar_failure();
                }
            } else {
                self.taskbar_failures = 0;
            }
            self.note_result(result);
            if !self.pending_dock {
                // Do not persist a transitional rectangle from queued resize events.
                self.next_taskbar_check = Instant::now() + Duration::from_millis(300);
            }
        }
        #[cfg(windows)]
        if let Some(desktop) = &mut self.desktop {
            let scale = ctx.input(|input| input.viewport().native_pixels_per_point);
            let scale_changed = self.native_scale.is_some() && scale != self.native_scale;
            self.native_scale = scale;
            let scale_result = if scale_changed {
                desktop.display_changed()
            } else {
                Ok(())
            };
            // Tray restore/quit and topmost preferences must be processed even
            // while the UI itself does not render a frame.
            let update = desktop.update(frame);
            self.note_result(scale_result);
            for error in update.errors {
                self.add_notice(error);
            }
            if let Some(topmost) = update.topmost_changed {
                self.config.always_on_top = topmost;
                self.preferences_changed();
            }
            if update.exit_requested {
                self.closing = true;
                return;
            }
        }
        #[cfg(windows)]
        if self.config.taskbar_mode
            && self.strip_gesture.is_none()
            && !self
                .desktop
                .as_ref()
                .is_some_and(platform::DesktopController::animation_suspended)
            && !self.pending_dock
            && self.desktop.is_some()
            && Instant::now() >= self.next_taskbar_check
        {
            self.next_taskbar_check = Instant::now() + Duration::from_secs(2);
            if let Err(error) = self
                .desktop
                .as_mut()
                .expect("desktop disponível")
                .place_strip(
                    frame,
                    self.slider_geometry
                        .map_or(self.config.taskbar_offset, |g| g[0]),
                    self.slider_geometry
                        .map_or(self.config.taskbar_width, |g| g[1]),
                    self.slider_geometry
                        .map_or(self.config.taskbar_height, |g| g[2]),
                    self.config.taskbar_fit_height,
                    self.config.taskbar_rounded,
                )
                .and_then(|()| platform::confirm_strip_surface(frame))
            {
                self.taskbar_failure();
                self.add_notice(error);
            } else {
                self.taskbar_failures = 0;
            }
            self.native_status = platform::taskbar_status(frame).unwrap_or(serde_json::Value::Null);
        }
        // Activating another window makes Windows raise the taskbar over the strip.
        // Put the strip back within a tenth of a second, without taking focus.
        #[cfg(windows)]
        if self.config.taskbar_mode
            && !self.pending_dock
            && !self.was_hidden
            && self.desktop.is_some()
            && Instant::now() >= self.next_strip_raise
        {
            self.next_strip_raise = Instant::now() + Duration::from_millis(100);
            if platform::keep_strip_above_taskbar(frame) == Ok(true) {
                ctx.request_repaint();
            }
        }
        #[cfg(windows)]
        if !self.config.taskbar_mode
            && !self.pending_dock
            && self.desktop.is_some()
            && Instant::now() >= self.next_taskbar_check
        {
            self.next_taskbar_check = Instant::now() + Duration::from_secs(1);
            if let Ok(status) = platform::taskbar_status(frame) {
                let rect = ["x", "y", "width", "height"]
                    .map(|key| status[key].as_i64().unwrap_or(0) as i32);
                if rect[2] >= 320
                    && rect[3] >= 140
                    && rect[0] > -32000
                    && rect[1] > -32000
                    && self.config.floating_rect != Some(rect)
                {
                    self.config.floating_rect = Some(rect);
                    self.preferences_changed();
                }
                self.native_status = status;
            }
        }
        self.persist_progress(false);
        if let Some(sample) = self.worker.as_ref().and_then(MetricsWorker::poll) {
            self.metrics = sample;
            self.samples += 1;
        }
        let viewport_hidden = ctx.input(|input| {
            (input.viewport().visible() == Some(false) && self.texture.is_some())
                || input.viewport().minimized == Some(true)
        });
        #[cfg(windows)]
        let hidden = self.desktop.as_ref().map_or(
            viewport_hidden,
            platform::DesktopController::animation_suspended,
        );
        #[cfg(not(windows))]
        let hidden = viewport_hidden;
        self.focused = ctx.input(|input| {
            input.focused
                || input
                    .raw
                    .viewports
                    .values()
                    .any(|v| v.focused == Some(true))
        });
        if hidden != self.was_hidden {
            self.next_frame = Instant::now();
        }
        self.was_hidden = hidden;
        self.sync_clock();
        self.record_diagnostics(false);
        self.persist_preferences(false);
        if self
            .exit_after
            .is_some_and(|timeout| self.opened.elapsed() >= timeout)
        {
            eprintln!(
                "smoke: elapsed={:.2}s frames={} mode={} paused={} cpu={:.1}% speed={:.3}km/h",
                self.opened.elapsed().as_secs_f64(),
                self.frames,
                if self.demo_mode {
                    "demo"
                } else {
                    "real/manual"
                },
                self.suspended(),
                self.cpu_percent(),
                self.train.speed_kmh()
            );
            self.dispatch(AppAction::Quit, ctx);
            return;
        }
        if hidden {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.closing {
            return;
        }
        if self.was_hidden {
            ui.ctx().request_repaint_after(Duration::from_secs(1));
            return;
        }
        self.focused = ui.input(|input| {
            input.focused
                || input
                    .raw
                    .viewports
                    .values()
                    .any(|v| v.focused == Some(true))
        });
        self.sync_clock();
        self.shortcuts(ui.ctx());
        if self.closing {
            return;
        }
        let bounds = ui.max_rect().intersect(ui.ctx().content_rect());
        let strip_ui =
            (self.config.taskbar_mode && !self.desktop_absent()) || bounds.height() < 85.0;
        let ppp = ui.ctx().pixels_per_point();
        let physical_scale = ((bounds.height() * ppp / HEIGHT as f32)
            .min(bounds.width() * ppp / if strip_ui { 240.0 } else { WIDTH as f32 }))
        .floor()
        .max(1.0);
        let width = if strip_ui {
            (bounds.width() * ppp / physical_scale).ceil() as usize
        } else {
            WIDTH
        };
        self.scene_dirty |= self.renderer.set_width(width);
        self.step_frame(ui.ctx());
        ui.ctx().request_repaint_after(
            self.next_frame
                .saturating_duration_since(Instant::now())
                .saturating_sub(Duration::from_millis(1))
                .max(Duration::from_millis(1)),
        );
        // The allocated UI can retain a larger minimum from the floating toolbar.
        // Render within the actual viewport, never that retained allocation.
        let bounds = ui.max_rect().intersect(ui.ctx().content_rect());
        self.render_surface_size = [
            bounds.width() * ui.ctx().pixels_per_point(),
            bounds.height() * ui.ctx().pixels_per_point(),
        ];
        if let Some(texture) = &self.texture {
            let pixels_per_point = ui.ctx().pixels_per_point();
            let scale = physical_scale / pixels_per_point;
            let size = egui::vec2(self.renderer.width() as f32 * scale, HEIGHT as f32 * scale);
            let scene_top = if bounds.height() < 85.0 {
                bounds.bottom() - size.y
            } else {
                bounds.center().y - size.y * 0.5
            };
            // On a taskbar shorter than 70 physical pixels, crop the sky instead
            // of shrinking the train. Every source pixel stays a whole pixel.
            let scene_bounds = egui::Rect::from_min_size(
                egui::pos2(
                    ((bounds.center().x - size.x * 0.5) * pixels_per_point).round()
                        / pixels_per_point,
                    (scene_top * pixels_per_point).round() / pixels_per_point,
                ),
                size,
            );
            ui.painter().rect_filled(bounds, 0.0, PANEL);
            ui.painter().image(
                texture.id(),
                scene_bounds,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        if strip_ui {
            #[cfg(windows)]
            if self.config.taskbar_mode && self.desktop.is_some() {
                self.manipulate_strip(ui, _frame, bounds);
            }
            let response = ui.interact(
                bounds,
                egui::Id::unique("taskbar-train"),
                if self.config.taskbar_mode && !self.desktop_absent() {
                    egui::Sense::hover()
                } else {
                    egui::Sense::CLICK
                },
            );
            if response.clicked() && (!self.config.taskbar_mode || self.desktop_absent()) {
                self.dispatch(AppAction::ToggleControls, ui.ctx());
            }
            if response.secondary_clicked() {
                self.dispatch(AppAction::TogglePause, ui.ctx());
            }
            response.on_hover_text(format!(
                "CPU Express · {} · {:.0} km/h\n{}\nClique: painel · Botão direito: pausa",
                self.cpu_label(),
                self.train.speed_kmh(),
                self.world.journey_snapshot().next_station.name
            ));
            let progress = self.world.journey_snapshot().segment_progress as f32;
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    bounds.left_bottom() - egui::vec2(0.0, 2.0),
                    egui::vec2(bounds.width() * progress, 2.0),
                ),
                0.0,
                ACCENT,
            );
            if self.paused {
                ui.painter().text(
                    bounds.right_top() + egui::vec2(-4.0, 3.0),
                    egui::Align2::RIGHT_TOP,
                    "||",
                    egui::FontId::proportional(13.0),
                    INK,
                );
            }
            self.capture_for_validation(ui.ctx(), 0);
            self.floating_panel(ui.ctx(), bounds);
            self.persist_preferences(false);
            return;
        }
        let toolbar_visible = self.toolbar_visible(ui, bounds);
        // Scene drag never steals a click from controls or an open overlay.
        if !self.show_controls && !self.show_help {
            let bottom = if self.config.effective_hud_mode() == HudMode::Clean {
                0.0
            } else {
                26.0
            };
            let top = if self.telemetry_notice().is_some() {
                59.0
            } else {
                34.0
            };
            let drag_area = egui::Rect::from_min_max(
                bounds.left_top() + egui::vec2(0.0, top),
                bounds.right_bottom() - egui::vec2(0.0, bottom),
            );
            if drag_area.is_positive()
                && ui
                    .interact(drag_area, egui::Id::unique("drag-scene"), egui::Sense::DRAG)
                    .drag_started_by(egui::PointerButton::Primary)
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        }
        self.header(ui, bounds, toolbar_visible);
        self.status_bar(ui, bounds);
        self.journey_indicator(ui, bounds);
        self.capture_for_validation(ui.ctx(), 0);
        self.floating_panel(ui.ctx(), bounds);
        self.persist_preferences(false);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.record_diagnostics(true);
        self.persist_preferences(true);
        self.persist_progress(true);
        #[cfg(windows)]
        if let Some(desktop) = &mut self.desktop
            && let Err(error) = desktop.shutdown()
        {
            eprintln!("desktop cleanup: {error}");
        }
        self.worker.take();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn preference_failure_retains_dirty_state_and_recovers_without_another_edit() {
        let directory = std::env::temp_dir().join(format!("ce-retry-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let blocked = directory.join("blocked");
        std::fs::write(&blocked, b"obstruction").unwrap();
        let path = blocked.join("config.json");
        let mut app = super::CpuExpressApp::new_demo(
            &eframe::egui::Context::default(),
            super::LaunchOptions {
                config: crate::config::Config::default(),
                config_path: Some(path.clone()),
                warning: None,
                cpu_override: None,
                exit_after: None,
                diagnostics: None,
            },
        );
        app.config.start_paused = true;
        app.preferences_changed();
        app.persist_preferences(true);
        assert!(app.preferences_dirty);
        assert!(app.save_after > std::time::Instant::now());
        std::fs::remove_file(&blocked).unwrap();
        app.persist_preferences(true);
        assert!(!app.preferences_dirty);
        assert!(crate::config::load(Some(&path)).0.start_paused);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(blocked).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
    use super::*;

    #[test]
    fn gesto_e_slider_so_gravam_geometria_confirmada() {
        let mut app = app_with_sample(MetricsSample::unavailable("teste"));
        let path = std::env::temp_dir().join(format!(
            "ce-faixa-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        app.config_path = Some(path.clone());
        let original = app.config.clone();
        let area = platform::PhysicalRect {
            left: 0,
            top: 1000,
            right: 1920,
            bottom: 1060,
        };
        let p = platform::strip_placement(area, 0, 400, 0, true).unwrap();
        let mut gesture =
            platform::StripGesture::new(p, area, platform::StripHandle::Right, [400, 1030]);
        gesture.update([700, 1030], true);
        app.strip_gesture = Some(gesture);
        app.slider_geometry = Some([120, 800, 160]);
        app.preferences_changed();
        app.persist_preferences(true);
        assert_eq!(config::load(Some(&path)).0, original);
        app.commit_strip_geometry([120, 800, 160]);
        app.persist_preferences(true);
        let saved = config::load(Some(&path)).0;
        assert_eq!(
            (
                saved.taskbar_offset,
                saved.taskbar_width,
                saved.taskbar_height
            ),
            (120, 800, 160)
        );
    }

    #[test]
    fn acolhida_e_recuperacao_nao_dependem_de_config_valida() {
        let path = std::env::temp_dir().join(format!(
            "ce-acolhida-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(config::first_launch(Some(&path)));
        let mut app = CpuExpressApp::new(
            &egui::Context::default(),
            LaunchOptions {
                config: Config::default(),
                config_path: Some(path.clone()),
                warning: None,
                cpu_override: None,
                exit_after: Some(Duration::from_secs(1)),
                diagnostics: None,
            },
        );
        assert!(app.welcome && app.show_controls);
        app.preferences_changed();
        app.persist_preferences(true);
        assert!(!path.exists());
        app.taskbar_failure();
        app.taskbar_failure();
        assert!(app.config.taskbar_mode);
        app.taskbar_failure();
        assert!(!app.config.taskbar_mode);
        assert!(!app.welcome_taskbar);
        assert!(app.pending_dock && app.show_controls);
        assert!(
            app.notification_message()
                .unwrap()
                .contains("janela flutuante")
        );
        std::fs::write(&path, b"{").unwrap();
        assert!(!config::first_launch(Some(&path)));
        assert!(!config::first_launch(None));
    }

    #[test]
    fn acolhida_confirma_escolhas_e_nao_reaparece_no_perfil_salvo() {
        let mut app = app_with_sample(MetricsSample::unavailable("teste"));
        app.welcome = true;
        app.welcome_taskbar = false;
        app.weather_draft = crate::local_weather::WeatherLocation {
            city: "Cidade de teste".to_owned(),
            latitude: -25.0,
            longitude: -49.0,
        };
        let path = std::env::temp_dir().join(format!(
            "ce-acolhida-ui-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        app.config_path = Some(path.clone());
        let ctx = egui::Context::default();
        let mut pos = None;
        for events in [vec![], vec![], vec![]] {
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 720.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| app.welcome_controls(ui),
            );
            pos = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == "Começar viagem" => {
                        Some(text.visual_bounding_rect().center())
                    }
                    _ => None,
                })
                .or(pos);
            output.drop_without_applying_deltas();
        }
        let pos = pos.expect("botão de confirmação visível");
        for pressed in [true, false] {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 720.0),
                    )),
                    events: vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
                |ui| app.welcome_controls(ui),
            )
            .drop_without_applying_deltas();
        }
        assert!(!app.welcome);
        let (saved, warning) = config::load(Some(&path));
        assert!(warning.is_none());
        assert!(!saved.taskbar_mode);
        assert_eq!(saved.weather_location, app.weather_draft);
        assert!(!config::first_launch(Some(&path)));
    }

    #[test]
    fn failed_checkpoint_is_reported_once_and_retried_only_on_the_next_interval() {
        let directory = std::env::temp_dir().join(format!("ce-progress-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let blocked = directory.join("blocked");
        std::fs::write(&blocked, b"obstruction").unwrap();
        let path = blocked.join("config.journey.json");
        let mut app = app_with_sample(MetricsSample::unavailable("test"));
        app.progress_path = Some(path.clone());
        app.persist_progress(true);
        assert!(app.progress_notice.is_some());
        assert!(app.progress_after > Instant::now() + Duration::from_secs(20));
        // The obstacle is gone, but an ordinary frame does not retry before the interval.
        std::fs::remove_file(&blocked).unwrap();
        for _ in 0..100 {
            app.persist_progress(false);
        }
        assert!(app.progress_notice.is_some());
        assert!(!path.exists());
        // The interval, the explicit button and a normal exit all use this forced path.
        app.persist_progress(true);
        assert_eq!(app.progress_notice, None);
        assert!(crate::progress::load_world(&path).unwrap().is_some());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn diagnostic_record_identifies_version_mode_freshness_pause_and_journey() {
        let directory = std::env::temp_dir().join(format!("ce-diag-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("samples.jsonl");
        let mut app = app_with_sample(MetricsSample::unavailable("test"));
        app.diagnostics = Some(Diagnostics::open(&path).unwrap());
        app.paused = true;
        app.record_diagnostics(true);
        app.diagnostics.take();
        let text = std::fs::read_to_string(&path).unwrap();
        let record: serde_json::Value = serde_json::from_str(text.trim()).unwrap();
        assert_eq!(record["diagnostics_schema_version"], 3);
        assert_eq!(record["app_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(record["event"], "shutdown");
        assert_eq!(record["mode"], "real");
        assert_eq!(record["metric_status"], "unavailable");
        assert!(record["metric_freshness"].is_string());
        assert!(record["cpu_percent"].is_null());
        assert_eq!(record["paused"], true);
        assert_eq!(record["stations_visited"], 0);
        assert!(record["journey_active_seconds"].is_number());
        assert!(record["services"]["composition"].is_array());
        // Local-only file: nothing that names the machine, account or profile location.
        for forbidden in ["USERNAME", "COMPUTERNAME", "USERPROFILE", "LOCALAPPDATA"] {
            if let Ok(value) = std::env::var(forbidden)
                && value.len() > 3
            {
                assert!(!text.contains(&value), "{forbidden}");
            }
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn app_with_sample(sample: MetricsSample) -> CpuExpressApp {
        let mut app = CpuExpressApp::new_demo(
            &egui::Context::default(),
            LaunchOptions {
                config: Config::default(),
                config_path: None,
                warning: None,
                cpu_override: Some(50.0),
                exit_after: None,
                diagnostics: None,
            },
        );
        app.worker.take();
        app.demo_mode = false;
        app.cpu_override = None;
        app.metrics = sample;
        app
    }

    #[test]
    fn returning_to_strip_clips_retained_floating_ui_allocation() {
        use eframe::App;
        let ctx = egui::Context::default();
        let mut app = app_with_sample(MetricsSample::unavailable("test"));
        let mut frame = eframe::Frame::_new_kittest();
        for [width, height] in [[800.0, 240.0], [274.0, 60.0], [800.0, 240.0], [274.0, 60.0]] {
            app.config.taskbar_mode = height < 85.0;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, height),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                // Model a minimum retained by the larger floating toolbar.
                ui.set_min_size(egui::vec2(800.0, 240.0));
                app.ui(ui, &mut frame);
            });
            output.textures_delta.clear();
            assert!(app.render_surface_size[0] <= width);
            assert!(app.render_surface_size[1] <= height);
            assert!(app.render_surface_size[1] > 0.0);
            assert_eq!(
                app.renderer.width(),
                if app.config.taskbar_mode { 274 } else { WIDTH }
            );
        }
    }

    #[test]
    fn partial_cpu_remains_usable_when_only_memory_failed() {
        let app = app_with_sample(MetricsSample {
            cpu_percent: Some(37.0),
            ram_percent: None,
            used_memory_bytes: 0,
            total_memory_bytes: 0,
            measured_at: Instant::now(),
            status: MetricStatus::Unavailable("RAM failure".into()),
        });
        assert_eq!(app.cpu_percent(), 37.0);
        assert_eq!(app.ram_percent(), None);
        assert_eq!(app.cpu_label(), "CPU 37.0%");
        assert_eq!(app.telemetry_notice(), Some("Leitura parcial"));
    }

    #[test]
    fn memory_remains_usable_during_cpu_warm_up_but_not_after_expiry() {
        let mut app = app_with_sample(MetricsSample {
            cpu_percent: None,
            ram_percent: Some(50.0),
            used_memory_bytes: 512,
            total_memory_bytes: 1024,
            measured_at: Instant::now(),
            status: MetricStatus::WarmingUp,
        });
        assert_eq!(app.ram_percent(), Some(50.0));
        assert_eq!(app.cpu_percent(), 0.0);
        assert_eq!(app.cpu_label(), "Lendo CPU…");
        app.metrics.measured_at = Instant::now() - Duration::from_secs(10);
        assert_eq!(app.ram_percent(), None);
        assert_eq!(app.cpu_label(), "Leitura atrasada");
        app.metrics.measured_at = Instant::now() + Duration::from_secs(10);
        assert!(app.metrics_stale());
    }

    #[test]
    fn demo_worker_never_invokes_real_provider_factory() {
        let mut worker = launch_worker(true, Duration::from_secs(1), 50.0, |_| {
            panic!("the demo must never construct a real telemetry provider")
        })
        .expect("manual worker");
        let deadline = Instant::now() + Duration::from_secs(2);
        let sample = loop {
            if let Some(sample) = worker.poll() {
                break sample;
            }
            assert!(Instant::now() < deadline, "manual sample timed out");
            std::thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(sample.cpu_percent, Some(50.0));
        assert_eq!(sample.ram_percent, None);
        assert_eq!(sample.total_memory_bytes, 0);
        worker.shutdown();
    }
    fn workshop_frame(
        app: &mut super::CpuExpressApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> Vec<egui::epaint::ClippedShape> {
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(760.0, 1000.0),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |ui| app.appearance_controls(ui),
        );
        let shapes = output.shapes.clone();
        output.drop_without_applying_deltas();
        shapes
    }
    fn workshop_click(app: &mut super::CpuExpressApp, ctx: &egui::Context, label: &str) {
        fn find(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::Shape::Text(t) if t.galley.job.text == label => {
                    Some(t.visual_bounding_rect().center())
                }
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| find(s, label)),
                _ => None,
            }
        }
        let shapes = workshop_frame(app, ctx, vec![]);
        let pos = shapes
            .iter()
            .find_map(|s| find(&s.shape, label))
            .expect("visible workshop control");
        for pressed in [true, false] {
            workshop_frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    #[test]
    fn workshop_equips_chosen_slot_and_locked_paint_stays_locked() {
        let ctx = egui::Context::default();
        let mut app = app_with_sample(MetricsSample::unavailable("test"));
        let original = app.config.visual.coaches;
        workshop_click(&mut app, &ctx, "Vagão 2");
        workshop_click(&mut app, &ctx, "Tanque");
        assert_eq!(app.config.visual.coaches[1], super::CoachStyle::Tank);
        assert_eq!(app.config.visual.coaches[0], original[0]);
        assert_eq!(app.config.visual.coaches[2], original[2]);
        assert!(app.equipment_notice.is_some());
        assert!(app.scene_dirty);
        workshop_click(&mut app, &ctx, "Pinturas");
        workshop_click(&mut app, &ctx, "Expresso Noturno");
        assert_eq!(
            app.config.visual.livery,
            crate::simulation::objectives::Livery::Aurora
        );
        app.demo_mode = true;
        workshop_click(&mut app, &ctx, "Expresso Noturno");
        assert_eq!(
            app.config.visual.livery,
            crate::simulation::objectives::Livery::Midnight
        );
    }
}
