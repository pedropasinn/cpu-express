use super::*;
use crate::simulation::{BranchExpedition, BranchKind, BranchPhase, BranchState, BranchStop};
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
                ui.scope(|ui| branch_map(ui, excursion, self.suspended()));
                ui.small("Circuito Aurora aguardando o retorno. Passageiros e cargas permanecem a bordo.");
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

// Presentation only: positions and ETAs use the domain's active phase clock.
fn phase_status(excursion: BranchExpedition, phase: BranchPhase) -> (f32, f64) {
    let order = |phase| match phase {
        BranchPhase::Outbound => 0,
        BranchPhase::Service => 1,
        BranchPhase::Return => 2,
    };
    match order(phase).cmp(&order(excursion.phase)) {
        std::cmp::Ordering::Less => (1.0, 0.0),
        std::cmp::Ordering::Equal => (
            excursion.progress() as f32,
            (phase.duration() - excursion.phase_seconds).max(0.0),
        ),
        std::cmp::Ordering::Greater => (0.0, phase.duration()),
    }
}

fn map_position(excursion: BranchExpedition) -> egui::Pos2 {
    let progress = excursion.progress() as f32;
    match excursion.phase {
        BranchPhase::Outbound => egui::pos2(progress, 0.0),
        BranchPhase::Service => egui::pos2(1.0, 0.5),
        BranchPhase::Return => egui::pos2(1.0 - progress, 1.0),
    }
}

fn branch_map(ui: &mut egui::Ui, excursion: BranchExpedition, suspended: bool) {
    ui.spacing_mut().item_spacing.y = 4.0;
    ui.strong(format!(
        "Vila Aurora > {} > Vila Aurora",
        excursion.kind.stop().name()
    ));
    let (map, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(0.0), 112.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(map);
    let landscape = match excursion.kind {
        BranchKind::Serra => egui::Color32::from_rgb(198, 213, 169),
        BranchKind::Litoral => egui::Color32::from_rgb(175, 202, 199),
    };
    painter.rect_filled(map, 3, landscape);
    painter.rect_stroke(
        map,
        3,
        egui::Stroke::new(1.0, game::EDGE),
        egui::StrokeKind::Inside,
    );
    let left = map.left() + map.width() * 0.12;
    let right = map.right() - map.width() * 0.12;
    let point = |x: f32, y: f32| egui::pos2(left + (right - left) * x, map.top() + 32.0 + y * 40.0);
    // Two tracks make the return direction unambiguous, without animation timers.
    for (phase, start, end) in [
        (BranchPhase::Outbound, point(0.0, 0.0), point(1.0, 0.0)),
        (BranchPhase::Service, point(1.0, 0.0), point(1.0, 1.0)),
        (BranchPhase::Return, point(1.0, 1.0), point(0.0, 1.0)),
    ] {
        painter.line_segment([start, end], egui::Stroke::new(6.0, game::EDGE));
        painter.line_segment([start, end], egui::Stroke::new(3.0, game::CARD));
        let (progress, _) = phase_status(excursion, phase);
        painter.line_segment(
            [start, start + (end - start) * progress],
            egui::Stroke::new(3.0, game::GREEN),
        );
    }
    for (x, label, align) in [
        (0.0, "Aurora", egui::Align2::LEFT_BOTTOM),
        (1.0, "Destino", egui::Align2::RIGHT_BOTTOM),
    ] {
        painter.text(
            point(x, 0.0) - egui::vec2(0.0, 16.0),
            align,
            label,
            egui::FontId::proportional(12.0),
            game::INK,
        );
        for y in [0.0, 1.0] {
            game::icon(
                &painter,
                egui::Rect::from_center_size(point(x, y), egui::vec2(18.0, 18.0)),
                Icon::Station,
                game::EDGE,
            );
        }
    }
    painter.text(
        point(0.5, 0.5),
        egui::Align2::CENTER_CENTER,
        "Ida >    < Volta",
        egui::FontId::proportional(12.0),
        game::INK,
    );
    let position = map_position(excursion);
    // During the visit the train stays at the destination platform.
    let train = point(position.x, position.y);
    painter.circle_filled(train, 12.0, game::GOLD);
    game::icon(
        &painter,
        egui::Rect::from_center_size(train, egui::vec2(20.0, 20.0)),
        Icon::Train,
        game::INK,
    );
    painter.text(
        egui::pos2(map.center().x, map.bottom() - 5.0),
        egui::Align2::CENTER_BOTTOM,
        if excursion.phase == BranchPhase::Service {
            "Em visita na plataforma"
        } else {
            "Trem no ramal"
        },
        egui::FontId::proportional(11.0),
        game::INK,
    );
    for (phase, label) in [
        (BranchPhase::Outbound, "Ida"),
        (BranchPhase::Service, "Visita"),
        (BranchPhase::Return, "Volta"),
    ] {
        let (progress, seconds) = phase_status(excursion, phase);
        let status = if phase == excursion.phase {
            format!(
                "{label} · {:.0}% · faltam {:.0} s",
                progress * 100.0,
                seconds.ceil()
            )
        } else if progress == 1.0 {
            format!("{label} · concluída")
        } else {
            format!("{label} · prevista · {:.0} s", seconds.ceil())
        };
        game::progress(
            ui,
            ("branch-phase", label),
            progress,
            &status,
            if phase == excursion.phase {
                game::BLUE
            } else {
                game::GREEN
            },
        );
    }
    ui.small(format!(
        "{}Retorno em {:.0} s ativos · total {:.0}%",
        if suspended { "Pausada · " } else { "" },
        excursion.remaining_seconds().max(0.0).ceil(),
        excursion.active_seconds() / 188.0 * 100.0
    ));
}

#[cfg(test)]
mod branch_map_tests {
    use super::*;

    #[test]
    fn phase_eta_and_tracks_follow_the_active_clock() {
        for kind in [BranchKind::Serra, BranchKind::Litoral] {
            for (phase, seconds, x, y, remaining) in [
                (BranchPhase::Outbound, 0.0, 0.0, 0.0, 90.0),
                (BranchPhase::Outbound, 45.0, 0.5, 0.0, 45.0),
                (BranchPhase::Service, 4.0, 1.0, 0.5, 4.0),
                (BranchPhase::Return, 45.0, 0.5, 1.0, 45.0),
                (BranchPhase::Return, 90.0, 0.0, 1.0, 0.0),
            ] {
                let trip = BranchExpedition {
                    kind,
                    phase,
                    phase_seconds: seconds,
                };
                assert_eq!(map_position(trip), egui::pos2(x, y));
                assert_eq!(phase_status(trip, phase).1, remaining);
                let sum: f64 = [
                    BranchPhase::Outbound,
                    BranchPhase::Service,
                    BranchPhase::Return,
                ]
                .into_iter()
                .map(|p| phase_status(trip, p).1)
                .sum();
                assert_eq!(sum, trip.remaining_seconds());
            }
        }
    }

    #[test]
    fn visit_keeps_train_at_destination_while_its_eta_counts_down() {
        for seconds in [0.0, 2.0, 7.9] {
            let trip = BranchExpedition {
                kind: BranchKind::Serra,
                phase: BranchPhase::Service,
                phase_seconds: seconds,
            };
            assert_eq!(map_position(trip), egui::pos2(1.0, 0.5));
            assert_eq!(phase_status(trip, BranchPhase::Outbound), (1.0, 0.0));
            assert_eq!(phase_status(trip, BranchPhase::Return), (0.0, 90.0));
        }
    }

    #[test]
    fn compact_map_renders_both_branches_and_all_phases_without_overflow() {
        for width in [240.0, 320.0, 640.0] {
            for kind in [BranchKind::Serra, BranchKind::Litoral] {
                for phase in [
                    BranchPhase::Outbound,
                    BranchPhase::Service,
                    BranchPhase::Return,
                ] {
                    let ctx = egui::Context::default();
                    let trip = BranchExpedition {
                        kind,
                        phase,
                        phase_seconds: 2.0,
                    };
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 400.0),
                        )),
                        ..Default::default()
                    };
                    let mut measured = egui::Vec2::ZERO;
                    let mut available = 0.0;
                    let output = ctx.run_ui(input, |ui| {
                        game::apply_style(ui);
                        available = ui.available_width();
                        branch_map(ui, trip, true);
                        measured = ui.min_rect().size();
                    });
                    let painted = !output.shapes.is_empty();
                    output.drop_without_applying_deltas();
                    assert!(painted);
                    assert!(measured.x <= available + 1.0);
                    assert!(measured.y < 300.0);
                }
            }
        }
    }
}
