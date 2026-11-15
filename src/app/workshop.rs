use super::*;

// Capacity is descriptive; only the domain decides whether a change is allowed.
// Clone the service only for the option being inspected, never for every card/frame.
fn composition_preview(
    ui: &mut egui::Ui,
    response: &egui::Response,
    service: &crate::simulation::ServiceState,
    coaches: [CoachStyle; 3],
    equipment_notice: &mut Option<(String, Instant)>,
) -> bool {
    use crate::simulation::CargoKind;

    let composition = coaches.map(service_wagon);
    let before = service.capacity();
    let after = composition.iter().fold((0, 0), |(seats, cargo), wagon| {
        let capacity = wagon.capacity();
        (seats + capacity.0, cargo + capacity.1)
    });
    // Only what the swap changes, next to the piece that changes it.
    let delta = |now: u32, then: u32, unit: &str| match now.cmp(&then) {
        std::cmp::Ordering::Greater => Some(format!("+{} {unit}", now - then)),
        std::cmp::Ordering::Less => Some(format!("-{} {unit}", then - now)),
        std::cmp::Ordering::Equal => None,
    };
    let changes: Vec<String> = [
        delta(after.0, before.0, "lugares"),
        delta(after.1, before.1, "carga"),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !changes.is_empty() {
        ui.small(format!("Após trocar: {}", changes.join(" · ")));
    }
    if !response.hovered() && !response.has_focus() && !response.clicked() {
        return false;
    }

    let mut preview = service.clone();
    let validation = preview.set_composition(composition);
    if response.clicked()
        && let Err(error) = &validation
    {
        *equipment_notice = Some((
            format!("{error} Aguarde desembarque, entrega ou devolução em Viagem > Serviços."),
            Instant::now() + Duration::from_secs(6),
        ));
    }
    // A rejected transaction rolls back. Set only the disposable preview's stock
    // to query the proposed capacities, including the domain's cargo categories.
    preview.composition = composition;
    // Keep the grid stable while inspecting options with the mouse or keyboard.
    egui::Tooltip::for_widget(response).show(|ui| {
        ui.strong("Prévia da composição");
        ui.small(format!("Lugares: {} > {}", before.0, after.0));
        ui.small(format!(
            "A bordo: {} passageiros · {} carga",
            service.passenger_count(),
            service.cargo_count()
        ));
        for (label, kind) in [
            ("Encomendas/suprimentos", CargoKind::Parcels),
            ("Líquidos", CargoKind::Liquid),
        ] {
            ui.small(format!(
                "{label}: {} > {} un.",
                service.cargo_capacity(kind),
                preview.cargo_capacity(kind)
            ));
        }
        if let Err(error) = &validation {
            ui.colored_label(game::BLUE, format!("Bloqueado: {error}"));
            if let Some(contract) = &service.contract
                && service.cargo_count() > 0
            {
                let category = match contract.kind {
                    CargoKind::Parcels => "encomendas",
                    CargoKind::Supplies => "suprimentos",
                    CargoKind::Liquid => "líquidos",
                };
                ui.small(format!(
                    "{category}: {} a bordo / {} un. compatíveis após a troca.",
                    service.cargo_count(),
                    preview.cargo_capacity(contract.kind)
                ));
            }
            game::hint(
                ui,
                "Aguarde desembarque, entrega ou devolução em Viagem > Serviços.",
            );
        } else {
            game::hint(ui, "Cabe o serviço a bordo · pode equipar.");
        }
    });
    validation.is_ok()
}

impl CpuExpressApp {
    pub(super) fn appearance_controls(&mut self, ui: &mut egui::Ui) {
        game::apply_style(ui);
        if self.stock_gallery.is_none() {
            self.stock_gallery = Some(StockGallery::new(ui.ctx()));
        }
        let mut changed = false;
        let equipment_before = (
            self.config.visual.coaches,
            self.config.visual.livery,
            self.config.visual.engine,
        );
        let page_width = ((ui.available_width() - 32.0) / 5.0).floor().max(76.0);
        ui.horizontal_wrapped(|ui| {
            for (page, icon, label) in [
                (0, Icon::Train, "Vagões"),
                (1, Icon::Paint, "Pinturas"),
                (2, Icon::Weather, "Clima"),
                (3, Icon::Settings, "Fluidez"),
                (4, Icon::Train, "Motores"),
            ] {
                if game::button(
                    ui,
                    ("workshop-page", page),
                    icon,
                    label,
                    self.appearance_page == page,
                    egui::vec2(page_width, if page_width < 100.0 { 46.0 } else { 34.0 }),
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
                let slots =
                    self.config
                        .visual
                        .coaches
                        .map(|coach| crate::ui_stock::CompositionSlot {
                            label: "",
                            name: coach_name(coach),
                            icon: match coach {
                                CoachStyle::Freight | CoachStyle::Tank => Icon::Cargo,
                                _ => Icon::Passenger,
                            },
                        });
                let mut slots = slots;
                for (index, slot) in slots.iter_mut().enumerate() {
                    slot.label = ["Vagão 1", "Vagão 2", "Vagão 3"][index];
                }
                if let Some(slot) =
                    crate::ui_stock::composition(ui, "consist", &slots, self.coach_slot).inner
                {
                    self.coach_slot = slot;
                }
                let (seats, hold) = self.world.service_snapshot().capacity();
                // Totals of the train as numbers beside their objects.
                ui.horizontal(|ui| {
                    for (sprite, value, name) in [
                        (game::Sprite::Passageiro, seats, "Lugares do trem"),
                        (game::Sprite::Caixote, hold, "Espaço de carga do trem"),
                    ] {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::Vec2::splat(32.0), egui::Sense::hover());
                        game::sprite(ui.painter(), rect, sprite);
                        game::pixel_label(ui, &value.to_string(), 2.0, game::INK)
                            .on_hover_text(name);
                        ui.add_space(12.0);
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "Trocando o vagão {} · {}",
                            self.coach_slot + 1,
                            coach_name(self.config.visual.coaches[self.coach_slot])
                        ))
                        .color(game::MUTED),
                    );
                });
                egui::CollapsingHeader::new("Composições favoritas").show(ui, |ui| {
                    if ui
                        .add_enabled(
                            self.config.visual.favorite_consists.len() < 4
                                && !self
                                    .config
                                    .visual
                                    .favorite_consists
                                    .contains(&self.config.visual.coaches),
                            egui::Button::new("Guardar composição atual"),
                        )
                        .clicked()
                    {
                        self.config
                            .visual
                            .favorite_consists
                            .push(self.config.visual.coaches);
                        changed = true;
                    }
                    let mut remove = None;
                    for (index, coaches) in self
                        .config
                        .visual
                        .favorite_consists
                        .clone()
                        .into_iter()
                        .enumerate()
                    {
                        let response = ui
                            .horizontal_wrapped(|ui| {
                                let label = coaches.map(coach_name).join(" / ");
                                let response = ui.button(label);
                                if ui.small_button("Remover").clicked() {
                                    remove = Some(index);
                                }
                                response
                            })
                            .inner;
                        let allowed = composition_preview(
                            ui,
                            &response,
                            self.world.service_snapshot(),
                            coaches,
                            &mut self.equipment_notice,
                        );
                        if response.clicked() && allowed {
                            match self
                                .world
                                .set_service_composition(coaches.map(service_wagon))
                            {
                                Ok(()) => {
                                    self.config.visual.coaches = coaches;
                                    changed = true;
                                }
                                Err(error) => {
                                    self.equipment_notice =
                                        Some((error, Instant::now() + Duration::from_secs(5)))
                                }
                            }
                        }
                    }
                    if let Some(index) = remove {
                        self.config.visual.favorite_consists.remove(index);
                        changed = true;
                    }
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
                            let mut coaches = self.config.visual.coaches;
                            coaches[self.coach_slot] = style;
                            let texture = self.stock_gallery.as_ref().unwrap().coach(style);
                            let response = stock_card(
                                column,
                                texture,
                                coach_name(style),
                                if selected { "EQUIPADO" } else { "Equipar" },
                                selected,
                                true,
                            );
                            let (seats, hold) = service_wagon(style).capacity();
                            // The one capacity this wagon has, never a zero of the other kind.
                            column.label(
                                egui::RichText::new(match style {
                                    CoachStyle::Tank => format!("{hold} un. líquidas"),
                                    _ if seats > 0 => format!("{seats} lugares"),
                                    _ => format!("{hold} un. de carga"),
                                })
                                .strong(),
                            );
                            let allowed = composition_preview(
                                column,
                                &response,
                                self.world.service_snapshot(),
                                coaches,
                                &mut self.equipment_notice,
                            );
                            if response.clicked() && allowed && !selected {
                                if let Err(error) = self
                                    .world
                                    .set_service_composition(coaches.map(service_wagon))
                                {
                                    self.equipment_notice =
                                        Some((error, Instant::now() + Duration::from_secs(5)));
                                    continue;
                                }
                                self.config.visual.coaches = coaches;
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
                game::hint(
                    ui,
                    "Carga: encomendas e suprimentos · Tanque: líquidos. Sem bônus de conforto ou turismo.",
                );
            }
            1 => {
                ui.label(
                    egui::RichText::new("Pinturas da locomotiva")
                        .size(18.0)
                        .strong(),
                );
                game::hint(ui, "Cada chegada aproxima você de uma nova pintura.");
                for pair in crate::simulation::objectives::Livery::ALL.chunks(2) {
                    ui.columns(2, |columns| {
                        for (column, &livery) in columns.iter_mut().zip(pair) {
                            let unlocked =
                                self.demo_mode || self.world.service_snapshot().owns(livery);
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
                            if self.highlight_livery == Some(livery) {
                                response.scroll_to_me(Some(egui::Align::Center));
                                self.highlight_livery = None;
                            }
                            if response.clicked() && !unlocked {
                                self.equipment_notice = Some((
                                    format!(
                                        "{}: faltam {} chegadas. Consulte Viagem / Objetivos.",
                                        livery.name(),
                                        livery
                                            .required_stops()
                                            .saturating_sub(journey.stations_visited)
                                    ),
                                    Instant::now() + Duration::from_secs(6),
                                ));
                            }
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
                    ui.strong(&self.config.weather_location.city);
                    game::hint(ui, "Paisagem completa · clima e iluminação locais");
                    if let Some(snapshot) = &self.local_weather.last_good {
                        ui.label(format!(
                            "{} · {:.1} °C · {}",
                            snapshot.description(),
                            snapshot.temperature_c,
                            if snapshot.is_day { "Dia" } else { "Noite" }
                        ));
                        ui.small(format!(
                            "Condição das {} (horário local) · recebida há {} min",
                            snapshot.local_time(),
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                                .saturating_sub(snapshot.received_unix)
                                / 60
                        ));
                        if let Some(wind) = snapshot.wind_speed_kmh {
                            ui.small(format!("Vento: {wind:.0} km/h"));
                        }
                        if let Some(rain) = snapshot.precipitation_mm {
                            ui.small(format!("Precipitação no intervalo do modelo: {rain:.1} mm"));
                        }
                        if snapshot.sunrise_unix.is_some() && snapshot.sunset_unix.is_some() {
                            game::hint(
                                ui,
                                "Iluminação gradual entre amanhecer, dia, entardecer e noite locais.",
                            );
                        }
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
                        game::hint(
                            ui,
                            "Condições estimadas para a cidade; podem variar no seu bairro.",
                        );
                    }
                    if let Some(error) = &self.local_weather.cache_error {
                        ui.small(error);
                    }
                    egui::CollapsingHeader::new("Alterar local do clima").show(ui, |ui| {
                        ui.label("Cidade");
                        ui.text_edit_singleline(&mut self.weather_draft.city);
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Latitude"); ui.add(egui::DragValue::new(&mut self.weather_draft.latitude).range(-90.0..=90.0).speed(0.001).max_decimals(4));
                            ui.label("Longitude"); ui.add(egui::DragValue::new(&mut self.weather_draft.longitude).range(-180.0..=180.0).speed(0.001).max_decimals(4));
                        });
                        game::hint(ui, "Use as coordenadas da sua cidade. O nome é uma identificação; não há localização automática do computador.");
                        if ui.button("Usar São Paulo, SP").clicked() { self.weather_draft = Default::default(); }
                        if ui.button("Aplicar local").clicked() {
                            match self.local_weather.set_location(self.weather_draft.clone()) {
                                Ok(()) => { self.config.weather_location = self.weather_draft.clone(); changed = true; self.local_weather.refresh(ui.ctx()); }
                                Err(error) => self.local_weather.error = Some(error),
                            }
                        }
                    });
                    ui.hyperlink_to("Dados: Open-Meteo · CC BY 4.0", "https://open-meteo.com/");
                });
            }
            4 => {
                ui.heading("Locomotivas da coleção");
                game::hint(
                    ui,
                    "Novas silhuetas para os serviços que sua ferrovia realiza.",
                );
                for engine in crate::rendering::EngineStyle::ALL {
                    use crate::{rendering::EngineStyle, simulation::service::ServiceReward};
                    let requirement = match engine {
                        EngineStyle::Steam => None,
                        EngineStyle::Diesel => Some(ServiceReward::CargoPostcard),
                        EngineStyle::Electric => Some(ServiceReward::PassengerPostcard),
                    };
                    let unlocked = self.demo_mode
                        || requirement
                            .is_none_or(|r| self.world.service_snapshot().collection.contains(&r));
                    let description = match engine {
                        EngineStyle::Steam => "Vapor original · disponível desde o início",
                        EngineStyle::Diesel => "Entregue 20 unidades de carga",
                        EngineStyle::Electric => "Entregue 20 passageiros",
                    };
                    let selected = self.config.visual.engine == engine;
                    let texture = self.stock_gallery.as_ref().unwrap().engine(engine);
                    let response = stock_card(
                        ui,
                        texture,
                        engine.name(),
                        if selected { "EQUIPADA" } else { description },
                        selected,
                        unlocked,
                    );
                    if response.clicked() && !unlocked {
                        self.equipment_notice = Some((
                            format!(
                                "{} · {}. Consulte Viagem / Serviços.",
                                engine.name(),
                                description
                            ),
                            Instant::now() + Duration::from_secs(6),
                        ));
                    }
                    if response.clicked() && unlocked {
                        self.config.visual.engine = engine;
                        changed = true;
                        self.equipment_notice = Some((
                            format!("{} equipada", engine.name()),
                            Instant::now() + Duration::from_secs(3),
                        ));
                    }
                }
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
                    game::hint(
                        ui,
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
                    changed |= ui
                        .checkbox(
                            &mut self.config.reduce_menu_motion,
                            "Reduzir animações dos menus",
                        )
                        .changed();
                    changed |= ui
                        .checkbox(
                            &mut self.config.quiet_mode,
                            "Trabalho tranquilo: ocultar avisos sobre a paisagem",
                        )
                        .changed();
                    game::hint(
                        ui,
                        "A arte completa e o progresso da viagem são preservados. Consulte as chegadas no diário a qualquer momento.",
                    );
                });
                section(ui, "Sons da ferrovia", |ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.config.sound_volume, 0.0..=1.0)
                                .text("Volume dos eventos"),
                        )
                        .changed();
                    game::hint(
                        ui,
                        "Sons curtos de chegada, conquista e equipamento. Zero desliga; modo tranquilo mantém silêncio.",
                    );
                    if ui
                        .add_enabled(
                            self.config.sound_volume > 0.0 && !self.config.quiet_mode,
                            egui::Button::new("Ouvir exemplo"),
                        )
                        .clicked()
                    {
                        self.play_cue(crate::audio::AudioCue::Arrival);
                    }
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
            if equipment_before
                != (
                    self.config.visual.coaches,
                    self.config.visual.livery,
                    self.config.visual.engine,
                )
            {
                self.play_cue(crate::audio::AudioCue::Equip);
            }
            self.preferences_changed();
            self.next_frame = Instant::now();
            ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
        }
    }
}
