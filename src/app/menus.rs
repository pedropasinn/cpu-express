//! Fixed-size central menu. Pagination is presentation state, never progression.
use super::*;
use crate::simulation::economy::{ItemId, Rarity};

#[derive(Default)]
pub(super) struct MenuState {
    pub line_station: usize,
    pub service_destinations: usize,
    pub drive: usize,
    pub settings: usize,
    pub stock: [usize; 3],
    pub missions: usize,
    pub orders: usize,
    pub diary: usize,
    pub diary_tab: usize,
    pub postcards: usize,
    pub window: usize,
    pub climate: usize,
    pub comfort: usize,
}

pub(super) fn panel_scale(available: [f32; 2]) -> f32 {
    (available[0] / 704.0)
        .min(available[1] / 676.0)
        .clamp(0.1, 1.0)
}

pub(super) fn synchronize_panel_size(ctx: &egui::Context) {
    // Zoom takes effect on the next pass. A constant ViewportBuilder alone
    // does not reissue native dimensions when only pixels per point changed.
    let target = egui::vec2(704.0, 676.0);
    if ctx.input(|i| {
        i.viewport()
            .inner_rect
            .is_some_and(|r| (r.size() - target).abs().max_elem() > 1.0)
    }) {
        for command in [
            egui::ViewportCommand::MinInnerSize(egui::Vec2::ZERO),
            egui::ViewportCommand::MaxInnerSize(egui::Vec2::INFINITY),
            egui::ViewportCommand::InnerSize(target),
            egui::ViewportCommand::MinInnerSize(target),
            egui::ViewportCommand::MaxInnerSize(target),
        ] {
            ctx.send_viewport_cmd(command);
        }
    }
}

pub(super) fn tabs(ui: &mut egui::Ui, id: &str, current: &mut usize, labels: &[&str]) {
    let width = (ui.available_width() - (labels.len() - 1) as f32 * 6.0) / labels.len() as f32;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (index, label) in labels.iter().enumerate() {
            if ui
                .push_id((id, index), |ui| {
                    ui.add_sized(
                        [width, 32.0],
                        egui::Button::new(egui::RichText::new(*label).strong())
                            .selected(*current == index),
                    )
                })
                .inner
                .clicked()
            {
                *current = index;
            }
        }
    });
}

pub(super) fn pager(
    ui: &mut egui::Ui,
    id: &str,
    page: &mut usize,
    total: usize,
    per_page: usize,
) -> std::ops::Range<usize> {
    let pages = total.div_ceil(per_page).max(1);
    *page = (*page).min(pages - 1);
    ui.push_id(id, |ui| {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(*page > 0, egui::Button::new("<"))
                .on_hover_text("Página anterior")
                .clicked()
            {
                *page -= 1;
            }
            ui.label(format!("Página {} / {pages}", *page + 1));
            if ui
                .add_enabled(*page + 1 < pages, egui::Button::new(">"))
                .on_hover_text("Próxima página")
                .clicked()
            {
                *page += 1;
            }
        })
    });
    *page * per_page..((*page + 1) * per_page).min(total)
}

pub(super) fn rarity_style(rarity: Rarity) -> (&'static str, egui::Color32) {
    match rarity {
        Rarity::Common => ("Comum", game::MUTED),
        Rarity::Uncommon => ("Incomum", game::GREEN),
        Rarity::Rare => ("Raro", game::BLUE),
        Rarity::Epic => ("Épico", egui::Color32::from_rgb(117, 62, 144)),
    }
}

impl CpuExpressApp {
    /// Select a deterministic review page. Never changes a live profile.
    pub fn prepare_demo_menu(&mut self, page: &str, index: usize) {
        if !self.demo_mode {
            return;
        }
        if page == "favoritos" {
            let mut styles = CoachStyle::ALL;
            styles.sort_by_key(|style| std::cmp::Reverse(style.name().len()));
            self.config.visual.favorite_consists = (0..4)
                .map(|i| [styles[i], styles[i + 1], styles[i + 2]])
                .collect();
        }
        if page == "servicos-oferta" {
            self.prepare_demo_contract_review(false, false);
        }
        if page == "servicos-bloqueado" {
            self.prepare_demo_contract_review(false, true);
        }
        if page == "servicos-devolucao" {
            self.prepare_demo_contract_review(true, false);
        }
        if page == "ramais-ativo" {
            self.prepare_demo_review(900, Some(crate::simulation::BranchKind::Serra));
        }
        if page == "janela-livre" {
            self.config.taskbar_fit_height = false;
            self.config.taskbar_width = 640;
        }
        if page == "janela-flutuante" {
            self.config.taskbar_mode = false;
        }
        let page = page.split('-').next().unwrap_or(page);
        self.show_controls = true;
        self.show_help = page == "ajuda";
        self.welcome = page == "acolhida";
        self.menu = MenuState::default();
        self.menu.window = index.min(1);
        self.menu.climate = index.min(1);
        self.menu.comfort = index.min(2);
        self.panel_tab = PanelTab::Drive;
        match page {
            "vagoes" | "motores" | "pinturas" | "favoritos" => {
                self.menu.drive = match page {
                    "vagoes" => 1,
                    "motores" => 2,
                    "pinturas" => 3,
                    _ => 4,
                };
                self.menu.stock = [index; 3];
            }
            "mapa" | "missoes" | "diario" | "servicos" | "ramais" | "arquivo" | "linhas" => {
                self.panel_tab = PanelTab::Journey;
                self.journey_page = match page {
                    "mapa" => 0,
                    "missoes" => 1,
                    "diario" | "arquivo" => 2,
                    "servicos" => 3,
                    "linhas" => 5,
                    _ => 4,
                };
                self.menu.missions = index;
                self.menu.diary = index;
                self.menu.line_station = index;
                if page == "arquivo" {
                    self.menu.diary_tab = 2;
                }
            }
            "janela" | "clima" | "fluidez" | "sessao" => {
                self.panel_tab = PanelTab::Behavior;
                self.menu.settings = match page {
                    "janela" => 0,
                    "clima" => 1,
                    "fluidez" => 2,
                    _ => 3,
                };
            }
            "loja" => self.panel_tab = PanelTab::Shop,
            "colecao" => {
                self.panel_tab = PanelTab::Collection;
                self.menu.postcards = index;
            }
            _ => {}
        }
    }

    pub(super) fn central_menu(&mut self, root: &mut egui::Ui) {
        self.begin_economy_visit();
        game::apply_style(root);
        game::wood_frame(root.painter(), root.max_rect());
        egui::Frame::new().inner_margin(20).show(root, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            let title = if self.welcome {
                "Boas-vindas"
            } else if self.show_help {
                "Ajuda"
            } else {
                match self.panel_tab {
                    PanelTab::Drive => "Seu trem",
                    PanelTab::Journey => "Viagem",
                    PanelTab::Appearance => "Oficina",
                    PanelTab::Behavior => "Ajustes",
                    PanelTab::Shop => "Loja",
                    PanelTab::Collection => "Coleção",
                }
            };
            ui.painter().rect_filled(
                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 42.0)),
                2,
                game::RIBBON,
            );
            let ribbon = ui
                .horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(388.0, 42.0), |ui| {
                        game::ribbon(ui, title, Icon::Train, true);
                    });
                    let (coin, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(32.0), egui::Sense::hover());
                    game::sprite(ui.painter(), coin, game::Sprite::Moeda);
                    ui.add_sized(
                        [150.0, 32.0],
                        egui::Label::new(
                            egui::RichText::new(format!("{} réis", self.world.economy().balance))
                                .strong()
                                .size(20.0)
                                .color(game::CREAM),
                        )
                        .truncate(),
                    )
                    .on_hover_text(format!(
                        "{} Réis de Ferro · moedas do jogo",
                        self.world.economy().balance
                    ));
                    if ui.button("?").on_hover_text("Ajuda (F1)").clicked() {
                        self.show_help = !self.show_help;
                    }
                })
                .response
                .rect;
            let drag_rect =
                egui::Rect::from_min_size(ribbon.min, egui::vec2(420.0, ribbon.height()));
            if ui
                .interact(
                    drag_rect,
                    ui.scope_id().with("move-menu"),
                    egui::Sense::drag(),
                )
                .on_hover_cursor(egui::CursorIcon::Grab)
                .drag_started()
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            let content =
                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 492.0));
            ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                ui.set_clip_rect(content);
                ui.painter().rect_filled(content, 2, game::PAPER);
                ui.spacing_mut().item_spacing.y = 5.0;
                if self.welcome {
                    self.welcome_controls(ui);
                } else if self.show_help {
                    self.help_page(ui);
                } else {
                    match self.panel_tab {
                        PanelTab::Drive => self.train_pages(ui),
                        PanelTab::Journey => self.travel_pages(ui),
                        PanelTab::Appearance => self.stock_page(ui),
                        PanelTab::Behavior => self.settings_pages(ui),
                        PanelTab::Shop => self.shop_page(ui),
                        PanelTab::Collection => self.collection_page(ui),
                    }
                }
                // Exporters and tests inspect the actual extent, including open controls.
                ui.ctx().data_mut(|data| {
                    data.insert_temp(
                        egui::Id::unique("central-content-height"),
                        ui.min_rect().height(),
                    )
                });
                ui.ctx().data_mut(|data| {
                    data.insert_temp(
                        egui::Id::unique("central-content-width"),
                        (ui.min_rect().width(), content.width()),
                    )
                });
                ui.expand_to_include_rect(content);
            });
            let status = self
                .progress_notice
                .as_deref()
                .or(self.preferences_notice.as_deref())
                .or(self.shop.visit_summary.as_deref())
                .or(self.notification.as_deref())
                .or_else(|| self.equipment_notice.as_ref().map(|v| v.0.as_str()))
                .unwrap_or(if self.paused {
                    "Viagem pausada · progresso preservado"
                } else {
                    "Tempo ativo · sem prazo nem perda por ausência"
                });
            ui.add(
                egui::Label::new(egui::RichText::new(status).size(12.0).color(
                    if self.progress_notice.is_some() {
                        game::RED
                    } else {
                        game::GREEN
                    },
                ))
                .truncate(),
            )
            .on_hover_text(status);
            let reward = self.has_claimable_rewards();

            ui.horizontal(|ui| {
                let size = egui::vec2((ui.available_width() - 30.0) / 6.0, 56.0);
                for (tab, label, icon) in [
                    (PanelTab::Drive, "Trem", Icon::Train),
                    (PanelTab::Journey, "Viagem", Icon::Map),
                    (PanelTab::Shop, "Loja", Icon::Cargo),
                    (PanelTab::Collection, "Coleção", Icon::Ticket),
                    (PanelTab::Behavior, "Ajustes", Icon::Settings),
                ] {
                    let response = game::dock_button(
                        ui,
                        ("dock-e5", tab as u8),
                        icon,
                        label,
                        self.panel_tab == tab && !self.show_help,
                        size,
                    );
                    if tab == PanelTab::Journey && reward {
                        let dot = response.rect.right_top() + egui::vec2(-9.0, 8.0);
                        ui.painter().circle_filled(dot, 5.0, game::GOLD);
                        ui.painter().text(
                            dot,
                            egui::Align2::CENTER_CENTER,
                            "!",
                            egui::FontId::proportional(10.0),
                            game::INK,
                        );
                    }
                    if response
                        .on_hover_text(if tab == PanelTab::Journey && reward {
                            "Há prêmios para recolher em Missões"
                        } else {
                            label
                        })
                        .clicked()
                    {
                        self.panel_tab = tab;
                        self.show_help = false;
                    }
                }
                if game::dock_button(ui, "close-e5", Icon::Close, "Fechar", false, size).clicked() {
                    self.dispatch(AppAction::ClosePanel, ui.ctx());
                }
            });
        });
    }

    fn train_pages(&mut self, ui: &mut egui::Ui) {
        tabs(
            ui,
            "train",
            &mut self.menu.drive,
            &[
                "Cabine",
                "Vagões",
                "Motores",
                "Pinturas",
                "Favoritos",
                "Máquina",
            ],
        );
        if self.menu.drive == 5 {
            self.machine_page(ui);
            return;
        }
        if (1..=3).contains(&self.menu.drive) {
            self.stock_page(ui);
            return;
        }
        if self.menu.drive == 4 {
            self.favorites_page(ui);
            return;
        }
        self.scene_preview(ui, true);
        ui.heading(self.destination_name());
        let remaining = self.world.excursion_snapshot().current.map_or_else(
            || {
                if self.world.station_wait() > 0.0 {
                    self.world.station_wait()
                } else {
                    self.world.travel_view().distance_to_next / 24.0
                }
            },
            |e| e.remaining_seconds(),
        );
        game::progress(
            ui,
            "cab-progress",
            self.travel_progress(),
            &format!(
                "{} · {:.0} s ativos",
                if self.world.boarding_closed() && self.world.can_leave_line() {
                    "Pronto para trocar de linha"
                } else if self.suspended() {
                    "Relógio parado"
                } else if self.world.station_wait() > 0.0 {
                    "Embarque"
                } else {
                    "Próxima parada"
                },
                remaining.ceil()
            ),
            game::GREEN,
        );
        if game::button(
            ui,
            "cab-pause",
            if self.paused { Icon::Play } else { Icon::Pause },
            if self.paused { "Retomar" } else { "Pausar" },
            false,
            egui::vec2(170.0, 36.0),
        )
        .clicked()
        {
            self.dispatch(AppAction::TogglePause, ui.ctx());
        }
        ui.label(format!(
            "CPU {} · Memória {} · {:.0} km/h visuais",
            self.cpu_label(),
            self.ram_percent()
                .map_or("sem leitura".into(), |v| format!("{v:.0}%")),
            self.train.speed_kmh()
        ));
        let service = self.world.service_view();
        let (seats, cargo) = service.capacity();
        if ui
            .button(format!(
                "Serviços · {}/{} passageiros · {}/{} carga",
                service.passenger_count(),
                seats,
                service.cargo_count(),
                cargo
            ))
            .clicked()
        {
            self.panel_tab = PanelTab::Journey;
            self.journey_page = 3;
        }
        self.arrival_ticket(ui, true);
    }

    fn travel_pages(&mut self, ui: &mut egui::Ui) {
        let mut page = self.journey_page as usize;
        tabs(
            ui,
            "travel",
            &mut page,
            &["Mapa", "Missões", "Diário", "Serviços", "Ramais", "Linhas"],
        );
        self.journey_page = page as u8;
        match page {
            1 => self.missions_page(ui),
            2 => self.diary_page(ui),
            3 => self.services_controls(ui),
            4 => self.branch_controls(ui),
            5 => self.line_map_page(ui),
            _ => self.map_page(ui),
        }
    }

    fn map_page(&mut self, ui: &mut egui::Ui) {
        if self.world.active_session().is_some() {
            self.line_map_page(ui);
            return;
        }
        let j = self.world.journey_snapshot();
        ui.strong("Circuito Aurora · linha fictícia");
        if self.world.excursion_snapshot().current.is_some() {
            self.branch_controls(ui);
            return;
        }
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 200.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3, egui::Color32::from_rgb(191, 211, 172));
        let points = [(0.20, 0.25), (0.73, 0.25), (0.73, 0.68), (0.20, 0.68)]
            .map(|(x, y)| rect.min + egui::vec2(rect.width() * x, rect.height() * y));
        for i in 0..4 {
            painter.line_segment(
                [points[i], points[(i + 1) % 4]],
                egui::Stroke::new(5.0, game::EDGE),
            );
        }
        for station in self.world.journey().route().stations() {
            let p = points[station.id.index()];
            let known = j.discovered_stations[station.id.index()];
            let hit = ui.interact(
                egui::Rect::from_center_size(p, egui::vec2(150.0, 58.0)),
                ui.scope_id().with(station.id.index()),
                egui::Sense::click(),
            );
            if hit.clicked() {
                self.selected_station = Some(station.id);
            }
            hit.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, station.name));
            painter.circle_filled(p, 14.0, game::CARD);
            game::icon(
                &painter,
                egui::Rect::from_center_size(p, egui::Vec2::splat(22.0)),
                if known { Icon::Station } else { Icon::Lock },
                game::GREEN,
            );
            painter.text(
                p + egui::vec2(0.0, 17.0),
                egui::Align2::CENTER_TOP,
                station.name,
                egui::FontId::proportional(14.0),
                game::INK,
            );
            if hit.has_focus() || self.selected_station == Some(station.id) {
                painter.circle_stroke(p, 18.0, egui::Stroke::new(2.0, game::BLUE));
            }
        }
        let i = j.route_segment.index;
        let p = points[i] + (points[(i + 1) % 4] - points[i]) * j.segment_progress as f32;
        painter.circle_filled(p, 8.0, game::GOLD);
        game::progress(
            ui,
            "map-progress",
            j.segment_progress as f32,
            &format!("{} → {}", j.route_segment.from.name(), j.next_station.name),
            game::GREEN,
        );
        ui.label(format!(
            "{} chegadas · {} circuitos · {}",
            j.stations_visited,
            j.completed_laps,
            if self.suspended() {
                "relógio parado"
            } else {
                "tempo ativo"
            }
        ));
        if let Some(station) = self.selected_station {
            let n: u32 = self
                .world
                .service_view()
                .passengers
                .iter()
                .filter(|p| {
                    p.destination
                        == crate::simulation::line_session::CatalogStationId::aurora(station)
                })
                .map(|p| p.count)
                .sum();
            ui.strong(station.name());
            ui.label(format!(
                "{n} passageiros desembarcam aqui · {}",
                if j.discovered_stations[station.index()] {
                    "descoberta"
                } else {
                    "a descobrir"
                }
            ));
        }
        ui.small("Ramais: visitas de exploração; os serviços do circuito aguardam o retorno.");
    }

    fn diary_page(&mut self, ui: &mut egui::Ui) {
        tabs(
            ui,
            "diary",
            &mut self.menu.diary_tab,
            &["Chegadas", "Acontecimentos", "Arquivo"],
        );
        if self.menu.diary_tab == 2 {
            ui.label(if self.progress_path.is_some() {
                "Salvamento automático a cada 30 s e nas transações."
            } else {
                "Viagem temporária · não salva neste perfil"
            });
            if ui
                .add_enabled(
                    self.progress_path.is_some(),
                    egui::Button::new("Salvar agora"),
                )
                .clicked()
            {
                self.persist_progress(true);
            }
            self.storage_controls(ui);
            return;
        }
        if self.menu.diary_tab == 1 {
            let range = pager(ui, "log", &mut self.menu.diary, self.travel_log.len(), 4);
            for message in self
                .travel_log
                .iter()
                .rev()
                .skip(range.start)
                .take(range.len())
            {
                ui.label(message);
                ui.separator();
            }
            return;
        }
        let state = self.world.service_view();
        let range = pager(
            ui,
            "arrivals",
            &mut self.menu.diary,
            state.arrivals.len(),
            3,
        );
        if state.arrivals.is_empty() {
            ui.label(format!("Seu diário começa em {}", self.destination_name()));
        }
        for a in state
            .arrivals
            .iter()
            .rev()
            .skip(range.start)
            .take(range.len())
        {
            egui::Frame::new()
                .fill(game::CARD)
                .inner_margin(8)
                .show(ui, |ui| {
                    ui.strong(format!("{} · chegada {}", a.station.name(), a.visit));
                    ui.label(format!(
                        "+{} embarcaram · −{} desembarcaram · +{} pontos",
                        a.boarded, a.alighted, a.earned
                    ));
                    if a.cargo_loaded + a.cargo_delivered + a.cargo_returned > 0 {
                        ui.label(format!(
                            "Carga: {} carregadas · {} entregues · {} devolvidas",
                            a.cargo_loaded, a.cargo_delivered, a.cargo_returned
                        ));
                    }
                });
        }
    }

    pub(super) fn stock_page(&mut self, ui: &mut egui::Ui) {
        if self.stock_gallery.is_none() {
            self.stock_gallery = Some(StockGallery::new(ui.ctx()));
        }
        let category = if self.panel_tab == PanelTab::Appearance {
            self.menu.drive = 1;
            0
        } else {
            self.menu.drive.saturating_sub(1).min(2)
        };
        if category == 0 {
            ui.horizontal(|ui| {
                ui.label("Equipar na posição:");
                for index in 0..3 {
                    ui.selectable_value(
                        &mut self.coach_slot,
                        index,
                        format!("Vagão {}", index + 1),
                    );
                }
            });
            let (seats, hold) = self.world.service_view().capacity();
            ui.label(format!("Trem: {seats} lugares · {hold} espaços de carga"));
        } else {
            ui.label(if category == 1 {
                "Locomotivas · aparência, sem bônus de velocidade"
            } else {
                "Pinturas · esmalte da locomotiva"
            });
        }
        let items: Vec<_> = match category {
            0 => CoachStyle::ALL
                .iter()
                .map(|v| (ItemId::wagon(service_wagon(*v)), v.name()))
                .collect(),
            1 => crate::rendering::EngineStyle::ALL
                .iter()
                .map(|v| (ItemId::Locomotive(v.id().into()), v.name()))
                .collect(),
            _ => crate::simulation::objectives::Livery::ALL
                .iter()
                .map(|v| (ItemId::livery(*v), v.name()))
                .collect(),
        };
        let range = pager(
            ui,
            "stock-pages",
            &mut self.menu.stock[category],
            items.len(),
            4,
        );
        let mut equip = None;
        for row in items[range.clone()].chunks(2) {
            ui.columns(2, |cols| {
                for (col, (id, name)) in cols.iter_mut().zip(row) {
                    let index = items.iter().position(|v| v.0 == *id).unwrap();
                    let owned = self.world.economy().owns(id);
                    let (texture, selected) = match category {
                        0 => {
                            let v = CoachStyle::ALL[index];
                            (
                                self.stock_gallery.as_ref().unwrap().coach(v),
                                self.config.visual.coaches[self.coach_slot] == v,
                            )
                        }
                        1 => {
                            let v = crate::rendering::EngineStyle::ALL[index];
                            (
                                self.stock_gallery.as_ref().unwrap().engine(v),
                                self.config.visual.engine == v,
                            )
                        }
                        _ => {
                            let v = crate::simulation::objectives::Livery::ALL[index];
                            (
                                self.stock_gallery.as_ref().unwrap().livery(v),
                                self.config.visual.livery == v,
                            )
                        }
                    };
                    let (rarity, color) = rarity_style(id.rarity().unwrap_or_default());
                    let response = stock_card(
                        col,
                        texture,
                        name,
                        if selected {
                            "Equipado · possuído"
                        } else if owned {
                            "Possuído · equipar"
                        } else {
                            "Não possuído · obtenha na loja"
                        },
                        selected,
                        owned,
                    );
                    col.painter().rect_stroke(
                        response.rect.shrink(1.0),
                        3,
                        egui::Stroke::new(2.0, color),
                        egui::StrokeKind::Inside,
                    );
                    col.horizontal(|ui| {
                        ui.colored_label(color, rarity);
                        if category == 0 {
                            let (seats, cargo) = service_wagon(CoachStyle::ALL[index]).capacity();
                            let (r, _) = ui
                                .allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                            game::icon(
                                ui.painter(),
                                r,
                                if seats > 0 {
                                    Icon::Passenger
                                } else {
                                    Icon::Cargo
                                },
                                game::INK,
                            );
                            ui.label(if seats > 0 {
                                format!("{seats} lugares")
                            } else {
                                format!("{cargo} carga")
                            })
                            .on_hover_text(
                                crate::simulation::CargoKind::ALL
                                    .iter()
                                    .filter(|kind| {
                                        service_wagon(CoachStyle::ALL[index]).cargo_capacity(**kind)
                                            > 0
                                    })
                                    .map(|kind| kind.name())
                                    .collect::<Vec<_>>()
                                    .join(" · "),
                            );
                        }
                    });
                    if response.clicked() {
                        if owned {
                            equip = Some(index);
                        } else {
                            self.notification =
                                Some(format!("{name}: ainda não possuído. Consulte a Loja."));
                        }
                    }
                }
            });
        }
        if let Some(index) = equip {
            let result = match category {
                0 => {
                    let mut coaches = self.config.visual.coaches;
                    coaches[self.coach_slot] = CoachStyle::ALL[index];
                    self.world
                        .set_service_composition(coaches.map(service_wagon))
                        .map(|()| self.config.visual.coaches = coaches)
                }
                1 => {
                    self.config.visual.engine = crate::rendering::EngineStyle::ALL[index];
                    Ok(())
                }
                _ => {
                    self.config.visual.livery = crate::simulation::objectives::Livery::ALL[index];
                    Ok(())
                }
            };
            match result {
                Ok(()) => {
                    self.preferences_changed();
                    self.persist_progress(true);
                    self.notification = Some("Peça equipada.".into());
                    self.play_cue(crate::audio::AudioCue::Equip);
                }
                Err(error) => self.notification = Some(error),
            }
        }
    }

    fn favorites_page(&mut self, ui: &mut egui::Ui) {
        ui.label("Até quatro composições; a ocupação atual precisa caber.");
        if ui
            .add_enabled(
                self.config.visual.favorite_consists.len() < 4,
                egui::Button::new("Guardar composição atual"),
            )
            .clicked()
        {
            self.config
                .visual
                .favorite_consists
                .push(self.config.visual.coaches);
            self.preferences_changed();
        }
        let mut remove = None;
        let mut equip = None;
        for (index, coaches) in self.config.visual.favorite_consists.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(420.0, 44.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(coaches.map(coach_name).join(" · "));
                    },
                );
                if ui.button("Equipar").clicked() {
                    equip = Some(*coaches);
                }
                if ui.button("Esquecer").clicked() {
                    remove = Some(index);
                }
            });
        }
        if let Some(index) = remove {
            self.config.visual.favorite_consists.remove(index);
            self.preferences_changed();
        }
        if let Some(coaches) = equip {
            match self
                .world
                .set_service_composition(coaches.map(service_wagon))
            {
                Ok(()) => {
                    self.config.visual.coaches = coaches;
                    self.preferences_changed();
                    self.persist_progress(true);
                }
                Err(e) => self.notification = Some(e),
            }
        }
    }

    pub(super) fn postcards_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Postais de viagem");
        let j = self.world.journey_snapshot();
        let mut cards: Vec<_> = self
            .world
            .journey()
            .route()
            .stations()
            .iter()
            .map(|s| (s.name, j.discovered_stations[s.id.index()]))
            .collect();
        if let Ok(catalog) = crate::simulation::catalog::Catalog::builtin()
            && let Some(line) = catalog.line("curitiba-paranagua")
        {
            let lines = self.world.line_progress();
            let progress = lines.get(&line.id);
            for station in &line.stations {
                cards.push((
                    station.name.as_str(),
                    progress.is_some_and(|p| p.discovered_stations.contains(&station.id)),
                ));
            }
        }
        for stop in [
            crate::simulation::BranchStop::MiranteDaSerra,
            crate::simulation::BranchStop::CaisDasConchas,
        ] {
            cards.push((
                stop.name(),
                self.world.excursion_snapshot().discoveries.contains(&stop),
            ));
        }
        for (reward, name) in [
            (
                crate::simulation::service::ServiceReward::PassengerPostcard,
                "Companhia de viagem",
            ),
            (
                crate::simulation::service::ServiceReward::CargoPostcard,
                "Correio sobre trilhos",
            ),
        ] {
            cards.push((name, self.world.service_view().collection.contains(&reward)));
        }
        ui.label(format!(
            "{} / {} postais conquistados",
            cards.iter().filter(|c| c.1).count(),
            cards.len()
        ));
        let range = pager(ui, "postcards", &mut self.menu.postcards, cards.len(), 4);
        for row in cards[range].chunks(2) {
            ui.columns(2, |cols| {
                for (col, (name, owned)) in cols.iter_mut().zip(row) {
                    egui::Frame::new()
                        .fill(game::CARD)
                        .inner_margin(12)
                        .show(col, |ui| {
                            let (r, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 56.0),
                                egui::Sense::hover(),
                            );
                            game::icon(
                                ui.painter(),
                                egui::Rect::from_center_size(r.center(), egui::Vec2::splat(40.0)),
                                if *owned { Icon::Station } else { Icon::Lock },
                                game::GREEN,
                            );
                            ui.strong(*name);
                            ui.label(if *owned {
                                "Conquistado"
                            } else {
                                "Ainda por descobrir"
                            });
                        });
                }
            });
        }
        if ui.button("Ver peças do trem").clicked() {
            self.panel_tab = PanelTab::Drive;
            self.menu.drive = 1;
        }
    }

    fn settings_pages(&mut self, ui: &mut egui::Ui) {
        tabs(
            ui,
            "settings",
            &mut self.menu.settings,
            &["Janela", "Clima", "Fluidez", "Sessão"],
        );
        match self.menu.settings {
            0 => {
                tabs(
                    ui,
                    "window",
                    &mut self.menu.window,
                    &["Posição e tamanho", "Durante o trabalho"],
                );
                self.behavior_controls(ui);
            }
            1 => self.climate_page(ui),
            2 => self.comfort_page(ui),
            _ => {
                self.compact_drive_controls(ui);
                ui.separator();
                self.storage_controls(ui);
                if ui.button("Rever boas-vindas").clicked() {
                    self.welcome_taskbar = self.config.taskbar_mode;
                    self.weather_draft = self.config.weather_location.clone();
                    self.welcome = true;
                }
                if game::button(
                    ui,
                    "quit-game",
                    Icon::Save,
                    "Salvar e sair",
                    false,
                    egui::vec2(200.0, 36.0),
                )
                .clicked()
                {
                    self.dispatch(AppAction::Quit, ui.ctx());
                }
            }
        }
    }

    fn climate_page(&mut self, ui: &mut egui::Ui) {
        tabs(
            ui,
            "climate",
            &mut self.menu.climate,
            &["Condições", "Alterar cidade"],
        );
        if self.menu.climate == 1 {
            ui.label("Cidade");
            ui.text_edit_singleline(&mut self.weather_draft.city);
            ui.horizontal(|ui| {
                ui.label("Latitude");
                ui.add(
                    egui::DragValue::new(&mut self.weather_draft.latitude)
                        .range(-90.0..=90.0)
                        .speed(0.001)
                        .max_decimals(4),
                );
                ui.label("Longitude");
                ui.add(
                    egui::DragValue::new(&mut self.weather_draft.longitude)
                        .range(-180.0..=180.0)
                        .speed(0.001)
                        .max_decimals(4),
                );
            });
            ui.label("O nome identifica o local; informe as coordenadas. Sem GPS ou localização automática.");
            if ui.button("Usar São Paulo, SP").clicked() {
                self.weather_draft = Default::default();
            }
            if ui.button("Aplicar local").clicked() {
                match self.local_weather.set_location(self.weather_draft.clone()) {
                    Ok(()) => {
                        self.config.weather_location = self.weather_draft.clone();
                        self.preferences_changed();
                        self.local_weather.refresh(ui.ctx());
                        self.notification = Some("Local do clima atualizado.".into());
                    }
                    Err(error) => self.notification = Some(error),
                }
            }
        } else {
            ui.heading(&self.config.weather_location.city);
            if let Some(s) = &self.local_weather.last_good {
                ui.label(format!(
                    "{} · {:.1} °C · {}",
                    s.description(),
                    s.temperature_c,
                    if s.is_day { "Dia" } else { "Noite" }
                ));
                ui.label(format!(
                    "Condição das {} · {}",
                    s.local_time(),
                    if s.stale() {
                        "dados antigos"
                    } else {
                        "última condição válida"
                    }
                ));
                if let Some(w) = s.wind_speed_kmh {
                    ui.label(format!("Vento: {w:.0} km/h"));
                }
                if let Some(r) = s.precipitation_mm {
                    ui.label(format!("Precipitação: {r:.1} mm"));
                }
            } else {
                ui.label("Atmosfera ilustrativa até receber a condição local.");
            }
            if self.local_weather.loading() {
                ui.label("Consultando clima…");
            }
            if self.local_weather.error.is_some() {
                ui.colored_label(
                    game::RED,
                    "Clima indisponível; mantendo a última condição válida.",
                );
            }
            if self.local_weather.enabled {
                if ui
                    .add_enabled(
                        self.local_weather.can_refresh(),
                        egui::Button::new("Atualizar clima"),
                    )
                    .clicked()
                {
                    self.local_weather.refresh(ui.ctx());
                }
            } else {
                ui.label("Demonstração: sem consulta à internet.");
            }
            ui.label("Atualização automática a cada 15 minutos. Condição estimada para a cidade.");
        }
        ui.hyperlink_to("Dados: Open-Meteo · CC BY 4.0", "https://open-meteo.com/");
    }

    fn comfort_page(&mut self, ui: &mut egui::Ui) {
        tabs(
            ui,
            "comfort",
            &mut self.menu.comfort,
            &["Movimento", "Atenção e som", "Informações"],
        );
        let mut changed = false;
        match self.menu.comfort {
            0 => {
                changed |= ui
                    .add(egui::Slider::new(&mut self.config.fps, 10..=60).text("Limite de FPS"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.config.smoothing_k, 0.05..=20.0)
                            .logarithmic(true)
                            .text("Resposta visual à CPU"),
                    )
                    .changed();
                changed |= ui
                    .checkbox(
                        &mut self.config.reduce_menu_motion,
                        "Reduzir movimento de menus, passageiros e paisagem",
                    )
                    .changed();
                ui.label("Poses estáticas nos elementos decorativos. Viagem e recompensas mantêm o mesmo ritmo.");
            }
            1 => {
                changed |= ui
                    .checkbox(
                        &mut self.config.quiet_mode,
                        "Trabalho tranquilo: ocultar avisos e silenciar eventos",
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.config.sound_volume, 0.0..=1.0)
                            .text("Volume dos eventos"),
                    )
                    .changed();
                if ui
                    .add_enabled(
                        self.config.sound_volume > 0.0 && !self.config.quiet_mode,
                        egui::Button::new("Ouvir exemplo"),
                    )
                    .clicked()
                {
                    self.play_cue(crate::audio::AudioCue::Arrival);
                }
                changed |= ui
                    .checkbox(
                        &mut self.config.use_input_activity,
                        "Usar atividade do teclado e mouse",
                    )
                    .changed();
                ui.label("Só presença de atividade no último minuto. Sem ler teclas, títulos de janela ou salvar histórico.");
            }
            _ => {
                let mut mode = self.config.effective_hud_mode();
                ui.horizontal(|ui| {
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
            }
        }
        if changed {
            self.preferences_changed();
        }
    }

    fn help_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Uma ferrovia para acompanhar seu dia");
        ui.label("1. A viagem avança com tempo ativo. CPU muda o ritmo visual.");
        ui.label(
            "2. Passageiros e cargas viajam nos vagões compatíveis; entregas são automáticas.",
        );
        ui.label("3. Clique no trem para abrir a central. Recolha moedas em Viagem > Missões.");
        ui.label("Pausa, ocultação e minimização não acumulam progresso offline.");
        ui.separator();
        self.help_contents(ui);
        ui.label("As leituras da máquina não saem do computador. O clima consulta o serviço de meteorologia.");
        ui.small("Mapas reais: © OpenStreetMap contributors. Aurora é fictícia.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::economy::EconomyAction;

    #[test]
    fn real_line_pages_fit_with_many_destinations_and_correct_route() {
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
        for seconds in [0, 1300, 2300, 3500, 4500] {
            app.prepare_demo_real_line(seconds);
            assert_eq!(app.destination_name(), app.world.travel_view().next_name);
            for page in [
                "trem", "mapa", "linhas", "servicos", "diario", "missoes", "ramais",
            ] {
                app.prepare_demo_menu(page, if page == "missoes" { 6 } else { 0 });
                for _ in 0..3 {
                    ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(704.0, 676.0),
                            )),
                            ..Default::default()
                        },
                        |ui| app.central_menu(ui),
                    )
                    .drop_without_applying_deltas();
                }
                let height = ctx
                    .data(|d| d.get_temp::<f32>(egui::Id::unique("central-content-height")))
                    .unwrap();
                let (width, limit) = ctx
                    .data(|d| d.get_temp::<(f32, f32)>(egui::Id::unique("central-content-width")))
                    .unwrap();
                assert!(height <= 492.1, "{seconds}/{page}: {height}");
                assert!(width <= limit + 0.1, "{seconds}/{page}: {width} > {limit}");
            }
        }
    }

    #[test]
    fn all_central_pages_fit_without_scrolling() {
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
        app.prepare_demo_review(3600, None);
        for page in [
            "trem",
            "mapa",
            "linhas",
            "missoes",
            "diario",
            "arquivo",
            "servicos",
            "ramais",
            "vagoes",
            "motores",
            "pinturas",
            "favoritos",
            "janela",
            "clima",
            "fluidez",
            "sessao",
            "ajuda",
            "acolhida",
            "colecao",
            "loja",
        ] {
            for index in 0..5 {
                app.prepare_demo_menu(page, index);
                for _ in 0..3 {
                    let output = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(704.0, 676.0),
                            )),
                            ..Default::default()
                        },
                        |ui| app.central_menu(ui),
                    );
                    fn close_rect(shape: &egui::Shape) -> Option<egui::Rect> {
                        match shape {
                            egui::Shape::Text(t) if t.galley.job.text == "Fechar" => {
                                Some(t.visual_bounding_rect())
                            }
                            egui::Shape::Vec(v) => v.iter().find_map(close_rect),
                            _ => None,
                        }
                    }
                    let close = output.shapes.iter().find_map(|s| close_rect(&s.shape));
                    output.drop_without_applying_deltas();
                    assert!(
                        close.is_some_and(|r| r.bottom() < 676.0),
                        "{page}: dock {close:?}"
                    );
                }
                let height = ctx
                    .data(|d| d.get_temp::<f32>(egui::Id::unique("central-content-height")))
                    .unwrap();
                assert!(height <= 492.1, "{page}/{index}: {height} points");
                let (width, limit) = ctx
                    .data(|d| d.get_temp::<(f32, f32)>(egui::Id::unique("central-content-width")))
                    .unwrap();
                assert!(width <= limit + 0.1, "{page}/{index}: {width} > {limit}");
            }
        }
    }

    #[test]
    fn small_screen_keeps_aspect_ratio_and_never_enlarges() {
        assert_eq!(panel_scale([1920.0, 1080.0]), 1.0);
        let scale = panel_scale([480.0, 480.0]);
        assert!(704.0 * scale <= 480.0 && 676.0 * scale <= 480.0);
    }

    #[test]
    fn zoom_reapplies_native_constraints_after_pixels_per_point_changes() {
        for (size, expected) in [(egui::vec2(880.0, 845.0), 5), (egui::vec2(704.0, 676.0), 0)] {
            let ctx = egui::Context::default();
            let mut input = egui::RawInput::default();
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size));
            let output = ctx.run_ui(input, |ui| synchronize_panel_size(ui.ctx()));
            let commands = output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .filter(|command| {
                    matches!(
                        command,
                        egui::ViewportCommand::MinInnerSize(_)
                            | egui::ViewportCommand::MaxInnerSize(_)
                            | egui::ViewportCommand::InnerSize(_)
                    )
                })
                .cloned()
                .collect::<Vec<_>>();
            output.drop_without_applying_deltas();
            assert_eq!(commands.len(), expected);
            if expected > 0 {
                assert!(
                    matches!(commands[0],egui::ViewportCommand::MinInnerSize(v) if v==egui::Vec2::ZERO)
                );
                assert!(
                    matches!(commands[2],egui::ViewportCommand::InnerSize(v) if v==egui::vec2(704.0,676.0))
                );
            }
        }
    }

    #[test]
    fn exceptional_pages_keep_actions_inside_the_window() {
        for page in [
            "servicos-oferta",
            "servicos-bloqueado",
            "servicos-devolucao",
            "ramais-ativo",
            "janela-livre",
            "janela-flutuante",
            "missoes-vazio",
        ] {
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
            app.prepare_demo_menu(page, 0);
            for _ in 0..3 {
                ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(704.0, 676.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        egui::Frame::new()
                            .inner_margin(8)
                            .show(ui, |ui| app.central_menu(ui));
                    },
                )
                .drop_without_applying_deltas();
            }
            let height = ctx
                .data(|d| d.get_temp::<f32>(egui::Id::unique("central-content-height")))
                .unwrap();
            assert!(height <= 492.1, "{page}: {height}");
        }
    }

    #[test]
    fn reward_button_commits_once_and_failed_write_keeps_wallet() {
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
        app.prepare_demo_review(3600, None);
        app.prepare_demo_menu("missoes", 0);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ce-e5-claim-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("journey.json");
        app.progress_path = Some(path.clone());
        let before = app.world.economy().clone();
        let mut position = None;
        for _ in 0..3 {
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(704.0, 676.0),
                    )),
                    ..Default::default()
                },
                |ui| app.central_menu(ui),
            );
            position = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == "Recolher" => {
                        Some(t.visual_bounding_rect().center())
                    }
                    _ => None,
                })
                .or(position);
            output.drop_without_applying_deltas();
        }
        let pos = position.expect("botão Recolher visível");
        for pressed in [true, false] {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(704.0, 676.0),
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
                |ui| app.central_menu(ui),
            )
            .drop_without_applying_deltas();
        }
        assert!(app.world.economy().balance > before.balance);
        let reopened = crate::progress::load_world(&path).unwrap().unwrap();
        assert_eq!(reopened.economy(), app.world.economy());
        let order_id = before.daily_orders[0].id;
        let claimed = app.world.economy().clone();
        app.perform_economy_action(EconomyAction::ClaimOrder { order_id });
        assert_eq!(app.world.economy(), &claimed);
        assert!(app.progress_notice.is_some());
        let mission = app
            .world
            .economy()
            .missions(app.world.journey_snapshot(), app.world.service_view())
            .into_iter()
            .find(|m| m.claimable())
            .unwrap();
        app.progress_path = Some(dir);
        app.perform_economy_action(EconomyAction::ClaimMission(mission.id.into()));
        assert_eq!(app.world.economy(), &claimed);
        assert!(app.progress_notice.is_some());
    }
}
