//! Opt-in eframe adapter. Requests are queued and executed together at the
//! beginning of `App::logic`, including while the viewport is hidden.
//!
//! Keep `ViewportBuilder::with_active(false)` as in the baseline. Visibility
//! uses winit's cache as well as Win32; topmost uses egui's viewport command.
//! Do not issue competing Visible/Minimized/WindowLevel commands elsewhere.

use std::{collections::VecDeque, marker::PhantomData, rc::Rc};

use eframe::egui;

use super::{DesktopAction, DesktopState, DockEdge, SystemTray, TrayEvent, windows};

const MAX_PENDING: usize = 64;

#[derive(Clone, Copy, Debug)]
enum Command {
    Visibility(DesktopAction),
    Dock(DockEdge),
    Topmost(bool),
    SurfaceTopmost(bool),
    ToggleTopmost,
    RecoverPlacement,
    DisableTray,
}

/// Result of one UI-thread update. Errors are recoverable user-facing messages.
#[derive(Debug, Default)]
pub struct DesktopUpdate {
    pub state: DesktopState,
    /// A preference change to persist in Config; application to a minimized
    /// window is deferred until restoration and to egui's next command flush.
    pub topmost_changed: Option<bool>,
    pub placement_changed: bool,
    pub exit_requested: bool,
    pub errors: Vec<String>,
}

/// Controls only the current eframe viewport, never owns its HWND.
///
/// Construct on the UI thread. Call `update(frame)` before any early return in
/// `App::logic`. Calls such as `hide` enqueue intentions, not native success;
/// inspect `DesktopUpdate` after the next logic pass. This ordering prevents a
/// pending topmost update from accidentally restoring a minimized window.
pub struct DesktopController {
    context: egui::Context,
    tray: Option<SystemTray>,
    pending: VecDeque<Command>,
    state: DesktopState,
    has_been_shown: bool,
    topmost: bool,
    topmost_dirty: bool,
    dock_edge: DockEdge,
    // A desktop controller belongs to one UI thread, even though the tray has
    // its own worker. Do not make HWND ownership a Send/Sync assumption.
    _ui_thread: PhantomData<Rc<()>>,
}

impl DesktopController {
    /// No thread or native window is created here. The initial topmost value
    /// must match NativeOptions; tray activation is explicit and fallible.
    #[must_use]
    pub fn new(context: &egui::Context, always_on_top: bool) -> Self {
        Self {
            context: context.clone(),
            tray: None,
            pending: VecDeque::new(),
            state: DesktopState::Visible,
            has_been_shown: false,
            topmost: always_on_top,
            topmost_dirty: false,
            dock_edge: DockEdge::Auto,
            _ui_thread: PhantomData,
        }
    }

    /// Try once to create a tray. Failure leaves the normal taskbar window
    /// usable; callers should display the error and leave hiding disabled.
    pub fn enable_tray(&mut self) -> Result<(), String> {
        self.ensure_open()?;
        if self.tray_available() {
            return Ok(());
        }
        self.stop_tray()?;
        let context = self.context.clone();
        self.tray = Some(SystemTray::new(move || context.request_repaint())?);
        Ok(())
    }

    #[must_use]
    pub fn tray_available(&self) -> bool {
        self.tray.as_ref().is_some_and(SystemTray::is_available)
    }

    /// Last observed state, refreshed in `update`. An external taskbar restore
    /// or minimize is reflected on the next UI logic pass.
    #[must_use]
    pub fn state(&self) -> DesktopState {
        self.state
    }

    /// Whether simulation and rasterization should stop in this state. Unlike
    /// checking `HiddenByUser` alone, this permits eframe's initial hidden paint.
    /// Query after `update(frame)`; tray callbacks continue waking logic.
    #[must_use]
    pub fn animation_suspended(&self) -> bool {
        self.state.suspends_animation(self.has_been_shown)
    }

    #[must_use]
    pub fn always_on_top(&self) -> bool {
        self.topmost
    }

    pub fn show(&mut self) -> Result<(), String> {
        self.enqueue(Command::Visibility(DesktopAction::Show))
    }

    pub fn hide(&mut self) -> Result<(), String> {
        self.state
            .after(DesktopAction::Hide, self.tray_available())
            .map_err(|error| error.to_string())?;
        self.enqueue(Command::Visibility(DesktopAction::Hide))
    }

    pub fn minimize(&mut self) -> Result<(), String> {
        self.enqueue(Command::Visibility(DesktopAction::Minimize))
    }

    /// Reposition explicitly beside the last selected edge (Auto initially).
    pub fn dock(&mut self) -> Result<(), String> {
        self.enqueue(Command::Dock(self.dock_edge))
    }

    pub fn dock_at(&mut self, edge: DockEdge) -> Result<(), String> {
        self.enqueue(Command::Dock(edge))
    }

    /// A faixa usa a mesma autoridade de topo das demais ações nativas.
    pub fn place_strip(
        &mut self,
        frame: &eframe::Frame,
        offset: i32,
        width: i32,
        height: i32,
        fit_height: bool,
        rounded: bool,
    ) -> Result<(), String> {
        windows::place_strip(frame, offset, width, height, fit_height, rounded)
    }

    pub fn set_topmost(&mut self, topmost: bool) -> Result<(), String> {
        self.enqueue(Command::Topmost(topmost))
    }

    /// Política temporária da superfície; não altera a preferência do modo flutuante.
    pub fn set_surface_topmost(&mut self, topmost: bool) -> Result<(), String> {
        self.enqueue(Command::SurfaceTopmost(topmost))
    }

    /// Call after a known viewport DPI/monitor event. It only moves a window
    /// that no longer fits; no timer or repeated monitor polling is installed.
    pub fn display_changed(&mut self) -> Result<(), String> {
        self.enqueue(Command::RecoverPlacement)
    }

    /// Removing the last recovery path first shows a hidden window. A native
    /// restore failure keeps the tray so the user can try again.
    pub fn disable_tray(&mut self) -> Result<(), String> {
        self.enqueue(Command::DisableTray)
    }

    /// Closing is terminal and distinct from hiding. It takes priority over a
    /// full request queue and sends eframe's real Close command in `update`.
    pub fn close(&mut self) -> Result<(), String> {
        self.enqueue(Command::Visibility(DesktopAction::Close))
    }

    fn ensure_open(&self) -> Result<(), String> {
        self.state
            .after(DesktopAction::Show, self.tray_available())
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn enqueue(&mut self, command: Command) -> Result<(), String> {
        self.ensure_open()?;
        if matches!(command, Command::Visibility(DesktopAction::Close)) {
            self.pending.clear();
        } else if self.pending.len() >= MAX_PENDING {
            return Err("Há muitas ações de desktop pendentes; tente novamente.".to_owned());
        }
        self.pending.push_back(command);
        self.context.request_repaint();
        Ok(())
    }

    /// Drain bounded event work on the UI thread, then apply topmost only if
    /// this pass leaves the window visible or hidden, never minimized.
    pub fn update(&mut self, frame: &eframe::Frame) -> DesktopUpdate {
        let mut output = DesktopUpdate {
            state: self.state,
            ..Default::default()
        };
        if self.state == DesktopState::Closed {
            return output;
        }
        match windows::window_state(frame) {
            Ok(state) => {
                self.state = state;
                self.has_been_shown |=
                    matches!(state, DesktopState::Visible | DesktopState::Minimized);
            }
            Err(error) => {
                output.errors.push(error);
                return output;
            }
        }

        // Events wake App::logic even without rendering a hidden viewport.
        // Bounded draining prevents a noisy message source starving the UI.
        for _ in 0..MAX_PENDING {
            let Some(event) = self.tray.as_ref().and_then(SystemTray::poll) else {
                break;
            };
            let command = match event {
                TrayEvent::Show => Command::Visibility(DesktopAction::Show),
                TrayEvent::Hide => Command::Visibility(DesktopAction::Hide),
                TrayEvent::Dock => Command::Dock(self.dock_edge),
                TrayEvent::ToggleTopmost => Command::ToggleTopmost,
                TrayEvent::Exit => Command::Visibility(DesktopAction::Close),
                TrayEvent::DisplayChanged => Command::RecoverPlacement,
                TrayEvent::Unavailable(error) => {
                    output.errors.push(error);
                    if self.has_been_shown
                        && self.state == DesktopState::HiddenByUser
                        && let Err(error) = self.apply_visibility(frame, DesktopAction::Show)
                    {
                        output.errors.push(error);
                    }
                    continue;
                }
            };
            if let Err(error) = self.enqueue(command) {
                output.errors.push(error);
            }
        }

        while let Some(command) = self.pending.pop_front() {
            let result = match command {
                Command::Visibility(DesktopAction::Close) => {
                    // Even a tray cleanup error must not turn Exit into Hide.
                    let result = self.shutdown();
                    self.context.send_viewport_cmd(egui::ViewportCommand::Close);
                    output.exit_requested = true;
                    result
                }
                Command::Visibility(action) => self.apply_visibility(frame, action),
                Command::Dock(edge) => {
                    let result = windows::dock_window_at_edge(frame, edge);
                    if result.is_ok() {
                        self.dock_edge = edge;
                        output.placement_changed = true;
                    }
                    result
                }
                Command::Topmost(topmost) => {
                    self.topmost = topmost;
                    self.topmost_dirty = true;
                    output.topmost_changed = Some(topmost);
                    Ok(())
                }
                Command::SurfaceTopmost(topmost) => {
                    self.topmost = topmost;
                    self.topmost_dirty = true;
                    Ok(())
                }
                Command::ToggleTopmost => {
                    self.topmost = !self.topmost;
                    self.topmost_dirty = true;
                    output.topmost_changed = Some(self.topmost);
                    Ok(())
                }
                Command::RecoverPlacement => {
                    windows::recover_window_placement(frame).map(|changed| {
                        output.placement_changed |= changed;
                    })
                }
                Command::DisableTray => self.remove_tray(frame),
            };
            if let Err(error) = result {
                output.errors.push(error);
            }
            if self.state == DesktopState::Closed {
                break;
            }
        }

        // This is a cheap local health flag, not a shell polling loop. Explorer
        // failure without a subsequent notification cannot be detected here.
        // eframe initially hides the viewport until its first painted frame.
        // Do not mistake that anti-flash startup stage for a user-hidden window.
        if self.has_been_shown
            && self.state.needs_recovery(self.tray_available())
            && let Err(error) = self.apply_visibility(frame, DesktopAction::Show)
        {
            output.errors.push(error);
        }
        if self.topmost_dirty
            && matches!(
                self.state,
                DesktopState::Visible | DesktopState::HiddenByUser
            )
        {
            // eframe updates winit's WindowFlags cache too. Native-only changes
            // would be overwritten by a later visibility/style operation.
            // This must be LAST: winit 0.30 can restore a minimized window when
            // applying other flags. Keep .with_active(false) at window creation.
            self.context
                .send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.topmost {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }));
            self.topmost_dirty = false;
        }
        output.state = self.state;
        output
    }

    fn apply_visibility(
        &mut self,
        frame: &eframe::Frame,
        action: DesktopAction,
    ) -> Result<(), String> {
        let next = self
            .state
            .after(action, self.tray_available())
            .map_err(|error| error.to_string())?;
        match action {
            DesktopAction::Show => {
                windows::show_window(frame)?;
                self.state = next;
                self.has_been_shown = true;
                // A detached monitor must not leave a restored window outside
                // the desktop. Invalid geometry reports an error without panic.
                windows::recover_window_placement(frame)?;
            }
            DesktopAction::Hide => {
                windows::hide_window(frame)?;
                self.state = next;
                // An explicit hide establishes recovery intent even if requested
                // during startup, before the first normally visible frame.
                self.has_been_shown = true;
                // Close the small race with a detected tray failure while the
                // visibility operation was in progress; no guarantee is made
                // about an Explorer crash that has not sent a notification.
                if !self.tray_available() {
                    windows::show_window(frame)?;
                    self.state = DesktopState::Visible;
                    return Err("A bandeja ficou indisponível; a janela foi reaberta.".to_owned());
                }
            }
            DesktopAction::Minimize => {
                if self.state == DesktopState::HiddenByUser {
                    windows::show_window(frame)?;
                    self.state = DesktopState::Visible;
                }
                windows::minimize_window(frame)?;
                self.state = next;
                self.has_been_shown = true;
            }
            DesktopAction::Close => {
                return Err("Use close() para encerrar o loop eframe.".to_owned());
            }
        }
        Ok(())
    }

    fn remove_tray(&mut self, frame: &eframe::Frame) -> Result<(), String> {
        if self.state == DesktopState::HiddenByUser {
            self.apply_visibility(frame, DesktopAction::Show)?;
        }
        self.stop_tray()
    }

    fn stop_tray(&mut self) -> Result<(), String> {
        if let Some(mut tray) = self.tray.take()
            && let Err(error) = tray.shutdown()
        {
            // SystemTray keeps its JoinHandle on signaling failure. Preserve
            // the object too, so a live app can report the failure and retry.
            self.tray = Some(tray);
            return Err(error);
        }
        Ok(())
    }

    /// Release the icon/helper/thread during actual application exit. This is
    /// not a hide operation and never destroys eframe's HWND. Call in on_exit;
    /// Drop is a fallback. To keep using the app, use disable_tray instead.
    pub fn shutdown(&mut self) -> Result<(), String> {
        self.state = DesktopState::Closed;
        self.pending.clear();
        self.stop_tray()
    }
}

impl Drop for DesktopController {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            eprintln!("CPU Express desktop cleanup: {error}");
        }
    }
}
