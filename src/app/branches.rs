use super::*;
use crate::simulation::{BranchKind, BranchPhase, BranchState, BranchStop};
impl CpuExpressApp {
    pub(super) fn destination_name(&self) -> &'static str {
        self.world
            .excursion_snapshot()
            .current
            .map_or(self.world.journey_snapshot().next_station.name, |e| {
                e.station_name()
            })
    }
    pub(super) fn travel_progress(&self) -> f32 {
        self.world
            .excursion_snapshot()
            .current
            .map_or(self.world.journey_snapshot().segment_progress as f32, |e| {
                (e.active_seconds() / 188.0) as f32
            })
    }
    pub(super) fn branch_controls(&mut self, ui: &mut egui::Ui) {
        let branch = self.world.excursion_snapshot();
        if let Some(excursion) = branch.current {
            section(ui, "Excursão em andamento", |ui| {
                ui.strong(excursion.kind.stop().name());
                let phase = match excursion.phase {
                    BranchPhase::Outbound => "Ida",
                    BranchPhase::Service => "Visita ao destino",
                    BranchPhase::Return => "Retorno a Vila Aurora",
                };
                game::progress(
                    ui,
                    "excursion",
                    (excursion.active_seconds() / 188.0) as f32,
                    &format!(
                        "{phase} · {:.0} s restantes",
                        excursion.remaining_seconds().ceil()
                    ),
                    game::BLUE,
                );
                ui.small("O Circuito Aurora continua após o retorno. Passageiros e cargas a bordo permanecem preservados.");
            });
        } else if let Some(kind) = branch.pending {
            ui.strong(format!("Excursão agendada: {}", kind.stop().name()));
            ui.small("Partida automática de Vila Aurora após o serviço de plataforma.");
        } else {
            egui::CollapsingHeader::new("Explorar ramais").default_open(BranchState::unlocked(self.world.journey_snapshot())).show(ui,|ui| {
                let unlocked=BranchState::unlocked(self.world.journey_snapshot());
                ui.small(if unlocked {"Agende uma visita de 3 min 08 s de tempo ativo. Partida de Vila Aurora, retorno ao mesmo ponto."} else {"Complete um circuito para liberar os ramais."});
                ui.add_enabled_ui(unlocked,|ui| {
                    for kind in [BranchKind::Serra,BranchKind::Litoral] {
                        if game::button(ui,kind.stop().name(),Icon::Map,kind.stop().name(),false,egui::vec2(ui.available_width(),36.0)).clicked() {
                            match self.world.request_excursion(kind) {Ok(())=>{self.persist_progress(true);self.scene_dirty=true;},Err(error)=>self.notification=Some(error)}
                        }
                    }
                });
            });
        }
    }
    pub(super) fn branch_collection(&self, ui: &mut egui::Ui) {
        for stop in [BranchStop::MiranteDaSerra, BranchStop::CaisDasConchas] {
            let known = self.world.excursion_snapshot().discoveries.contains(&stop);
            section(ui, stop.name(), |ui| {
                ui.label(if known {
                    "Postal de excursão conquistado"
                } else {
                    "Explore este ramal para conquistar o postal"
                });
            });
        }
    }
}
