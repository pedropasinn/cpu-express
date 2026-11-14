//! CPU Express: reusable simulation, an isolated metrics provider and an original pixel renderer.

pub mod app;
pub mod audio;
pub mod config;
pub mod diagnostics;
mod frame_pacing;
mod frame_timings;
pub mod local_weather;
pub mod metrics;
pub mod platform;
pub mod progress;
pub mod rendering;
pub mod simulation;

pub mod ui_game;
pub mod ui_stock;

pub mod ui_services;
