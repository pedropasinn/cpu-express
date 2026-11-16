//! Mission presentation delegates all transactions to the economy domain.
use super::menus::pager;
use super::*;
use crate::simulation::economy::EconomyAction;

impl CpuExpressApp {
    pub(super) fn has_claimable_rewards(&self) -> bool {
        let e = self.world.economy();
        let j = self.world.journey_snapshot();
        let s = self.world.service_state();
        e.missions(j, s).iter().any(|m| m.claimable())
            || e.daily_orders.iter().any(|o| o.claimable(e, j, s))
    }

    pub(super) fn perform_economy_action(&mut self, action: EconomyAction) -> bool {
        let message = match &action {
            EconomyAction::SwapOrder { .. } => "Ordem trocada e salva.",
            EconomyAction::BuyCrate { .. } => "Remessa salva. Confira sua peça.",
            EconomyAction::BuyDirect(_) => "Peça comprada e salva.",
            EconomyAction::Reveal { .. } => "Remessa conferida e salva.",
            EconomyAction::BuyMuseumStamp => "Selo do museu comprado e salvo.",
            _ => "Prêmio recolhido e salvo.",
        };
        let result = if let Some(path) = &self.progress_path {
            crate::progress::economy_action(path, &mut self.world, action)
        } else {
            Err("Viagem temporária: abra um perfil salvo para comprar ou recolher.".into())
        };
        match result {
            Ok(warning) => {
                self.progress_notice = warning;
                self.shop.visit_summary = None;
                self.notification = Some(message.into());
                self.scene_dirty = true;
                true
            }
            Err(error) => {
                self.progress_notice = Some(error);
                false
            }
        }
    }

    pub(super) fn missions_page(&mut self, ui: &mut egui::Ui) {
        let e = self.world.economy();
        let j = self.world.journey_snapshot();
        let s = self.world.service_state();
        let missions: Vec<_> = e.missions(j, s).into_iter().filter(|m| m.visible).collect();
        let next = missions
            .iter()
            .filter(|m| !m.paid)
            .max_by(|a, b| (a.current.min(a.goal) * b.goal).cmp(&(b.current.min(b.goal) * a.goal)));
        egui::Frame::new()
            .fill(game::CARD)
            .inner_margin(6)
            .show(ui, |ui| {
                ui.strong(next.map_or("Todos os marcos recolhidos".into(), |m| {
                    format!("Próxima recompensa · {} · +{} réis", m.title, m.reward)
                }));
            });
        ui.horizontal(|ui| {
            ui.strong("Ordens do dia");
            ui.small("Sem prazo · uma nova a cada 10 min ativos, na chegada");
        });
        let range = pager(
            ui,
            "orders",
            &mut self.menu.orders,
            e.daily_orders.len().max(3),
            3,
        );
        let mut action = None;
        ui.columns(3, |cols| {
            for (column, index) in cols.iter_mut().zip(range) {
                egui::Frame::new().fill(game::CARD).inner_margin(6).show(column, |ui| {
                    ui.set_min_height(118.0);
                    if let Some(order) = e.daily_orders.get(index) {
                        ui.horizontal(|ui| {
                            let(rect,_)=ui.allocate_exact_size(egui::Vec2::splat(16.0),egui::Sense::hover());
                            game::icon(ui.painter(),rect,Icon::Mail,game::INK);
                            let title=order.template().map_or("Ordem", |t| t.title);
                            ui.add(egui::Label::new(title).truncate()).on_hover_text(title);
                        });
                        if order.telegram { ui.small("Telegrama extra"); }
                        let n = order.progress(e,j,s);
                        game::progress(ui, ("order", order.id), n as f32/order.goal as f32, &format!("{n}/{}", order.goal), game::GREEN);
                        ui.label(format!("+{} réis", order.reward));
                        ui.horizontal(|ui| {
                            if ui.add_enabled(order.claimable(e,j,s), egui::Button::new("Recolher")).clicked() { action = Some(EconomyAction::ClaimOrder { order_id: order.id }); }
                            let can_swap = self.world.station_wait() > 0.0 && j.active_seconds - e.last_order_active_seconds.max(e.last_order_swap_active_seconds) >= 600.0;
                            if ui.add_enabled(can_swap, egui::Button::new("Trocar")).on_hover_text("Troca gratuita na parada, após 10 min ativos da última ordem ou troca").clicked() { action = Some(EconomyAction::SwapOrder { order_id: order.id }); }
                        });
                    } else { ui.label("Aguardando ordem"); ui.small("Continue a viagem. A próxima chegada elegível traz uma tarefa possível para seu trem."); }
                });
            }
        });
        ui.horizontal(|ui| {
            ui.strong("Marcos");
            pager(ui, "missions", &mut self.menu.missions, missions.len(), 3);
        });
        for mission in missions.iter().skip(self.menu.missions * 3).take(3) {
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(245.0, 40.0), |ui| {
                    ui.label(mission.title);
                    game::progress(
                        ui,
                        mission.id,
                        mission.current as f32 / mission.goal as f32,
                        &format!("{} / {}", mission.current.min(mission.goal), mission.goal),
                        game::BLUE,
                    );
                });
                ui.label(format!("+{} réis", mission.reward));
                if ui
                    .add_enabled(
                        mission.claimable(),
                        egui::Button::new(if mission.paid {
                            "Recolhido"
                        } else {
                            "Recolher"
                        }),
                    )
                    .clicked()
                {
                    action = Some(EconomyAction::ClaimMission(mission.id.into()));
                }
            });
        }
        if let Some(action) = action {
            self.perform_economy_action(action);
        }
    }
}
