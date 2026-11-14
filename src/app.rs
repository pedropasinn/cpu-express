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
    game::panel_frame().show(ui, |ui| {
        game::apply_style(ui);
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(title)
                .size(16.0)
                .strong()
                .color(game::INK),
        );
        ui.separator();
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
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 118.0), egui::Sense::hover());
    let response = ui.interact(rect, ui.scope_id().with(name), egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
    let painter = ui.painter_at(rect);
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id, response.hovered() && unlocked, 0.12);
    let face = if selected {
        egui::Color32::from_rgb(219, 231, 183)
    } else {
        game::CARD
    };
    painter.rect_filled(rect.translate(egui::vec2(0.0, 3.0)), 2.0, game::EDGE);
    painter.rect_filled(rect, 2.0, face);
    painter.rect_stroke(
        rect.shrink(1.0),
        2.0,
        egui::Stroke::new(
            if selected { 3.0 } else { 1.0 + hover },
            if selected { game::GREEN } else { game::EDGE },
        ),
        egui::StrokeKind::Inside,
    );
    let scale = ((width - 20.0) / texture.size()[0] as f32)
        .floor()
        .clamp(1.0, 2.0);
    let size = texture.size_vec2() * scale;
    let image_rect =
        egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 39.0), size);
    painter.image(
        texture.id(),
        image_rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        if unlocked {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_gray(125)
        },
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 73.0),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(13.0),
        game::INK,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 98.0),
        egui::Align2::CENTER_CENTER,
        state,
        egui::FontId::proportional(11.0),
        if selected { game::GREEN } else { game::MUTED },
    );
    game::icon(
        &painter,
        egui::Rect::from_min_size(rect.min + egui::vec2(7.0, 7.0), egui::vec2(16.0, 16.0)),
        if !unlocked {
            Icon::Lock
        } else if selected {
            Icon::Check
        } else {
            Icon::Train
        },
        game::INK,
    );
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(4.0),
            0.0,
            egui::Stroke::new(2.0, game::BLUE),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_text(if unlocked {
        format!("Equipar {name}")
    } else {
        state.to_owned()
    })
}

fn achievement_name(achievement: Achievement) -> &'static str {
    match achievement {
        Achievement::FirstStation => "Primeira estação",
        Achievement::FirstCircuit => "Primeiro circuito",
        Achievement::LongJourney => "Longa jornada",
    }
}

fn coach_name(style: CoachStyle) -> &'static str {
    match style {
        CoachStyle::Passenger => "Passageiros",
        CoachStyle::Sleeper => "Dormitório",
        CoachStyle::Freight => "Carga",
        CoachStyle::Tank => "Tanque",
        CoachStyle::Observation => "Panorâmico",
    }
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
    show_help: bool,
    panel_tab: PanelTab,
    appearance_page: u8,
    journey_page: u8,
    coach_slot: usize,
    stock_gallery: Option<StockGallery>,
    equipment_notice: Option<(String, Instant)>,
    pending_dock: bool,
    applied_window_mode: Option<bool>,
    layout_test_step: Option<u8>,
    render_surface_size: [f32; 2],
    next_taskbar_check: Instant,
    next_panel_save: Instant,
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
        let config = options.config.normalized();
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
        if let Some(path) = &progress_path {
            match crate::progress::load(path) {
                Ok(Some((journey, wait))) => {
                    world.restore_journey(journey);
                    world.set_station_wait(wait);
                }
                Ok(None) => {}
                Err(error) => progress_notice = Some(error),
            }
        }
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
            notification: options.warning,
            preferences_notice: None,
            show_controls: std::env::var_os("CPU_EXPRESS_OPEN_PANEL").is_some(),
            show_help: false,
            panel_tab: PanelTab::default(),
            appearance_page: 0,
            journey_page: 0,
            coach_slot: 0,
            stock_gallery: None,
            equipment_notice: None,
            pending_dock: true,
            applied_window_mode: None,
            layout_test_step: (options.exit_after.is_some()
                && std::env::var_os("CPU_EXPRESS_LAYOUT_TEST").is_some())
            .then_some(0),
            render_surface_size: [0.0; 2],
            next_taskbar_check: opened,
            next_panel_save: opened + Duration::from_secs(2),
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
            local_weather: LocalWeather::new(
                (!demo_mode && options.exit_after.is_none())
                    || std::env::var("CPU_EXPRESS_LIVE_WEATHER").is_ok_and(|v| v == "1"),
            ),
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
            "next_station": journey.next_station.name,
            "segment_progress": journey.segment_progress,
            "completed_laps": journey.completed_laps,
            "stations_visited": journey.stations_visited,
            "dropped_travel_events": journey.dropped_events,
            "native_window": self.native_status,
            "requested_taskbar_mode": self.config.taskbar_mode,
            "render_surface_size": self.render_surface_size,
            "station_wait_seconds": self.world.station_wait(),
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
            self.world
                .advance_expedition(&mut self.train, cpu, dt, &self.config);
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
            }
            self.renderer
                .set_scene_context(Some(SceneContext::from_journey(
                    &self.world.journey_snapshot(),
                )));
            let livery = self.config.visual.livery;
            self.renderer.set_livery(
                if livery.available(self.world.journey_snapshot()) || self.demo_mode {
                    livery
                } else {
                    crate::simulation::objectives::Livery::Aurora
                },
            );
            self.renderer
                .set_local_atmosphere(self.local_weather.atmosphere());
            let image = egui::ColorImage::new(
                [WIDTH, HEIGHT],
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

    fn consume_travel_events(&mut self) {
        while let Some(event) = self.world.pop_travel_event() {
            let message = match event {
                TravelEvent::StationArrived { station, .. } => {
                    Some(format!("Chegada: {}", station.name()))
                }
                TravelEvent::CircuitCompleted { completed_laps, .. } => {
                    Some(format!("Circuito {completed_laps} concluído"))
                }
                TravelEvent::AchievementUnlocked { achievement, .. } => {
                    Some(format!("Conquista: {}", achievement_name(achievement)))
                }
                TravelEvent::StationDeparted { .. } => None,
            };
            if let Some(message) = message {
                if self.travel_log.len() == 32 {
                    self.travel_log.pop_front();
                }
                self.travel_log.push_back(message.clone());
                self.travel_notice = Some((message, Instant::now() + Duration::from_secs(5)));
            }
        }
    }

    fn persist_preferences(&mut self, force: bool) {
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
            self.progress_notice = crate::progress::save(path, &self.world).err();
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
                    journey.next_station.name,
                    journey.segment_progress * 100.0
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
                    egui::vec2(track.width() * journey.segment_progress as f32, 2.0),
                ),
                0.0,
                ACCENT,
            );
            ui.interact(rect, egui::Id::unique("journey-progress"), egui::Sense::hover())
                .on_hover_text(format!("{} até {}\nFaltam {:.0} unidades fictícias.\n{} visitas · {} circuitos. Abra Opções / Viagem para detalhes.",
                    journey.route_segment.from.name(), journey.next_station.name, journey.distance_to_next_station_units, journey.stations_visited, journey.completed_laps));
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

    fn journey_controls(&mut self, ui: &mut egui::Ui) {
        game::apply_style(ui);
        let journey = self.world.journey_snapshot();
        ui.horizontal_wrapped(|ui| {
            for (page, icon, label) in [
                (0, Icon::Map, "Mapa"),
                (1, Icon::Star, "Objetivos"),
                (2, Icon::Book, "Diário"),
            ] {
                if game::button(
                    ui,
                    ("journey-page", page),
                    icon,
                    label,
                    self.journey_page == page,
                    egui::vec2(118.0, 34.0),
                )
                .clicked()
                {
                    self.journey_page = page;
                }
            }
        });
        ui.add_space(8.0);
        if self.journey_page == 0 {
            section(ui, "Circuito Aurora", |ui| {
                let (map, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 180.0),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(map);
                painter.rect_filled(map, 2.0, egui::Color32::from_rgb(198, 213, 169));
                painter.rect_filled(
                    egui::Rect::from_min_max(egui::pos2(map.center().x, map.top()), map.max),
                    2.0,
                    egui::Color32::from_rgb(175, 202, 199),
                );
                let positions =
                    [(0.19, 0.32), (0.67, 0.26), (0.80, 0.73), (0.27, 0.77)].map(|(x, y)| {
                        egui::pos2(map.left() + map.width() * x, map.top() + map.height() * y)
                    });
                for index in 0..4 {
                    let a = positions[index];
                    let b = positions[(index + 1) % 4];
                    painter.line_segment([a, b], egui::Stroke::new(6.0, game::EDGE));
                    painter.line_segment([a, b], egui::Stroke::new(3.0, if index == journey.route_segment.index { game::GOLD } else { game::CARD }));
                }
                for (index, label) in ["Vila Aurora", "Bosque", "Serra Clara", "Porto do Sol"]
                    .iter()
                    .enumerate()
                {
                    let pos = positions[index];
                    painter.circle_filled(pos, 13.0, game::CARD);
                    game::icon(
                        &painter,
                        egui::Rect::from_center_size(pos, egui::vec2(21.0, 21.0)),
                        Icon::Station,
                        if journey.discovered_stations[index] {
                            game::GREEN
                        } else {
                            game::MUTED
                        },
                    );
                    painter.text(
                        pos + egui::vec2(0.0, 18.0),
                        egui::Align2::CENTER_TOP,
                        label,
                        egui::FontId::proportional(12.0),
                        game::INK,
                    );
                }
                let a = positions[journey.route_segment.index];
                let b = positions[(journey.route_segment.index + 1) % 4];
                let train_pos = a + (b - a) * journey.segment_progress as f32;
                painter.circle_filled(train_pos, 12.0, game::GOLD);
                game::icon(
                    &painter,
                    egui::Rect::from_center_size(train_pos, egui::vec2(20.0, 20.0)),
                    Icon::Train,
                    game::INK,
                );
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} até {}",
                        journey.route_segment.from.name(),
                        journey.next_station.name
                    ))
                    .size(17.0)
                    .strong(),
                );
                let status = if self.world.station_wait() > 0.0 {
                    format!(
                        "Embarque · partida em {:.0} s",
                        self.world.station_wait().ceil()
                    )
                } else {
                    format!(
                        "Próxima parada em {:.0} s",
                        (journey.distance_to_next_station_units / 24.0).ceil()
                    )
                };
                game::progress(
                    ui,
                    "route-progress",
                    journey.segment_progress as f32,
                    &status,
                    game::GREEN,
                );
                ui.small(format!(
                    "{} chegadas  ·  {} circuitos completos",
                    journey.stations_visited, journey.completed_laps
                ));
            });
            if game::button(
                ui,
                "open-objectives",
                Icon::Star,
                "Ver objetivos e recompensas",
                false,
                egui::vec2(270.0, 38.0),
            )
            .clicked()
            {
                self.journey_page = 1;
            }
        }
        if self.journey_page == 1 {
            ui.label(
                egui::RichText::new("Caderno de objetivos")
                    .size(20.0)
                    .strong(),
            );
            ui.small("Viaje, descubra estações e conquiste novas pinturas.");
            for objective in crate::simulation::objectives::OBJECTIVES {
                let done = journey.stations_visited >= objective.goal;
                ui.push_id(objective.id, |ui| {
                    section(ui, objective.title, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let (badge, _) = ui
                                .allocate_exact_size(egui::vec2(35.0, 35.0), egui::Sense::hover());
                            game::icon(
                                ui.painter(),
                                badge,
                                if done { Icon::Star } else { Icon::Lock },
                                if done { game::GOLD } else { game::MUTED },
                            );
                            ui.vertical(|ui| {
                                ui.label(objective.detail);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Recompensa: {}",
                                        objective.reward.name()
                                    ))
                                    .strong()
                                    .color(game::BLUE),
                                );
                            });
                        });
                        game::progress(
                            ui,
                            objective.id,
                            (journey.stations_visited as f32 / objective.goal as f32).min(1.0),
                            &format!(
                                "{} / {} chegadas",
                                journey.stations_visited.min(objective.goal),
                                objective.goal
                            ),
                            if done { game::GREEN } else { game::BLUE },
                        );
                        if done {
                            if game::button(
                                ui,
                                (objective.id, "reward"),
                                Icon::Paint,
                                "Ver pintura liberada",
                                false,
                                egui::vec2(210.0, 32.0),
                            )
                            .clicked()
                            {
                                self.panel_tab = PanelTab::Appearance;
                                self.appearance_page = 1;
                            }
                        } else {
                            ui.small("Desbloqueio automático ao completar o objetivo.");
                        }
                    });
                });
            }
        }
        if self.journey_page == 2 {
            ui.separator();
            if let Some(message) = &self.progress_notice {
                ui.colored_label(game::BLUE, message);
            }
            if self.progress_path.is_some() {
                ui.small("Progresso salvo a cada 30 s e ao sair. Sem avanço offline.");
                if ui.button("Salvar viagem agora").clicked() {
                    self.persist_progress(true);
                }
            } else {
                ui.small("Viagem temporária de demonstração/teste.");
            }
            egui::CollapsingHeader::new("Diário da viagem").show(ui, |ui| {
                if self.travel_log.is_empty() {
                    ui.label("As próximas chegadas e conquistas aparecerão aqui.");
                }
                for message in self.travel_log.iter().rev() {
                    ui.label(message);
                }
            });
            egui::CollapsingHeader::new("Estações e conquistas").show(ui, |ui| {
                let journey = self.world.journey_snapshot();
                ui.label(
                    egui::RichText::new(journey.route_name)
                        .strong()
                        .color(game::BLUE),
                );
                ui.label(format!(
                    "{} até {}",
                    journey.route_segment.from.name(),
                    journey.next_station.name
                ));
                ui.add(
                    egui::ProgressBar::new(journey.segment_progress as f32).text(format!(
                        "Trecho {:.0}% · faltam {:.0} un.",
                        journey.segment_progress * 100.0,
                        journey.distance_to_next_station_units
                    )),
                );
                ui.label(format!(
                    "{} visitas · {} circuitos · {:.0} un. percorridas",
                    journey.stations_visited, journey.completed_laps, journey.total_distance_units
                ));
                ui.horizontal_wrapped(|ui| {
                    for station in self.world.journey().route().stations() {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {}",
                                if journey.discovered_stations[station.id.index()] {
                                    "[visitada]"
                                } else {
                                    "[ ]"
                                },
                                station.name
                            ))
                            .size(10.5)
                            .color(
                                if journey.discovered_stations[station.id.index()] {
                                    ACCENT
                                } else {
                                    game::INK
                                },
                            ),
                        );
                    }
                });
                for achievement in [
                    Achievement::FirstStation,
                    Achievement::FirstCircuit,
                    Achievement::LongJourney,
                ] {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            if journey.achievements.contains(achievement) {
                                "[x]"
                            } else {
                                "[ ]"
                            },
                            achievement_name(achievement)
                        ))
                        .size(10.5),
                    );
                }
                ui.label(
                egui::RichText::new(
                    "Chegadas liberam pinturas. Pausar não perde progresso; não há avanço offline.",
                )
                .size(10.5),
            );
            });
        }
    }

    fn scene_preview(&self, ui: &mut egui::Ui, compact: bool) {
        let ppp = ui.ctx().pixels_per_point();
        let scale = (ui.available_width() * ppp / WIDTH as f32)
            .floor()
            .clamp(1.0, if compact { 1.0 } else { 3.0 });
        let size = egui::vec2(WIDTH as f32, HEIGHT as f32) * scale / ppp;
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
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        ui.add_space(8.0);
    }

    fn compact_drive_controls(&mut self, ui: &mut egui::Ui) {
        if self.demo_mode {
            ui.label(
                egui::RichText::new("DEMO · CPU simulada; RAM indisponível")
                    .size(11.0)
                    .color(ACCENT),
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
            ui.label(egui::RichText::new(notice).color(ACCENT));
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
        section(ui, "Próxima parada", |ui| {
            ui.label(
                egui::RichText::new(journey.next_station.name)
                    .size(19.0)
                    .color(game::BLUE),
            );
            game::progress(
                ui,
                "next-stop",
                journey.segment_progress as f32,
                &format!("Trecho · {:.0}%", journey.segment_progress * 100.0),
                game::GREEN,
            );
            ui.label(format!(
                "{} visitas  ·  {} circuitos",
                journey.stations_visited, journey.completed_laps
            ));
            ui.small(
                "A CPU muda o ritmo visual. A viagem progride igualmente em qualquer computador.",
            );
        });
        if game::button(
            ui,
            "drive-toggle",
            if self.paused { Icon::Play } else { Icon::Pause },
            if self.paused {
                "Retomar viagem"
            } else {
                "Pausar viagem"
            },
            true,
            egui::vec2(ui.available_width(), 44.0),
        )
        .clicked()
        {
            self.dispatch(AppAction::TogglePause, ui.ctx());
        }
        ui.label(
            egui::RichText::new("Espaço também pausa e retoma. Seu progresso fica preservado.")
                .size(11.0)
                .color(game::MUTED),
        );
        ui.add_space(8.0);
        self.scene_preview(ui, false);
        let speed = if self.suspended() { "Parado".to_owned() } else { format!("{:.0} km/h", self.train.speed_kmh()) };
        let cpu = if self.demo_mode
            || self.cpu_override.is_some()
            || (self.metrics.cpu_percent.is_some() && !self.metrics_stale())
        {
            format!("{:.0}%", self.cpu_percent())
        } else {
            "—".to_owned()
        };
        if self.demo_mode { ui.small("DEMO · CPU simulada · memória não consultada"); }
        let ram = self
            .ram_percent()
            .map_or_else(|| "—".to_owned(), |value| format!("{value:.0}%"));
        ui.columns(3, |columns| {
            for (column, (label, value)) in columns.iter_mut().zip([
                ("Velocidade", speed),
                ("Processador", cpu),
                ("Memória", ram),
            ]) {
                egui::Frame::new()
                    .fill(game::CARD)
                    .corner_radius(6)
                    .inner_margin(10)
                    .show(column, |ui| {
                        ui.set_width(ui.available_width());
                        let (glyph, _) =
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                        game::icon(
                            ui.painter(),
                            glyph,
                            match label {
                                "Processador" => Icon::Cpu,
                                "Memória" => Icon::Memory,
                                _ => Icon::Train,
                            },
                            game::BLUE,
                        );
                        ui.label(egui::RichText::new(label).size(11.0).color(game::MUTED));
                        ui.label(
                            egui::RichText::new(value)
                                .size(18.0)
                                .strong()
                                .color(game::INK),
                        );
                    });
            }
        });
        ui.add_space(12.0);
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

    fn appearance_controls(&mut self, ui: &mut egui::Ui) {
        game::apply_style(ui);
        if self.stock_gallery.is_none() {
            self.stock_gallery = Some(StockGallery::new(ui.ctx()));
        }
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            for (page, icon, label) in [
                (0, Icon::Train, "Vagões"),
                (1, Icon::Paint, "Pinturas"),
                (2, Icon::Weather, "Clima"),
                (3, Icon::Settings, "Fluidez"),
            ] {
                if game::button(
                    ui,
                    ("workshop-page", page),
                    icon,
                    label,
                    self.appearance_page == page,
                    egui::vec2(100.0, 34.0),
                )
                .clicked()
                {
                    self.appearance_page = page;
                }
            }
        });
        ui.add_space(8.0);
        let journey = self.world.journey_snapshot();
        match self.appearance_page {
            0 => {
                section(ui, "Monte sua composição", |ui| {
                    ui.small("Escolha uma posição e equipe um vagão da coleção.");
                    ui.horizontal_wrapped(|ui| {
                        for slot in 0..3 {
                            if game::button(
                                ui,
                                ("slot", slot),
                                Icon::Train,
                                &format!("Vagão {}", slot + 1),
                                self.coach_slot == slot,
                                egui::vec2(110.0, 36.0),
                            )
                            .clicked()
                            {
                                self.coach_slot = slot;
                            }
                        }
                    });
                    ui.small("Ordem: cauda, vagões 1 / 2 / 3, locomotiva");
                });
                let choices = [
                    CoachStyle::Passenger,
                    CoachStyle::Observation,
                    CoachStyle::Sleeper,
                    CoachStyle::Freight,
                    CoachStyle::Tank,
                ];
                let count = if ui.available_width() >= 600.0 { 3 } else { 2 };
                for pair in choices.chunks(count) {
                    ui.columns(count, |columns| {
                        for (column, &style) in columns.iter_mut().zip(pair) {
                            let selected = self.config.visual.coaches[self.coach_slot] == style;
                            let texture = self.stock_gallery.as_ref().unwrap().coach(style);
                            let response = stock_card(
                                column,
                                texture,
                                coach_name(style),
                                if selected {
                                    "EQUIPADO"
                                } else {
                                    "Equipar neste vagão"
                                },
                                selected,
                                true,
                            );
                            if response.clicked() {
                                self.config.visual.coaches[self.coach_slot] = style;
                                self.equipment_notice = Some((
                                    format!(
                                        "{} equipado no vagão {}",
                                        coach_name(style),
                                        self.coach_slot + 1
                                    ),
                                    Instant::now() + Duration::from_secs(3),
                                ));
                                changed = true;
                            }
                        }
                    });
                    ui.add_space(10.0);
                }
                ui.small("Modelos visuais: personalize o trem sem alterar o ritmo dos objetivos.");
            }
            1 => {
                ui.label(
                    egui::RichText::new("Pinturas da locomotiva")
                        .size(18.0)
                        .strong(),
                );
                ui.small("Cada chegada aproxima você de uma nova pintura.");
                for pair in crate::simulation::objectives::Livery::ALL.chunks(2) {
                    ui.columns(2, |columns| {
                        for (column, &livery) in columns.iter_mut().zip(pair) {
                            let unlocked = self.demo_mode || livery.available(journey);
                            let selected = self.config.visual.livery == livery;
                            let state = if !unlocked {
                                format!(
                                    "{} / {} chegadas",
                                    journey.stations_visited,
                                    livery.required_stops()
                                )
                            } else if selected {
                                "EQUIPADA".into()
                            } else {
                                "Equipar pintura".into()
                            };
                            let texture = self.stock_gallery.as_ref().unwrap().livery(livery);
                            let response = stock_card(
                                column,
                                texture,
                                livery.name(),
                                &state,
                                selected,
                                unlocked,
                            );
                            if response.clicked() && unlocked {
                                self.config.visual.livery = livery;
                                self.equipment_notice = Some((
                                    format!("Pintura {} equipada", livery.name()),
                                    Instant::now() + Duration::from_secs(3),
                                ));
                                changed = true;
                            }
                        }
                    });
                    ui.add_space(10.0);
                }
            }
            2 => {
                section(ui, "Paisagem e atmosfera", |ui| {
                    ui.strong("São Paulo, SP · Brasil");
                    ui.small("Paisagem completa · clima e iluminação locais");
                    if let Some(snapshot) = &self.local_weather.last_good {
                        ui.label(format!(
                            "{} · {:.1} °C · {}",
                            snapshot.description(),
                            snapshot.temperature_c,
                            if snapshot.is_day { "Dia" } else { "Noite" }
                        ));
                        ui.small(format!(
                            "Condição das {} (São Paulo) · recebida há {} min",
                            snapshot.local_time(),
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                                .saturating_sub(snapshot.received_unix)
                                / 60
                        ));
                        if snapshot.stale() {
                            ui.colored_label(game::BLUE, "Dados antigos; aguardando atualização. O cenário conserva a última condição.");
                        }
                    } else if self.local_weather.enabled {
                        ui.label("Cenário ilustrativo até receber o clima local.");
                    } else {
                        ui.label(
                            "Demonstração/teste: atmosfera ilustrativa, sem consulta à internet.",
                        );
                    }
                    if self.local_weather.loading() {
                        ui.label("Consultando clima…");
                    }
                    if let Some(error) = &self.local_weather.error {
                        ui.colored_label(
                            game::BLUE,
                            "Clima indisponível; nova tentativa automática.",
                        );
                        ui.small(error);
                    }
                    if self.local_weather.enabled {
                        if ui.add_enabled(self.local_weather.can_refresh(), egui::Button::new("Atualizar clima"))
                    .on_hover_text("Atualização automática a cada 15 minutos. Atualização manual: intervalo mínimo de 1 minuto.")
                    .clicked() { self.local_weather.refresh(ui.ctx()); }
                        ui.small("Condições estimadas para a cidade; podem variar no seu bairro.");
                    }
                    ui.hyperlink_to("Dados: Open-Meteo · CC BY 4.0", "https://open-meteo.com/");
                });
            }
            _ => {
                section(ui, "Fluidez", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        changed |= ui
                            .selectable_value(&mut self.config.fps, 60, "Fluido · 60 FPS")
                            .changed();
                        changed |= ui
                            .selectable_value(&mut self.config.fps, 30, "Econômico · 30 FPS")
                            .changed();
                    });
                    ui.small(
                        "Apenas a animação muda: objetivos e recompensas mantêm o mesmo ritmo.",
                    );
                    egui::CollapsingHeader::new("Ajuste fino").show(ui, |ui| {
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.config.fps, 10..=60)
                                    .text("Limite de FPS"),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.config.smoothing_k, 0.05..=20.0)
                                    .logarithmic(true)
                                    .text("Resposta à CPU"),
                            )
                            .changed();
                    });
                });
                section(ui, "Conforto e atenção", |ui| {
                    changed |= ui.checkbox(&mut self.config.reduce_menu_motion, "Reduzir animações dos menus").changed();
                    changed |= ui.checkbox(&mut self.config.quiet_mode, "Trabalho tranquilo: ocultar avisos sobre a paisagem").changed();
                    ui.small("A arte completa e o progresso da viagem são preservados. Consulte as chegadas no diário a qualquer momento.");
                });
                egui::CollapsingHeader::new("Informações na janela flutuante").show(ui, |ui| {
                    let mut mode = self.config.effective_hud_mode();
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(&mut mode, HudMode::Compact, "Compacto");
                        ui.selectable_value(&mut mode, HudMode::Detailed, "Completo");
                        ui.selectable_value(&mut mode, HudMode::Clean, "Limpo");
                    });
                    if mode != self.config.effective_hud_mode() {
                        self.config.set_hud_mode(mode);
                        changed = true;
                    }
                    changed |= ui
                        .checkbox(
                            &mut self.config.auto_hide_controls,
                            "Recolher controles sem mouse e sem foco",
                        )
                        .changed();
                });
            }
        }
        if changed {
            self.preferences_changed();
            self.next_frame = Instant::now();
            ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
        }
    }

    fn behavior_controls(&mut self, ui: &mut egui::Ui) {
        let mut placement_changed = ui
            .checkbox(&mut self.config.taskbar_mode, "Na área da barra de tarefas")
            .changed();
        if self.config.taskbar_mode {
            let bar_width = self.native_status["taskbar_width"].as_i64().unwrap_or(1920) as i32;
            let bar_height = self.native_status["taskbar_height"].as_i64().unwrap_or(60) as i32;
            placement_changed |= ui
                .add(
                    egui::Slider::new(&mut self.config.taskbar_offset, 0..=bar_width.max(160))
                        .text("Posição horizontal (px)"),
                )
                .changed();
            let mut automatic = self.config.taskbar_width == 0 && self.config.taskbar_height == 0;
            if ui
                .checkbox(&mut automatic, "Tamanho automático da barra")
                .changed()
            {
                self.config.taskbar_width = if automatic {
                    0
                } else {
                    (bar_height * 32 / 7).max(160)
                };
                self.config.taskbar_height = if automatic {
                    0
                } else {
                    bar_height.clamp(24, 240)
                };
                placement_changed = true;
            }
            if !automatic {
                placement_changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut self.config.taskbar_width,
                            160..=bar_width.clamp(160, 2400),
                        )
                        .text("Largura (px)"),
                    )
                    .changed();
                placement_changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut self.config.taskbar_height,
                            24..=bar_height.clamp(24, 240),
                        )
                        .text("Altura (px)"),
                    )
                    .changed();
            }
            ui.small("A faixa fica dentro da barra. Para aumentar além dessa altura ou mover livremente, desmarque o modo da barra.");
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
            ui.small("Arraste pelo título da janela e redimensione pelas bordas. Posição e tamanho ficam salvos.");
        }
        if ui.button("Restaurar posição e tamanho").clicked() {
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
        ui.small("Clique na faixa para abrir o painel; botão direito pausa.");
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
            egui::RichText::new("A coleta continua durante a pausa; a paisagem fica congelada.")
                .size(11.0),
        );
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
            return;
        }
        if ctx.embed_viewports() {
            self.overlay(ctx, bounds);
            return;
        }
        let builder = egui::ViewportBuilder::default()
            .with_title(platform::PANEL_TITLE)
            .with_inner_size([760.0, 720.0])
            .with_min_inner_size([480.0, 480.0])
            .with_position(ctx.input(|i| {
                i.viewport()
                    .outer_rect
                    .map_or(egui::pos2(24.0, 80.0), |rect| {
                        egui::pos2(rect.left().max(8.0), (rect.top() - 760.0).max(8.0))
                    })
            }))
            .with_resizable(true)
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
                }
                game::apply_style(root_ui);
                egui::Frame::new()
                    .fill(game::EDGE)
                    .inner_margin(6)
                    .show(root_ui, |ui| {
                        game::panel_frame().show(ui, |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                            let compact = ui.available_height() < 610.0 || ui.available_width() < 560.0;
                            let (title, subtitle, emblem) = if self.show_help {
                                (
                                    "Manual do maquinista",
                                    "Um guia para sua ferrovia de desktop",
                                    Icon::Help,
                                )
                            } else {
                                match self.panel_tab {
                                    PanelTab::Drive => (
                                        "Cabine do maquinista",
                                        "Acompanhe seu trem e o ritmo da viagem",
                                        Icon::Train,
                                    ),
                                    PanelTab::Journey => (
                                        "Diário de viagem",
                                        "Estações, objetivos e novas descobertas",
                                        Icon::Book,
                                    ),
                                    PanelTab::Appearance => (
                                        "Oficina ferroviária",
                                        "Monte um trem com a sua personalidade",
                                        Icon::Paint,
                                    ),
                                    PanelTab::Behavior => (
                                        "Ajustes da ferrovia",
                                        "Posicione sua companhia de viagem",
                                        Icon::Settings,
                                    ),
                                }
                            };
                            ui.horizontal(|ui| {
                                let (crest, _) = ui.allocate_exact_size(
                                    egui::vec2(if compact { 30.0 } else { 44.0 }, if compact { 32.0 } else { 52.0 }),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(crest, 2.0, game::GOLD);
                                game::icon(ui.painter(), crest.shrink(7.0), emblem, game::INK);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("CPU EXPRESS  /  CIRCUITO AURORA")
                                            .monospace()
                                            .size(10.0)
                                            .color(game::MUTED),
                                    );
                                    ui.label(
                                        egui::RichText::new(title)
                                            .size(if compact { 18.0 } else { 24.0 })
                                            .strong()
                                            .color(game::INK),
                                    );
                                    if !compact { ui.label(egui::RichText::new(subtitle).size(12.0).color(game::MUTED)); }
                                });
                            });
                            ui.add_space(3.0);
                            let tab_width = ((ui.available_width() - 64.0) / 5.0).floor().max(64.0);
                            ui.horizontal_wrapped(|ui| {
                                for (tab, label, icon) in [
                                    (PanelTab::Drive, "Trem", Icon::Train),
                                    (PanelTab::Journey, "Viagem", Icon::Map),
                                    (PanelTab::Appearance, "Visual", Icon::Paint),
                                    (PanelTab::Behavior, "Janela", Icon::Settings),
                                ] {
                                    if game::button(
                                        ui,
                                        ("nav", tab as u8),
                                        icon,
                                        label,
                                        self.panel_tab == tab && !self.show_help,
                                        egui::vec2(tab_width, if compact { 46.0 } else { 56.0 }),
                                    )
                                    .clicked()
                                    {
                                        self.panel_tab = tab;
                                        self.show_help = false;
                                    }
                                }
                                if game::button(
                                    ui,
                                    "help-nav",
                                    Icon::Help,
                                    "Ajuda",
                                    self.show_help,
                                    egui::vec2(tab_width, if compact { 46.0 } else { 56.0 }),
                                )
                                .clicked()
                                {
                                    self.show_help = !self.show_help;
                                }
                            });
                            ui.add_space(4.0);
                            let content_height = (ui.available_height() - 76.0).max(90.0);
                            egui::ScrollArea::vertical()
                                .id_salt((
                                    "game-panel-content",
                                    self.panel_tab as u8,
                                    self.appearance_page,
                                    self.journey_page,
                                    self.show_help,
                                ))
                                .max_height(content_height)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    if self.show_help {
                                        self.help_contents(ui);
                                    } else {
                                        match self.panel_tab {
                                            PanelTab::Drive => self.drive_controls(ui),
                                            PanelTab::Journey => self.journey_controls(ui),
                                            PanelTab::Appearance => self.appearance_controls(ui),
                                            PanelTab::Behavior => {
                                                section(ui, "Posição e tamanho", |ui| {
                                                    self.behavior_controls(ui)
                                                });
                                            }
                                        }
                                    }
                                    if let Some(message) = self.notification_message() {
                                        ui.colored_label(game::BLUE, message);
                                    }
                                    if let Some(id) = ui.memory(|memory| memory.focused())
                                        && let Some(response) = ui.ctx().read_response(id)
                                        && response.gained_focus()
                                        && ui.min_rect().intersects(response.rect)
                                    {
                                        response.scroll_to_me(Some(egui::Align::Center));
                                    }
                                });
                            ui.separator();
                            let toast = self
                                .equipment_notice
                                .as_ref()
                                .filter(|(_, until)| Instant::now() < *until)
                                .or_else(|| {
                                    self.travel_notice
                                        .as_ref()
                                        .filter(|(_, until)| Instant::now() < *until)
                                });
                            let message = if let Some((text, until)) = toast {
                                ctx.request_repaint_after(
                                    until.saturating_duration_since(Instant::now()),
                                );
                                text.as_str()
                            } else if self.preferences_dirty {
                                "Salvando ajustes…"
                            } else if self.paused {
                                "Viagem pausada · seu progresso está preservado"
                            } else if self.demo_mode {
                                "Demonstração · viagem temporária"
                            } else {
                                "Sua ferrovia, no seu ritmo"
                            };
                            ui.label(
                                egui::RichText::new(message)
                                    .size(12.0)
                                    .strong()
                                    .color(game::GREEN),
                            );
                            ui.horizontal(|ui| {
                                let journey = self.world.journey_snapshot();
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} chegadas · {} voltas",
                                        journey.stations_visited, journey.completed_laps
                                    ))
                                    .size(11.0)
                                    .color(game::MUTED),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if game::button(
                                            ui,
                                            "close-book",
                                            Icon::Close,
                                            "Fechar painel",
                                            false,
                                            egui::vec2(144.0, 30.0),
                                        )
                                        .clicked()
                                        {
                                            self.dispatch(AppAction::ClosePanel, ctx);
                                        }
                                        if ui
                                            .small_button("Sair")
                                            .on_hover_text("Salvar e encerrar CPU Express")
                                            .clicked()
                                        {
                                            self.dispatch(AppAction::Quit, ctx);
                                        }
                                    },
                                );
                            });
                        });
                    });
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
        if self.pending_dock && Instant::now() >= self.next_taskbar_check {
            self.pending_dock = false;
            #[cfg(windows)]
            let result = if self.config.taskbar_mode && self.desktop.is_some() {
                platform::embed_taskbar(
                    frame,
                    self.config.taskbar_offset,
                    self.config.taskbar_width,
                    self.config.taskbar_height,
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
            && !self.pending_dock
            && self.desktop.is_some()
            && Instant::now() >= self.next_taskbar_check
        {
            self.next_taskbar_check = Instant::now() + Duration::from_secs(2);
            if let Err(error) = platform::embed_taskbar(
                frame,
                self.config.taskbar_offset,
                self.config.taskbar_width,
                self.config.taskbar_height,
            ) {
                self.config.taskbar_mode = false;
                self.pending_dock = true;
                self.next_taskbar_check = Instant::now();
                self.add_notice(error);
            }
            self.native_status = platform::taskbar_status(frame).unwrap_or(serde_json::Value::Null);
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
            let physical_scale = ((bounds.height() * pixels_per_point / HEIGHT as f32)
                .min(bounds.width() * pixels_per_point / WIDTH as f32))
            .floor()
            .max(1.0);
            let scale = physical_scale / pixels_per_point;
            let size = egui::vec2(WIDTH as f32 * scale, HEIGHT as f32 * scale);
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
        if bounds.height() < 85.0 {
            let response = ui.interact(
                bounds,
                egui::Id::unique("taskbar-train"),
                egui::Sense::CLICK,
            );
            if response.clicked() {
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
