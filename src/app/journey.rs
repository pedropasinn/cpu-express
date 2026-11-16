use super::*;

impl CpuExpressApp {
    pub(super) fn journey_controls(&mut self, ui: &mut egui::Ui) {
        game::apply_style(ui);
        if self.world.active_session().is_some() {
            self.line_map_page(ui);
            return;
        }
        let journey = self.world.journey_snapshot();
        let page_width = ((ui.available_width() - 32.0) / 5.0).floor().max(76.0);
        ui.horizontal_wrapped(|ui| {
            for (page, icon, label) in [
                (0, Icon::Map, "Mapa"),
                (1, Icon::Star, "Objetivos"),
                (2, Icon::Book, "Diário"),
                (3, Icon::Passenger, "Serviços"),
                (4, Icon::Ticket, "Coleção"),
            ] {
                if game::button(
                    ui,
                    ("journey-page", page),
                    icon,
                    label,
                    self.journey_page == page,
                    egui::vec2(page_width, if page_width < 100.0 { 46.0 } else { 34.0 }),
                )
                .clicked()
                {
                    self.journey_page = page;
                }
            }
        });
        ui.add_space(8.0);
        if self.journey_page == 0 {
            self.branch_controls(ui);
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
                    painter.line_segment(
                        [a, b],
                        egui::Stroke::new(
                            3.0,
                            if index == journey.route_segment.index {
                                game::GOLD
                            } else {
                                game::CARD
                            },
                        ),
                    );
                }
                for (index, label) in ["Vila Aurora", "Bosque", "Serra Clara", "Porto do Sol"]
                    .iter()
                    .enumerate()
                {
                    let pos = positions[index];
                    let station = self.world.journey().route().stations()[index];
                    let hit = ui.interact(
                        egui::Rect::from_center_size(
                            pos + egui::vec2(0.0, 10.0),
                            egui::vec2(96.0, 55.0),
                        ),
                        ui.scope_id().with(("station", index)),
                        egui::Sense::click(),
                    );
                    hit.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Button, true, station.name)
                    });
                    if hit.clicked() {
                        self.selected_station = Some(station.id);
                    }
                    if hit.has_focus() || self.selected_station == Some(station.id) {
                        painter.circle_stroke(pos, 17.0, egui::Stroke::new(2.0, game::BLUE));
                    }
                    hit.on_hover_text("Conhecer estação e serviços");
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
                if self.world.excursion_snapshot().current.is_none() {
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
                }
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
                let status = if self.world.excursion_snapshot().current.is_some() {
                    "Circuito aguardando o retorno da excursão".to_owned()
                } else if self.world.station_wait() > 0.0 {
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
            if let Some(station) = self.selected_station {
                section(ui, station.name(), |ui| {
                    use crate::simulation::StationId::*;
                    let description = match station {
                        Aurora => {
                            "A praça da vila reúne viajantes e correspondências. É o ponto de encontro do Circuito Aurora."
                        }
                        Pines => {
                            "Entre pinheiros, a estação recebe passeios panorâmicos e suprimentos para o bosque."
                        }
                        Ridge => {
                            "No alto da serra, a plataforma de pedra atende moradores e viajantes em busca da paisagem."
                        }
                        Harbor => {
                            "O porto conecta passageiros, encomendas e abastecimento junto ao mar."
                        }
                    };
                    ui.label(description);
                    ui.small(if journey.discovered_stations[station.index()] {
                        "Estação descoberta · cartão da coleção disponível"
                    } else {
                        "Ainda não visitada · viaje até aqui para descobrir"
                    });
                    let service = self.world.service_view();
                    let arriving: u32 = service
                        .passengers
                        .iter()
                        .filter(|p| {
                            p.destination
                                == crate::simulation::line_session::CatalogStationId::aurora(
                                    station,
                                )
                        })
                        .map(|p| p.count)
                        .sum();
                    ui.label(format!(
                        "{arriving} passageiros a bordo com destino a esta estação"
                    ));
                });
            }
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
            game::hint(ui, "Viaje, descubra estações e conquiste novas pinturas.");
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
                                self.highlight_livery = Some(objective.reward);
                            }
                        } else {
                            game::hint(ui, "Desbloqueio automático ao completar o objetivo.");
                        }
                    });
                });
            }
        }
        if self.journey_page == 3 {
            self.services_controls(ui);
        }
        if self.journey_page == 4 {
            self.collection_controls(ui);
        }
        if self.journey_page == 2 {
            // A failed save stays visible even with the file section closed.
            if let Some(message) = &self.progress_notice {
                ui.colored_label(game::RED, message);
            }
            ui.horizontal(|ui| {
                game::pixel_label(ui, "Últimas chegadas", 2.0, game::INK);
                game::hint(ui, "O diário guarda as últimas 32 chegadas.");
            });
            if self.world.service_view().arrivals.is_empty() {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(32.0), egui::Sense::hover());
                    game::sprite(ui.painter(), rect, game::Sprite::Bilhete);
                    self.arrival_ticket(ui, false);
                });
                if game::button(
                    ui,
                    "diary-open-map",
                    Icon::Map,
                    "Ver mapa",
                    false,
                    egui::vec2(170.0, 34.0),
                )
                .clicked()
                {
                    self.journey_page = 0;
                }
            } else {
                self.arrival_ticket(ui, false);
            }
            if !self.travel_log.is_empty() {
                egui::CollapsingHeader::new("Outros acontecimentos").show(ui, |ui| {
                    for message in self.travel_log.iter().rev() {
                        ui.label(message);
                    }
                });
            }
            egui::CollapsingHeader::new("Arquivo da viagem").show(ui, |ui| {
                if self.progress_path.is_some() {
                    ui.label(
                        egui::RichText::new("Salva sozinho a cada 30 s e ao sair.")
                            .color(game::MUTED),
                    );
                    if game::button(
                        ui,
                        "save-now",
                        Icon::Save,
                        "Salvar agora",
                        false,
                        egui::vec2(200.0, 34.0),
                    )
                    .clicked()
                    {
                        self.persist_progress(true);
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Viagem temporária · não salva neste perfil")
                            .color(game::MUTED),
                    );
                }
                self.storage_controls(ui);
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
}
