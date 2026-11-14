//! Windows integration and pure, testable physical-pixel placement.
//!
//! The simulation never uses these APIs. The host-validation feature only
//! supplies a no-op dock for graphical development checks outside Windows.

/// Shared identity for the independent game panel and native diagnostics.
pub const PANEL_TITLE: &str = "CPU Express · Caderno da ferrovia";

#[cfg(windows)]
mod desktop;
#[cfg(windows)]
mod tray;
#[cfg(windows)]
mod windows;

mod geometry;
mod lifecycle;

pub use geometry::{
    DockEdge, PhysicalRect, WindowPlacement, constrain_placement, dock_margin_for_dpi,
    dock_placement, dock_placement_at_edge, fit_placement_to_work_area, intersect_work_area,
    taskbar_placement,
};
pub use lifecycle::{DesktopAction, DesktopState, DesktopTransitionError};

#[cfg(windows)]
pub use desktop::{DesktopController, DesktopUpdate};
#[cfg(windows)]
pub use tray::{SystemTray, TrayEvent};

#[cfg(windows)]
pub use windows::{detach_taskbar, embed_taskbar, place_floating, taskbar_status};
#[cfg(windows)]
pub use windows::{dock_window, dock_window_at_edge, recover_window_placement, show_error};

#[cfg(all(not(windows), feature = "host-validation"))]
pub fn dock_window(_frame: &eframe::Frame) -> Result<(), String> {
    // Deliberately no Linux desktop integration in version 0.1.
    Ok(())
}

#[cfg(all(not(windows), not(feature = "host-validation")))]
pub fn dock_window(_frame: &eframe::Frame) -> Result<(), String> {
    Err("A integração com a área de trabalho nesta versão requer Windows.".to_owned())
}

#[cfg(not(windows))]
pub fn show_error(message: &str) {
    eprintln!("CPU Express: {message}");
}
