use super::*;
use crate::simulation::service::{CargoKind, ContractStatus, ServiceReward};
use crate::ui_services::{self as widgets, ArrivalItem, ArrivalSummary, CapacityView, ServiceView};

impl CpuExpressApp {
    pub(super) fn services_controls(&mut self, ui: &mut egui::Ui) {
        let mut automatic = self.world.service_state().automatic_contracts;
        if ui
            .checkbox(&mut automatic, "Despacho automático de encomendas")
            .changed()
        {
            self.world.set_automatic_contracts(automatic);
            self.persist_progress(true);
        }
        ui.small(if automatic {
            "Novas ofertas são aceitas nas paradas. Desative para escolher cada serviço; o contrato atual é mantido."
        } else {
            "Analise a oferta e aceite quando quiser. Sem prazo; passageiros continuam embarcando automaticamente."
        });
        let state = self.world.service_snapshot().clone();
        let mut accept = false;
        let mut cancel = false;
        let (seats, _) = state.capacity();
        widgets::capacity_meter(
            ui,
            "passenger-capacity",
            &CapacityView {
                label: "Passageiros a bordo",
                icon: Icon::Passenger,
                used: state.passenger_count(),
                total: seats,
            },
        );
        ui.small(
            "Embarque e entrega são automáticos nas estações. Sem prazos ou cobrança por ausência.",
        );
        if state.passengers.is_empty() {
            ui.label("Os próximos passageiros embarcarão na próxima parada com lugares livres.");
        }
        ui.add_space(8.0);
        if let Some(c) = &state.contract {
            let title = match c.kind {
                CargoKind::Parcels => "Correio ferroviário",
                CargoKind::Supplies => "Suprimentos da ferrovia",
                CargoKind::Liquid => "Abastecimento em tanque",
            };
            let status = match c.status {
                ContractStatus::Offered => "Oferta disponível · aguardando sua escolha",
                ContractStatus::Returning => {
                    "Devolução solicitada · carga retorna à origem sem pontos"
                }
                ContractStatus::Cancelled => "Cancelado · nenhuma carga pendente",
                ContractStatus::Accepted => "Aceito · aguardando origem e capacidade compatível",
                ContractStatus::Loaded => "Carregado · entrega automática no destino",
                ContractStatus::Delivered => "Entregue · pontos registrados",
            };
            let blocked = state.acceptance_error();
            accept = widgets::service_card(
                ui,
                &ServiceView {
                    id: "cargo-contract",
                    icon: Icon::Cargo,
                    title,
                    detail: &format!(
                        "{} unidades · {} até {} · {} pontos na entrega",
                        c.units,
                        c.origin.name(),
                        c.destination.name(),
                        c.reward
                    ),
                    status,
                    progress: None,
                    action_label: (c.status == ContractStatus::Offered)
                        .then_some("Aceitar encomenda"),
                    unavailable_reason: if c.status == ContractStatus::Offered {
                        blocked.as_deref()
                    } else {
                        None
                    },
                },
            )
            .inner;
            if matches!(
                c.status,
                ContractStatus::Offered | ContractStatus::Accepted | ContractStatus::Loaded
            ) {
                let label = match c.status {
                    ContractStatus::Offered => "Recusar oferta",
                    ContractStatus::Loaded => "Devolver carga à origem",
                    _ => "Cancelar encomenda",
                };
                cancel = game::button(
                    ui,
                    "cancel-cargo",
                    Icon::Cargo,
                    label,
                    false,
                    egui::vec2(ui.available_width(), 34.0),
                )
                .clicked();
                if c.status == ContractStatus::Loaded {
                    ui.small("A devolução mantém a carga a bordo até a origem. O espaço só é liberado ao descarregar; não há prêmio nem penalidade.");
                }
            }
            widgets::capacity_meter(
                ui,
                "cargo-capacity",
                &CapacityView {
                    label: "Compartimento compatível",
                    icon: Icon::Cargo,
                    used: state.cargo_count(),
                    total: state.cargo_capacity(c.kind),
                },
            );
        } else {
            ui.label("Uma encomenda será oferecida na próxima chegada.");
        }
        ui.add_space(8.0);
        for (index, group) in state.passengers.iter().enumerate() {
            ui.push_id(index, |ui| {
                widgets::service_card(
                    ui,
                    &ServiceView {
                        id: "passengers",
                        icon: Icon::Ticket,
                        title: &format!("{} passageiros", group.count),
                        detail: &format!(
                            "{} até {}",
                            group.origin.name(),
                            group.destination.name()
                        ),
                        status: "A bordo · desembarque automático",
                        progress: None,
                        action_label: None,
                        unavailable_reason: None,
                    },
                );
            });
        }
        ui.label(format!(
            "{} passageiros entregues · {} contratos concluídos · {} pontos de serviço",
            state.passengers_delivered, state.contracts_completed, state.credits
        ));
        ui.small("Pontos registram o serviço realizado; não são uma moeda de compra.");
        if accept || cancel {
            let result = if accept {
                self.world.accept_cargo_contract()
            } else {
                self.world.cancel_cargo_contract()
            };
            match result {
                Ok(()) => {
                    self.notification = Some(if accept {
                        "Encomenda aceita. Carregamento na origem com espaço compatível.".into()
                    } else {
                        "Solicitação registrada. Consulte o estado da encomenda acima.".into()
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
            ui.label("Seu primeiro bilhete de chegada aparecerá após a próxima parada.");
        }
        for arrival in state
            .arrivals
            .iter()
            .rev()
            .take(if last_only { 1 } else { 32 })
        {
            let passengers = format!(
                "{} embarcaram · {} desembarcaram",
                arrival.boarded, arrival.alighted
            );
            let cargo = format!(
                "{} carregadas · {} entregues · {} devolvidas",
                arrival.cargo_loaded, arrival.cargo_delivered, arrival.cargo_returned
            );
            let points = format!("+{} pontos de serviço", arrival.earned);
            widgets::arrival_summary(
                ui,
                ("arrival", arrival.visit),
                &ArrivalSummary {
                    station: arrival.station.name(),
                    detail: &format!(
                        "Chegada {} · registro salvo · últimas 32 chegadas",
                        arrival.visit
                    ),
                    items: &[
                        ArrivalItem {
                            icon: Icon::Passenger,
                            label: "Passageiros",
                            value: &passengers,
                        },
                        ArrivalItem {
                            icon: Icon::Cargo,
                            label: "Carga",
                            value: &cargo,
                        },
                    ],
                    reward: (arrival.earned > 0).then_some(points.as_str()),
                    action_label: None,
                },
            );
        }
    }

    pub(super) fn collection_controls(&mut self, ui: &mut egui::Ui) {
        let journey = self.world.journey_snapshot();
        ui.heading("Postais da ferrovia");
        ui.small("As descobertas e os serviços completam seu caderno.");
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
