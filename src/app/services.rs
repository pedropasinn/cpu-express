use super::*;
use crate::simulation::service::{CargoKind, ContractStatus, ServiceReward};
use crate::ui_services::{self as widgets, ArrivalItem, ArrivalSummary, CapacityView, ServiceView};

impl CpuExpressApp {
    pub(super) fn services_controls(&mut self, ui: &mut egui::Ui) {
        let state = self.world.service_snapshot();
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
        ui.add_space(8.0);
        if let Some(c) = &state.contract {
            let title = match c.kind {
                CargoKind::Parcels => "Correio ferroviário",
                CargoKind::Supplies => "Suprimentos da ferrovia",
                CargoKind::Liquid => "Abastecimento em tanque",
            };
            let status = match c.status {
                ContractStatus::Accepted => "Aceito · aguardando origem e capacidade compatível",
                ContractStatus::Loaded => "Carregado · entrega automática no destino",
                ContractStatus::Delivered => "Entregue · pontos registrados",
            };
            widgets::service_card(
                ui,
                &ServiceView {
                    id: "cargo-contract",
                    icon: Icon::Cargo,
                    title,
                    detail: &format!(
                        "{} unidades · {} até {}",
                        c.units,
                        c.origin.name(),
                        c.destination.name()
                    ),
                    status,
                    progress: None,
                    action_label: None,
                    unavailable_reason: None,
                },
            );
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
        ui.label(format!(
            "{} passageiros entregues · {} contratos concluídos · {} pontos de serviço",
            state.passengers_delivered, state.contracts_completed, state.credits
        ));
        ui.small("Pontos registram o serviço realizado; não são uma moeda de compra.");
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
                "{} carregadas · {} entregues",
                arrival.cargo_loaded, arrival.cargo_delivered
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
