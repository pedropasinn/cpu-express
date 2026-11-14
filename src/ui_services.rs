//! Compact railway service presentation, independent of domain and persistence.
//!
//! The integrator supplies truthful display values and handles returned actions.
//! No timers, simulated counts, reward grants, or repaint loops live here.

use crate::ui_game::{self as game, Icon};
use eframe::egui;
use std::hash::Hash;

/// One fact from an arrival, e.g. passengers delivered or a discovered station.
pub struct ArrivalItem<'a> {
    pub icon: Icon,
    pub label: &'a str,
    pub value: &'a str,
}

pub struct ArrivalSummary<'a> {
    pub station: &'a str,
    pub detail: &'a str,
    pub items: &'a [ArrivalItem<'a>],
    /// Already-earned reward text, if any. This component does not unlock it.
    pub reward: Option<&'a str>,
    /// Exact visible label for an optional caller-owned action.
    pub action_label: Option<&'a str>,
}

/// Parchment arrival ticket. `inner` is true only when the action is activated.
pub fn arrival_summary(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    summary: &ArrivalSummary<'_>,
) -> egui::InnerResponse<bool> {
    ui.push_id(id, |ui| {
        game::apply_style(ui);
        game::panel_frame()
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                emblem_heading(ui, Icon::Station, summary.station);
                if !summary.detail.is_empty() {
                    ui.label(egui::RichText::new(summary.detail).color(game::MUTED));
                }
                for item in summary.items {
                    fact(ui, item.icon, item.label, item.value);
                }
                if let Some(reward) = summary.reward {
                    ui.separator();
                    emblem_heading(ui, Icon::Star, reward);
                }
                summary.action_label.is_some_and(|label| {
                    game::button(
                        ui,
                        "arrival-action",
                        Icon::Ticket,
                        label,
                        true,
                        egui::vec2(ui.available_width(), 34.0),
                    )
                    .clicked()
                })
            })
            .inner
    })
}

/// Caller-owned progress; use `None` in ServiceView when no measurement exists.
pub struct ServiceProgress<'a> {
    pub fraction: f32,
    pub label: &'a str,
}

pub struct CapacityView<'a> {
    pub label: &'a str,
    pub icon: Icon,
    pub used: u32,
    pub total: u32,
}

/// Honest occupancy: totals of zero show no capacity, overflow stays visible in
/// the count while the graphical fill is clamped. No capacity is inferred.
pub fn capacity_meter(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    capacity: &CapacityView<'_>,
) -> egui::InnerResponse<()> {
    ui.push_id(id, |ui| {
        game::apply_style(ui);
        fact(
            ui,
            capacity.icon,
            capacity.label,
            &format!("{} / {}", capacity.used, capacity.total),
        );
        let fraction = if capacity.total == 0 {
            0.0
        } else {
            capacity.used as f32 / capacity.total as f32
        };
        let label = if capacity.total == 0 {
            "Sem capacidade"
        } else if capacity.used > capacity.total {
            "Capacidade excedida"
        } else {
            "Ocupação"
        };
        game::progress(ui, "capacity", fraction, label, game::BLUE);
    })
}

/// A service entry. IDs should describe the service, not its current list index.
pub struct ServiceView<'a> {
    pub id: &'a str,
    pub icon: Icon,
    pub title: &'a str,
    pub detail: &'a str,
    pub status: &'a str,
    pub progress: Option<ServiceProgress<'a>>,
    pub action_label: Option<&'a str>,
    /// Keeps explanation visible while disabling the action.
    pub unavailable_reason: Option<&'a str>,
}

/// Compact service card. Returns an action intent; it never starts a service.
pub fn service_card(ui: &mut egui::Ui, service: &ServiceView<'_>) -> egui::InnerResponse<bool> {
    ui.push_id(service.id, |ui| {
        game::apply_style(ui);
        game::panel_frame()
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                emblem_heading(ui, service.icon, service.title);
                if !service.detail.is_empty() {
                    ui.label(service.detail);
                }
                if !service.status.is_empty() {
                    ui.label(egui::RichText::new(service.status).color(game::MUTED));
                }
                if let Some(progress) = &service.progress {
                    game::progress(
                        ui,
                        "service-progress",
                        progress.fraction,
                        progress.label,
                        game::BLUE,
                    );
                }
                if let Some(reason) = service.unavailable_reason {
                    ui.label(egui::RichText::new(reason).color(game::MUTED));
                }
                service.action_label.is_some_and(|label| {
                    ui.add_enabled_ui(service.unavailable_reason.is_none(), |ui| {
                        game::button(
                            ui,
                            "service-action",
                            service.icon,
                            label,
                            false,
                            egui::vec2(ui.available_width(), 34.0),
                        )
                        .clicked()
                    })
                    .inner
                })
            })
            .inner
    })
}

/// A vertical dashboard that fits narrow panels without horizontal scrolling.
/// Returns the index of the activated service in this slice; use its `id` to
/// dispatch. Caller supplies the empty-state text instead of fictitious data.
pub fn service_dashboard(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    services: &[ServiceView<'_>],
    empty_label: &str,
) -> egui::InnerResponse<Option<usize>> {
    ui.push_id(id, |ui| {
        game::apply_style(ui);
        if services.is_empty() {
            ui.label(empty_label);
        }
        let mut action = None;
        for (index, service) in services.iter().enumerate() {
            if service_card(ui, service).inner {
                action = Some(index);
            }
        }
        action
    })
}

fn emblem_heading(ui: &mut egui::Ui, icon: Icon, label: &str) {
    ui.horizontal_wrapped(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(24.0), egui::Sense::hover());
        game::icon(ui.painter(), rect, icon, game::GREEN);
        ui.label(
            egui::RichText::new(label)
                .size(16.0)
                .strong()
                .color(game::INK),
        );
    });
}

fn fact(ui: &mut egui::Ui, icon: Icon, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
        game::icon(ui.painter(), rect, icon, game::MUTED);
        ui.label(label);
        ui.label(egui::RichText::new(value).strong().color(game::INK));
    });
}
