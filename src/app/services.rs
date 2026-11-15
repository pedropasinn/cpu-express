use super::*;
use crate::simulation::service::{CargoKind, ContractStatus, ServiceReward};
use crate::ui_services::{self as widgets, ArrivalItem, ArrivalSummary, ServiceView};

impl CpuExpressApp {
    pub(super) fn services_controls(&mut self, ui: &mut egui::Ui) {
        use game::Sprite;
        let state = self.world.service_snapshot().clone();
        let mut accept = false;
        let mut cancel = false;
        let mut open_workshop = false;
        let art = |ui: &mut egui::Ui, sprite: Sprite, size: f32| {
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
            game::sprite(ui.painter(), rect, sprite);
        };

        // The order comes first; how offers are dispatched is a compact switch beside its title.
        ui.horizontal(|ui| {
            game::pixel_label(ui, "Encomenda", 2.0, game::INK);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let automatic = state.automatic_contracts;
                for (label, value, help) in [
                    (
                        "Manual",
                        false,
                        "Você aceita ou recusa cada oferta. Sem prazo.",
                    ),
                    ("Auto", true, "Ofertas são aceitas sozinhas nas paradas."),
                ] {
                    if game::button(
                        ui,
                        ("dispatch-mode", value),
                        Icon::Mail,
                        label,
                        automatic == value,
                        egui::vec2(104.0, 30.0),
                    )
                    .on_hover_text(help)
                    .clicked()
                        && automatic != value
                    {
                        self.world.set_automatic_contracts(value);
                        self.persist_progress(true);
                        self.notification = Some(
                            if value {
                                "Despacho automático: a próxima oferta é aceita na parada."
                            } else {
                                "Despacho manual: a encomenda atual é mantida."
                            }
                            .into(),
                        );
                    }
                }
                ui.label(egui::RichText::new("Despacho").color(game::MUTED));
            });
        });

        egui::Frame::new()
            .fill(game::CARD)
            .stroke(egui::Stroke::new(2.0, game::WOOD_DARK))
            .corner_radius(4)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let Some(c) = &state.contract else {
                    ui.horizontal(|ui| {
                        art(ui, Sprite::Encomenda, 32.0);
                        ui.label(
                            egui::RichText::new("Nova encomenda na próxima chegada")
                                .size(15.0)
                                .color(game::MUTED),
                        );
                    });
                    return;
                };
                let cargo_art = match c.kind {
                    CargoKind::Parcels => Sprite::Encomenda,
                    CargoKind::Liquid => Sprite::Tanque,
                    _ => Sprite::Caixote,
                };
                let wagon_hint = c.kind.wagon_hint();
                let capacity = state.cargo_capacity(c.kind);
                let deficit = c.units.saturating_sub(capacity);
                let origin = c.origin.name();
                let destination = c.destination.name();
                let (status_art, status) = match c.status {
                    ContractStatus::Offered => (Sprite::Bilhete, "Oferta · sem prazo".to_owned()),
                    ContractStatus::Accepted if deficit > 0 => (
                        Sprite::Relogio,
                        format!("Aguardando espaço · faltam {deficit} un. de carga"),
                    ),
                    ContractStatus::Accepted => {
                        (Sprite::Relogio, format!("Aguardando origem: {origin}"))
                    }
                    ContractStatus::Loaded => {
                        (cargo_art, format!("A bordo · entrega em {destination}"))
                    }
                    ContractStatus::Returning => (
                        Sprite::Devolver,
                        format!("Voltando à origem: {origin} · sem pontos"),
                    ),
                    ContractStatus::Delivered => (
                        Sprite::Confirmar,
                        format!("Entregue · +{} pontos registrados", c.reward),
                    ),
                    ContractStatus::Cancelled => (
                        Sprite::Bilhete,
                        "Cancelada · nova oferta na próxima chegada".to_owned(),
                    ),
                };
                ui.horizontal(|ui| {
                    let (tile, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(56.0), egui::Sense::hover());
                    ui.painter().rect_filled(tile, 4, game::PAPER);
                    ui.painter().rect_stroke(
                        tile,
                        4,
                        egui::Stroke::new(2.0, game::WOOD),
                        egui::StrokeKind::Inside,
                    );
                    game::sprite(ui.painter(), tile.shrink(4.0), cargo_art);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        ui.horizontal(|ui| {
                            game::pixel_label(ui, &format!("{} un.", c.units), 3.0, game::INK);
                            ui.add_space(12.0);
                            art(ui, Sprite::Pontos, 32.0);
                            let promised = !matches!(
                                c.status,
                                ContractStatus::Returning | ContractStatus::Cancelled
                            );
                            game::pixel_label(
                                ui,
                                &format!("+{}", c.reward),
                                2.0,
                                if promised { game::GREEN } else { game::MUTED },
                            )
                            .on_hover_text("Pontos de serviço: um registro, não uma moeda.");
                        });
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 5.0;
                            ui.label(egui::RichText::new(origin).size(15.0).strong());
                            art(ui, Sprite::SetaDireita, 16.0);
                            ui.label(egui::RichText::new(destination).size(15.0).strong());
                        });
                        ui.horizontal(|ui| {
                            art(ui, status_art, 16.0);
                            ui.label(egui::RichText::new(status).color(game::MUTED));
                        });
                    });
                });
                ui.add_space(4.0);
                match c.status {
                    ContractStatus::Offered if deficit > 0 => {
                        ui.label(
                            egui::RichText::new(format!("Faltam {deficit} espaços de carga"))
                                .size(15.0)
                                .strong()
                                .color(game::RED),
                        );
                        ui.label(format!("Equipe um {wagon_hint}."));
                        ui.horizontal(|ui| {
                            open_workshop = game::button(
                                ui,
                                "open-wagons",
                                Icon::Paint,
                                "Abrir vagões",
                                true,
                                egui::vec2(190.0, 40.0),
                            )
                            .clicked();
                            cancel = game::button(
                                ui,
                                "cancel-cargo",
                                Icon::Close,
                                "Recusar",
                                false,
                                egui::vec2(130.0, 32.0),
                            )
                            .clicked();
                        });
                    }
                    ContractStatus::Offered => {
                        ui.horizontal(|ui| {
                            accept = game::button(
                                ui,
                                "accept-cargo",
                                Icon::Check,
                                "Aceitar",
                                true,
                                egui::vec2(190.0, 40.0),
                            )
                            .clicked();
                            cancel = game::button(
                                ui,
                                "cancel-cargo",
                                Icon::Close,
                                "Recusar",
                                false,
                                egui::vec2(130.0, 32.0),
                            )
                            .clicked();
                        });
                    }
                    ContractStatus::Accepted => {
                        cancel = game::button(
                            ui,
                            "cancel-cargo",
                            Icon::Close,
                            "Cancelar",
                            false,
                            egui::vec2(150.0, 32.0),
                        )
                        .clicked();
                    }
                    ContractStatus::Loaded => {
                        cancel = game::button(
                            ui,
                            "cancel-cargo",
                            Icon::Cargo,
                            "Devolver à origem",
                            false,
                            egui::vec2(240.0, 32.0),
                        )
                        .clicked();
                        ui.label(
                            egui::RichText::new(
                                "O espaço é liberado ao chegar à origem. Sem prêmio nem penalidade.",
                            )
                            .size(12.0)
                            .color(game::MUTED),
                        );
                    }
                    _ => {}
                }
                ui.add_space(4.0);
                game::progress(
                    ui,
                    "cargo-capacity",
                    if capacity == 0 {
                        0.0
                    } else {
                        state.cargo_count() as f32 / capacity as f32
                    },
                    &format!("Carga a bordo {}/{}", state.cargo_count(), capacity),
                    game::BLUE,
                );
            });
        ui.add_space(8.0);

        // Passengers: one bar, then who is on board by destination.
        let (seats, _) = state.capacity();
        ui.horizontal(|ui| {
            game::pixel_label(ui, "Passageiros", 2.0, game::INK);
            game::hint(
                ui,
                "Embarque e desembarque são automáticos nas estações. Sem prazos.",
            );
        });
        game::progress(
            ui,
            "passenger-capacity",
            if seats == 0 {
                0.0
            } else {
                state.passenger_count() as f32 / seats as f32
            },
            &format!("{}/{} lugares", state.passenger_count(), seats),
            game::GREEN,
        );
        if seats == 0 {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Sem lugares para passageiros")
                        .strong()
                        .color(game::RED),
                );
                open_workshop |= game::button(
                    ui,
                    "open-wagons-seats",
                    Icon::Paint,
                    "Abrir vagões",
                    false,
                    egui::vec2(170.0, 30.0),
                )
                .clicked();
            });
        } else if state.passengers.is_empty() {
            ui.label(
                egui::RichText::new("Passageiros embarcam na próxima parada").color(game::MUTED),
            );
        }
        let mut by_destination: Vec<(&'static str, u32)> = Vec::new();
        for group in &state.passengers {
            let name = group.destination.name();
            match by_destination.iter_mut().find(|(n, _)| *n == name) {
                Some(entry) => entry.1 += group.count,
                None => by_destination.push((name, group.count)),
            }
        }
        for (destination, count) in &by_destination {
            ui.horizontal(|ui| {
                art(ui, Sprite::Passageiro, 32.0);
                game::pixel_label(ui, &count.to_string(), 2.0, game::INK);
                art(ui, Sprite::SetaDireita, 16.0);
                ui.label(egui::RichText::new(*destination).size(15.0));
            });
        }
        ui.add_space(8.0);

        // Lifetime record, never a balance to spend.
        ui.horizontal_wrapped(|ui| {
            for (sprite, value, caption) in [
                (Sprite::Passageiro, state.passengers_delivered, "entregues"),
                (Sprite::Caixote, state.contracts_completed, "encomendas"),
                (Sprite::Pontos, state.credits, "pontos de serviço"),
            ] {
                art(ui, sprite, 16.0);
                ui.label(
                    egui::RichText::new(format!("{value} {caption}"))
                        .size(12.0)
                        .color(game::MUTED),
                );
                ui.add_space(10.0);
            }
        });

        if open_workshop {
            self.panel_tab = PanelTab::Appearance;
            self.appearance_page = 0;
        }
        if accept || cancel {
            let before = state.contract.as_ref().map(|c| c.status);
            let result = if accept {
                self.world.accept_cargo_contract()
            } else {
                self.world.cancel_cargo_contract()
            };
            match result {
                Ok(()) => {
                    // Report what the transaction actually did, read back from the domain.
                    let now = self.world.service_snapshot().contract.clone();
                    self.notification = Some(match now {
                        Some(c) if c.status == ContractStatus::Loaded => {
                            format!("{} un. carregadas", c.units)
                        }
                        Some(c) if c.status == ContractStatus::Accepted => {
                            format!("Encomenda aceita · carrega em {}", c.origin.name())
                        }
                        Some(c) if c.status == ContractStatus::Returning => {
                            format!("Devolução solicitada · destino: {}", c.origin.name())
                        }
                        Some(c) if c.status == ContractStatus::Cancelled => {
                            if before == Some(ContractStatus::Offered) {
                                "Oferta recusada · nova oferta na próxima chegada".into()
                            } else {
                                "Encomenda cancelada".into()
                            }
                        }
                        _ => "Encomenda atualizada".into(),
                    });
                    self.persist_progress(true);
                    self.scene_dirty = true;
                }
                Err(error) => self.notification = Some(error),
            }
        }
    }

    pub(super) fn arrival_ticket(&self, ui: &mut egui::Ui, last_only: bool) {
        let state = self.world.service_snapshot();
        if state.arrivals.is_empty() {
            ui.label(
                egui::RichText::new(format!(
                    "Seu diário começa na próxima estação: {}",
                    self.destination_name()
                ))
                .color(game::MUTED),
            );
        }
        for arrival in state
            .arrivals
            .iter()
            .rev()
            .take(if last_only { 1 } else { 32 })
        {
            let passengers = format!(
                "+{} embarcaram / -{} desembarcaram",
                arrival.boarded, arrival.alighted
            );
            // Only what happened: a stop without cargo shows no row of zeros.
            let cargo = [
                (arrival.cargo_loaded, "carregadas"),
                (arrival.cargo_delivered, "entregues"),
                (arrival.cargo_returned, "devolvidas, sem pontos"),
            ]
            .iter()
            .filter(|(units, _)| *units > 0)
            .map(|(units, verb)| format!("{units} {verb}"))
            .collect::<Vec<_>>()
            .join(" · ");
            let points = format!("+{}", arrival.earned);
            let mut items = vec![ArrivalItem {
                icon: Icon::Passenger,
                label: "Passageiros",
                value: &passengers,
            }];
            if !cargo.is_empty() {
                items.push(ArrivalItem {
                    icon: Icon::Cargo,
                    label: "Carga",
                    value: &cargo,
                });
            }
            widgets::arrival_summary(
                ui,
                ("arrival", arrival.visit),
                &ArrivalSummary {
                    station: arrival.station.name(),
                    detail: &format!("Chegada {}", arrival.visit),
                    items: &items,
                    reward: (arrival.earned > 0).then_some(points.as_str()),
                    action_label: None,
                },
            );
        }
    }

    pub(super) fn collection_controls(&mut self, ui: &mut egui::Ui) {
        let journey = self.world.journey_snapshot();
        ui.heading("Postais da ferrovia");
        game::hint(ui, "As descobertas e os serviços completam seu caderno.");
        for station in self.world.journey().route().stations() {
            let known = journey.discovered_stations[station.id.index()];
            section(ui, station.name, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 48.0),
                    egui::Sense::hover(),
                );
                let colors = [
                    egui::Color32::from_rgb(169, 198, 132),
                    egui::Color32::from_rgb(93, 145, 112),
                    egui::Color32::from_rgb(155, 170, 185),
                    egui::Color32::from_rgb(113, 179, 203),
                ];
                ui.painter().rect_filled(
                    rect,
                    2,
                    if known {
                        colors[station.id.index()]
                    } else {
                        game::CARD
                    },
                );
                game::icon(
                    ui.painter(),
                    egui::Rect::from_center_size(rect.center(), egui::vec2(36.0, 36.0)),
                    if known { Icon::Station } else { Icon::Lock },
                    game::INK,
                );
                ui.label(if known {
                    "Descoberta registrada"
                } else {
                    "Visite esta estação para revelar seu postal"
                });
            });
        }
        self.branch_collection(ui);
        let state = self.world.service_snapshot();
        for (reward, title, detail) in [
            (
                ServiceReward::PassengerPostcard,
                "Companhia de viagem",
                "Entregue 20 passageiros · libera a Elétrica da Serra",
            ),
            (
                ServiceReward::CargoPostcard,
                "Correio sobre trilhos",
                "Entregue 20 unidades de carga · libera a Diesel do Porto",
            ),
        ] {
            let unlocked = state.collection.contains(&reward);
            widgets::service_card(
                ui,
                &ServiceView {
                    id: title,
                    icon: if unlocked { Icon::Star } else { Icon::Lock },
                    title,
                    detail,
                    status: if unlocked {
                        "Postal conquistado"
                    } else {
                        "Ainda não conquistado"
                    },
                    progress: None,
                    action_label: None,
                    unavailable_reason: None,
                },
            );
        }
    }
}
