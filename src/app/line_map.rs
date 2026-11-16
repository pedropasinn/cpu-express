//! Read-only railway atlas. Selecting a station never changes a live journey.
use super::*;
use crate::simulation::{
    catalog::{Catalog, LineDef},
    itinerary::{DEFAULT_UNITS_PER_KM, Itinerary},
    map_layout::{MapLayout, MapPoint},
};
use std::sync::OnceLock;

fn preview() -> Result<&'static (&'static LineDef, MapLayout), &'static str> {
    static MAP: OnceLock<Result<(&'static LineDef, MapLayout), String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let catalog = Catalog::builtin()
            .as_ref()
            .map_err(|_| "Catálogo indisponível.")?;
        let line = catalog
            .line("curitiba-paranagua")
            .ok_or("Linha indisponível.")?;
        let itinerary = Itinerary::from_line(line, DEFAULT_UNITS_PER_KM)?;
        Ok((line, MapLayout::new(line, &itinerary)?))
    })
    .as_ref()
    .map_err(|e| e.as_str())
}

impl CpuExpressApp {
    pub(super) fn line_map_page(&mut self, ui: &mut egui::Ui) {
        let (line, map) = match preview() {
            Ok(map) => map,
            Err(error) => {
                ui.colored_label(game::RED, error);
                return;
            }
        };
        ui.strong("Curitiba–Paranaguá · consulta da linha");
        ui.small("14 paradas reais · viagem ainda indisponível · Aurora continua em curso.");
        let count = map.stations().len();
        self.menu.line_station = self.menu.line_station.min(count - 1);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.menu.line_station > 0, egui::Button::new("Anterior"))
                .clicked()
            {
                self.menu.line_station = self.menu.line_station.saturating_sub(1);
            }
            ui.label(format!("Parada {} de {count}", self.menu.line_station + 1));
            if ui
                .add_enabled(
                    self.menu.line_station + 1 < count,
                    egui::Button::new("Próxima"),
                )
                .clicked()
            {
                self.menu.line_station = (self.menu.line_station + 1).min(count - 1);
            }
        });
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 140.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, game::CARD);
        let inside = rect.shrink2(egui::vec2(24.0, 30.0));
        let project = |point: MapPoint| {
            inside.min
                + egui::vec2(
                    inside.width() * point.x as f32,
                    inside.height() * point.y as f32,
                )
        };
        for pair in map.stations().windows(2) {
            painter.line_segment(
                [project(pair[0].point), project(pair[1].point)],
                egui::Stroke::new(4.0, game::EDGE),
            );
        }
        let focused = ui.memory(|m| m.focused());
        let station_id = |index| egui::Id::unique(("real-line-station", line.id.as_str(), index));
        for station in map.stations() {
            let pos = project(station.point);
            let selected = self.menu.line_station == station.ordinal;
            let hit = ui.interact(
                egui::Rect::from_center_size(pos, egui::Vec2::splat(32.0)),
                station_id(station.ordinal),
                egui::Sense::click(),
            );
            let name = &line.stations[station.catalog_index].name;
            hit.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::Button,
                    true,
                    format!("{}: {name}", station.ordinal + 1),
                )
            });
            if hit.clicked() {
                self.menu.line_station = station.ordinal;
            }
            if focused == Some(hit.id) {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    self.menu.line_station = (station.ordinal + 1).min(count - 1);
                    ui.memory_mut(|m| m.request_focus(station_id(self.menu.line_station)));
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    self.menu.line_station = station.ordinal.saturating_sub(1);
                    ui.memory_mut(|m| m.request_focus(station_id(self.menu.line_station)));
                }
                painter.circle_stroke(pos, 17.0, egui::Stroke::new(2.0, game::BLUE));
            }
            painter.circle_filled(pos, 12.0, if selected { game::GOLD } else { game::CARD });
            painter.circle_stroke(pos, 12.0, egui::Stroke::new(2.0, game::EDGE));
            painter.text(
                pos,
                egui::Align2::CENTER_CENTER,
                (station.ordinal + 1).to_string(),
                egui::FontId::proportional(14.0),
                game::INK,
            );
            hit.on_hover_text(name);
        }
        if let Some((low, high)) = map.elevation_range() {
            ui.small(format!(
                "Perfil de altitude · {low:.0}–{high:.0} m · distância proporcional"
            ));
        }
        let (profile, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 65.0), egui::Sense::hover());
        let plot = profile.shrink2(egui::vec2(10.0, 6.0));
        let project = |p: MapPoint| {
            plot.min + egui::vec2(plot.width() * p.x as f32, plot.height() * p.y as f32)
        };
        for pair in map.stations().windows(2) {
            if let (Some(a), Some(b)) = (pair[0].profile, pair[1].profile) {
                ui.painter().line_segment(
                    [project(a), project(b)],
                    egui::Stroke::new(2.0, game::GREEN),
                );
            }
        }
        let station = &map.stations()[self.menu.line_station];
        if let Some(point) = station.profile {
            ui.painter().circle_filled(project(point), 5.0, game::GOLD);
        }
        let definition = &line.stations[station.catalog_index];
        ui.strong(&definition.name);
        let km = definition
            .km
            .map_or("km não informado".into(), |km| format!("km {km:.1}"));
        let altitude = definition
            .elevation_m
            .map_or("altitude não informada".into(), |h| {
                format!("{h:.0} m de altitude")
            });
        ui.label(format!("{km} · {altitude}"));
        if let Some(architecture) = &definition.architecture {
            ui.small(format!(
                "Arte fictícia: {}",
                super::scenery::station_style(architecture.style).name()
            ));
        }
        ui.small("Esquema de paradas; não representa a geometria dos trilhos.");
        ui.small("Serviços planejados são do jogo e não representam a operação real.");
        ui.label(
            line.attribution
                .as_deref()
                .unwrap_or("Crédito não informado"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_keyboard_moves_one_stop_and_never_mutates_journey() {
        let ctx = egui::Context::default();
        let mut app = CpuExpressApp::new_demo(
            &ctx,
            LaunchOptions {
                config: Config::default(),
                config_path: None,
                warning: None,
                cpu_override: Some(50.0),
                exit_after: None,
                diagnostics: None,
            },
        );
        app.prepare_demo_menu("linhas", 5);
        let journey = app.world.journey_snapshot();
        let economy = app.world.economy().clone();
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(704.0, 676.0),
            )),
            ..Default::default()
        };
        for _ in 0..3 {
            ctx.run_ui(input(), |ui| app.central_menu(ui))
                .drop_without_applying_deltas();
        }
        ctx.memory_mut(|m| {
            m.request_focus(egui::Id::unique((
                "real-line-station",
                "curitiba-paranagua",
                5_usize,
            )))
        });
        let mut key = input();
        key.events.push(egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        });
        ctx.run_ui(key, |ui| app.central_menu(ui))
            .drop_without_applying_deltas();
        assert_eq!(app.menu.line_station, 6);
        assert_eq!(app.world.journey_snapshot(), journey);
        assert_eq!(app.world.economy(), &economy);
        assert!(
            preview()
                .unwrap()
                .0
                .attribution
                .as_ref()
                .unwrap()
                .contains("OpenStreetMap")
        );
    }
}
