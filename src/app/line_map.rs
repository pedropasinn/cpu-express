//! Railway map and durable line selection. Station selection remains read-only.
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
    fn line_action(&mut self, line: Option<&str>) {
        let Some(path) = &self.progress_path else {
            self.progress_notice =
                Some("Viagem temporária: use um perfil salvo para trocar de linha.".into());
            return;
        };
        let mut warning = None;
        let persist = |candidate: &World| {
            warning = crate::progress::save_checkpoint(path, candidate)?;
            Ok(())
        };
        let result = if let Some(line) = line {
            self.world.switch_line(line, persist)
        } else {
            self.world
                .set_boarding_closed(!self.world.boarding_closed(), persist)
        };
        match result {
            Ok(()) => {
                self.progress_notice = warning;
                self.notification = Some("Escolha salva.".into());
                self.scene_dirty = true;
                if line.is_some() {
                    self.train.stop_at_station();
                }
            }
            Err(error) => self.progress_notice = Some(error),
        }
    }
    pub(super) fn line_map_page(&mut self, ui: &mut egui::Ui) {
        let (line, map) = match preview() {
            Ok(map) => map,
            Err(error) => {
                ui.colored_label(game::RED, error);
                return;
            }
        };
        ui.strong("Curitiba–Paranaguá · 14 paradas reais");
        let real_active = self.world.active_line_id() == line.id;
        ui.horizontal(|ui| {
            ui.small(if real_active {
                "Linha ativa · ida e volta"
            } else {
                "Linha ativa: Circuito Aurora"
            });
            let target = if real_active {
                "aurora-v1"
            } else {
                line.id.as_str()
            };
            let label = if real_active {
                "Voltar ao Circuito Aurora"
            } else {
                "Viajar nesta linha"
            };
            if ui
                .add_enabled(
                    self.progress_path.is_some() && self.world.can_leave_line(),
                    egui::Button::new(label),
                )
                .clicked()
            {
                self.line_action(Some(target));
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.progress_path.is_some(),
                    egui::Button::new(if self.world.boarding_closed() {
                        "Retomar embarques"
                    } else {
                        "Encerrar embarques"
                    }),
                )
                .clicked()
            {
                self.line_action(None);
            }
            let s = self.world.service_view();
            ui.small(format!(
                "A bordo: {} passageiros · {} carga",
                s.passenger_count(),
                s.cargo_count()
            ));
        });
        ui.small(
            if self.world.boarding_closed() && self.world.can_leave_line() {
                "Trem pronto: escolha uma linha ou retome os embarques para partir."
            } else {
                "Troca: parado, trem vazio, sem contrato aceito; terminal na linha real."
            },
        );
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
            egui::vec2(ui.available_width(), 112.0),
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
        let progress = self.world.line_progress().get(&line.id).cloned();
        for station in map.stations() {
            let pos = project(station.point);
            let selected = self.menu.line_station == station.ordinal;
            let hit = ui.interact(
                egui::Rect::from_center_size(pos, egui::Vec2::splat(32.0)),
                station_id(station.ordinal),
                egui::Sense::click(),
            );
            let name = &line.stations[station.catalog_index].name;
            let known = progress.as_ref().is_some_and(|p| {
                p.discovered_stations
                    .contains(&line.stations[station.catalog_index].id)
            });
            hit.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::Button,
                    true,
                    format!(
                        "{}: {name} · {}",
                        station.ordinal + 1,
                        if known { "Descoberta" } else { "A descobrir" }
                    ),
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
            if known {
                painter.circle_filled(pos + egui::vec2(9.0, -10.0), 4.0, game::GREEN);
                painter.text(
                    pos + egui::vec2(9.0, -10.0),
                    egui::Align2::CENTER_CENTER,
                    "+",
                    egui::FontId::proportional(10.0),
                    game::PAPER,
                );
            }
            painter.text(
                pos,
                egui::Align2::CENTER_CENTER,
                (station.ordinal + 1).to_string(),
                egui::FontId::proportional(14.0),
                game::INK,
            );
            hit.on_hover_text(name);
        }
        if real_active && let Some(progress) = &progress {
            let train = map.train(progress.distance);
            let p = project(train.point) + egui::vec2(0.0, 21.0);
            game::sprite(
                &painter,
                egui::Rect::from_center_size(p, egui::Vec2::splat(20.0)),
                game::Sprite::Locomotiva,
            );
            painter.line_segment(
                [p, project(train.point)],
                egui::Stroke::new(2.0, game::GOLD),
            );
        }
        if let Some((low, high)) = map.elevation_range() {
            ui.small(format!(
                "Perfil de altitude · {low:.0}–{high:.0} m · distância proporcional"
            ));
        }
        let (profile, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 42.0), egui::Sense::hover());
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
        let known = progress
            .as_ref()
            .is_some_and(|p| p.discovered_stations.contains(&definition.id));
        ui.label(format!(
            "{km} · {altitude} · {}",
            if known { "Descoberta" } else { "A descobrir" }
        ));
        if let Some(architecture) = &definition.architecture {
            ui.small(format!(
                "Arte fictícia: {}",
                super::scenery::station_style(architecture.style).name()
            ));
        }
        ui.small("Verde: descoberta · dourado: trem · esquema sem geometria real.");
        ui.small(crate::simulation::line_session::LineSession::SERVICE_NOTICE);
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
    fn line_actions_save_immediately_and_keep_state_after_failure() {
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
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!("ce-e3-ui-{nonce}"));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("journey.json");
        app.progress_path = Some(path.clone());
        std::fs::write(&path, "protegido").unwrap();
        app.line_action(Some("curitiba-paranagua"));
        assert_eq!(app.world.active_line_id(), "aurora-v1");
        assert!(app.progress_notice.is_some());
        app.line_action(None);
        assert!(!app.world.boarding_closed());
        let valid = folder.join("valid.json");
        app.progress_path = Some(valid.clone());
        app.line_action(None);
        assert!(
            crate::progress::load_world(&valid)
                .unwrap()
                .unwrap()
                .boarding_closed()
        );
        app.line_action(Some("curitiba-paranagua"));
        let reopened = crate::progress::load_world(&valid).unwrap().unwrap();
        assert_eq!(reopened.active_line_id(), "curitiba-paranagua");
        assert_eq!(reopened.economy(), app.world.economy());
        assert_eq!(std::fs::read(&path).unwrap(), b"protegido");
    }
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
