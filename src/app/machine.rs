//! Presentation adapter; all monetary effects are applied by World on arrival.
use super::*;
use crate::simulation::machine::{MachineEffects, MachineSignals, MachineState};

impl CpuExpressApp {
    pub(super) fn apply_input_preference(&mut self) {
        if let Some(worker) = &self.worker {
            worker.set_input_enabled(self.config.use_input_activity);
        }
        if !self.config.use_input_activity {
            self.machine.clear_input();
            self.metrics.machine = self.metrics.machine.without_input();
        }
    }

    pub(super) fn update_machine(&mut self) {
        self.apply_input_preference();
        let now = self.opened.elapsed().as_secs_f64() + 60.0;
        if self.demo_mode {
            let mut signals = self
                .machine_demo_signals
                .unwrap_or_else(|| MachineSignals::demo(self.cpu_percent()));
            if !self.config.use_input_activity {
                signals = signals.without_input();
            }
            if !self.machine_demo_prepared {
                for second in 0..60 {
                    self.machine.observe(second as f64, signals);
                }
                self.machine_demo_prepared = true;
            }
            self.machine.observe(now, signals);
        } else if !self.metrics_stale() {
            let mut signals = self.metrics.machine;
            signals.cpu_percent = if matches!(self.metrics.status, MetricStatus::WarmingUp) {
                None
            } else {
                self.metrics.cpu_percent
            };
            signals.memory_percent = self.metrics.ram_percent;
            if !self.config.use_input_activity {
                signals = signals.without_input();
            }
            if let Some(age) = self.metrics.measured_at.checked_duration_since(self.opened) {
                self.machine.observe(age.as_secs_f64() + 60.0, signals);
            }
        }
        let limit = self
            .worker
            .as_ref()
            .map_or(3.0, |w| w.stale_after().as_secs_f64());
        self.machine_effects = self
            .machine
            .effects(now, limit, self.config.use_input_activity);
        if !self.demo_mode && self.metrics_stale() {
            self.machine_effects = MachineEffects::default();
        }
        self.world
            .set_machine_bonus(self.machine_effects.multiplier());
        if !self.suspended()
            && self.machine.take_telegram(
                self.world.economic_snapshot().active_seconds,
                self.machine_effects,
            )
        {
            self.world.request_telegram();
        }
    }

    pub(super) fn machine_page(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 3.0;
        let e = self.machine_effects;
        ui.heading("Sala de máquinas");
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
            game::sprite(ui.painter(), r, game::Sprite::Moeda);
            ui.colored_label(
                game::GREEN,
                egui::RichText::new(format!("+{:.0}% de moedas", (e.multiplier() - 1.0) * 100.0))
                    .size(23.0),
            );
            ui.small("Teto +30% · chegadas e entregas");
        });
        if self.demo_mode {
            ui.colored_label(
                game::BLUE,
                "DEMONSTRAÇÃO · leituras simuladas, sem gerar carga",
            );
        } else {
            ui.small("Média do último minuto · usar normalmente já basta");
        }
        let rows = [
            (
                game::Sprite::Cpu,
                "Caldeira · CPU",
                format!(
                    "+{:.0}% moedas · satura em 15% de CPU",
                    e.coin_parts[0] * 100.0
                ),
            ),
            (
                game::Sprite::Oficina,
                "Foguista · entrada",
                if !self.config.use_input_activity {
                    "Desligado por você".into()
                } else if e.quiet_march {
                    "Marcha tranquila · só visual".into()
                } else {
                    format!(
                        "+{:.0}% moedas · 20 s ativos/min bastam",
                        e.coin_parts[1] * 100.0
                    )
                },
            ),
            (
                game::Sprite::Passageiro,
                "Plataforma · memória",
                "Movimento de embarque · só visual".into(),
            ),
            (
                game::Sprite::VagaoCarga,
                "Guindaste · disco",
                "Trabalho no pátio · só visual".into(),
            ),
            (
                game::Sprite::Despacho,
                "Telégrafo · rede",
                format!(
                    "Fios animados · Linha viva +{:.0}% · telegramas",
                    e.coin_parts[2] * 100.0
                ),
            ),
            (
                game::Sprite::Tanque,
                "Tênder · energia",
                if e.low_power {
                    "Bateria baixa · menos animação".into()
                } else if e.on_ac_power == Some(true) {
                    "Na tomada · carvão e água completos".into()
                } else {
                    "Na bateria · ritmo normal".into()
                },
            ),
        ];
        for (index, (sprite, title, effect)) in rows.into_iter().enumerate() {
            egui::Frame::new()
                .fill(game::CARD)
                .inner_margin(3.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.spacing_mut().interact_size.y = 0.0;
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
                        game::sprite(ui.painter(), r, sprite);
                        let text_width = (ui.available_width() - 106.0).max(100.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(text_width, 32.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(text_width);
                                ui.horizontal(|ui| {
                                    ui.strong(title);
                                    ui.label(e.gauges[index].map_or_else(
                                        || "Sem leitura".into(),
                                        |v| format!("{:.0}% do efeito", v * 100.0),
                                    ));
                                });
                                ui.small(
                                    if e.gauges[index].is_none()
                                        && (index != 1 || self.config.use_input_activity)
                                    {
                                        "Sem leitura · sem efeito".to_owned()
                                    } else {
                                        effect
                                    },
                                );
                            },
                        );
                        ui.add(
                            egui::ProgressBar::new(e.gauges[index].unwrap_or(0.0))
                                .fill(game::GREEN)
                                .desired_width(90.0)
                                .desired_height(10.0),
                        );
                    });
                });
        }
        ui.small(format!(
            "Hodômetro: {:.0} min ativos de viagem · sem progresso offline",
            self.world.economic_snapshot().active_seconds / 60.0
        ));
        if ui
            .checkbox(
                &mut self.config.use_input_activity,
                "Usar atividade do teclado e mouse",
            )
            .changed()
        {
            self.preferences_changed();
            self.update_machine();
        }
        ui.small("Só presença de atividade; sem teclas, janelas ou histórico salvo.");
        ui.small("As leituras da máquina não saem do computador.");
    }

    /// Deterministic review fixtures, confined to the existing demonstration mode.
    pub fn prepare_demo_machine(&mut self, variant: usize) {
        if !self.demo_mode {
            return;
        }
        self.prepare_demo_store("maquina", 0);
        self.machine = MachineState::default();
        let mut signals = MachineSignals::demo(15.0);
        match variant {
            1 => signals = MachineSignals::default(),
            2 => {
                signals.on_ac_power = Some(false);
                signals.battery_percent = Some(12.0);
                signals.input_active = Some(false);
                signals.input_idle_seconds = Some(301);
            }
            3 => self.config.use_input_activity = false,
            _ => {}
        }
        for second in 0..60 {
            self.machine.observe(second as f64, signals);
        }
        self.machine_effects = self
            .machine
            .effects(59.0, 3.0, self.config.use_input_activity);
        self.machine_demo_prepared = true;
        self.machine_demo_signals = Some(signals);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app(ctx: &egui::Context) -> CpuExpressApp {
        CpuExpressApp::new_demo(
            ctx,
            LaunchOptions {
                config: Config::default(),
                config_path: None,
                warning: None,
                cpu_override: Some(15.0),
                exit_after: None,
                diagnostics: None,
            },
        )
    }

    #[test]
    fn adapter_applies_only_fresh_bonus_and_clears_opted_out_input() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        app.update_machine();
        assert!((app.world.machine_bonus() - 1.30).abs() < 0.001);
        app.config.use_input_activity = false;
        app.preferences_changed();
        assert_eq!(
            app.machine
                .effects(app.opened.elapsed().as_secs_f64() + 60.0, 3.0, true)
                .gauges[1],
            None,
            "opting out must erase input even without another scene frame"
        );
        app.update_machine();
        assert!((app.world.machine_bonus() - 1.15).abs() < 0.001);
        app.config.use_input_activity = true;
        app.update_machine();
        assert!(
            app.world.machine_bonus() < 1.16,
            "opt-in cannot recover erased history"
        );
        app.worker.take();
        app.demo_mode = false;
        app.metrics.measured_at = Instant::now() - Duration::from_secs(10);
        app.update_machine();
        assert_eq!(app.world.machine_bonus(), 1.0);
        assert_eq!(app.machine_effects, MachineEffects::default());
        assert!(app.progress_path.is_none());
    }

    #[test]
    fn six_gauges_and_dock_fit_every_machine_state() {
        let ctx = egui::Context::default();
        let mut app = app(&ctx);
        for variant in 0..4 {
            app.prepare_demo_machine(variant);
            for _ in 0..3 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(704.0, 676.0),
                        )),
                        ..Default::default()
                    },
                    |ui| app.central_menu(ui),
                );
                output.textures_delta.clear();
                let h = ctx
                    .data(|d| d.get_temp::<f32>(egui::Id::unique("central-content-height")))
                    .unwrap();
                assert!(h <= 492.1, "{variant}: height {h}");
                fn find(shape: &egui::Shape, label: &str) -> Option<egui::Rect> {
                    match shape {
                        egui::Shape::Text(t) if t.galley.job.text == label => {
                            Some(t.visual_bounding_rect())
                        }
                        egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, label)),
                        _ => None,
                    }
                }
                for label in [
                    "Caldeira · CPU",
                    "Foguista · entrada",
                    "Plataforma · memória",
                    "Guindaste · disco",
                    "Telégrafo · rede",
                    "Tênder · energia",
                    "Fechar",
                ] {
                    let r = output
                        .shapes
                        .iter()
                        .find_map(|s| find(&s.shape, label))
                        .expect(label);
                    assert!(
                        r.bottom() < 676.0 && r.right() < 704.0,
                        "{variant}: {label} {r:?}"
                    );
                }
                output.drop_without_applying_deltas();
            }
            let h = ctx
                .data(|d| d.get_temp::<f32>(egui::Id::unique("central-content-height")))
                .unwrap();
            let (w, limit) = ctx
                .data(|d| d.get_temp::<(f32, f32)>(egui::Id::unique("central-content-width")))
                .unwrap();
            assert!(
                h <= 492.1 && w <= limit + 0.1,
                "{variant}: {w}/{limit} × {h}"
            );
        }
    }
}
