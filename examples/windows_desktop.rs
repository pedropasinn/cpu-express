//! Isolated integration demo: cargo run --locked --example windows_desktop.
//! The main game uses the same controller; this isolates native desktop diagnostics.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("Esta demonstração usa Win32 e deve ser executada no Windows 10/11.");
}

#[cfg(windows)]
fn main() {
    if let Err(error) = demo::run() {
        cpu_express::platform::show_error(&format!("Demonstração Windows: {error}"));
    }
}

#[cfg(windows)]
mod demo {
    use std::{
        error::Error,
        time::{Duration, Instant},
    };

    use cpu_express::platform::{DesktopController, DesktopState, DockEdge};
    use eframe::egui;

    pub fn run() -> Result<(), Box<dyn Error>> {
        let mut smoke = None;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg != "--smoke-seconds" {
                return Err(format!("Argumento desconhecido: {arg}").into());
            }
            let seconds: f64 = args
                .next()
                .ok_or("Faltam os segundos de --smoke-seconds")?
                .parse()?;
            if !seconds.is_finite() || !(0.1..=3600.0).contains(&seconds) {
                return Err("O intervalo deve estar entre 0,1 e 3600 segundos.".into());
            }
            smoke = Some(Duration::from_secs_f64(seconds));
        }
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([640.0, 220.0])
                .with_resizable(false)
                .with_decorations(false)
                .with_taskbar(true)
                .with_active(false)
                .with_window_level(egui::WindowLevel::AlwaysOnTop),
            renderer: eframe::Renderer::Glow,
            persist_window: false,
            ..Default::default()
        };
        eframe::run_native(
            "CPU Express — integração Windows",
            options,
            Box::new(move |cc| {
                let mut desktop = DesktopController::new(&cc.egui_ctx, true);
                let notice = desktop.enable_tray().err();
                desktop.dock()?;
                Ok(Box::new(Demo {
                    desktop,
                    notice,
                    started: Instant::now(),
                    smoke,
                    last_scale: None,
                }))
            }),
        )?;
        Ok(())
    }

    struct Demo {
        desktop: DesktopController,
        notice: Option<String>,
        started: Instant,
        smoke: Option<Duration>,
        last_scale: Option<f32>,
    }

    impl eframe::App for Demo {
        fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            // Only inspect the cached viewport scale, not monitor APIs each frame.
            let scale = ctx.input(|input| input.viewport().native_pixels_per_point);
            if self.last_scale.is_some() && scale != self.last_scale {
                self.notice = self.desktop.display_changed().err();
            }
            self.last_scale = scale;
            if let Some(timeout) = self.smoke {
                if self.started.elapsed() >= timeout {
                    let _ = self.desktop.close();
                    self.smoke = None;
                } else {
                    ctx.request_repaint_after(timeout.saturating_sub(self.started.elapsed()));
                }
            }
            // Required while hidden too: TrayEvent::Show must reach this code.
            let update = self.desktop.update(frame);
            if !update.errors.is_empty() {
                self.notice = Some(update.errors.join("\n"));
            }
        }

        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            egui::Frame::central_panel(ui.style()).show(ui, |ui| {
                let heading = ui.add(
                    egui::Label::new(egui::RichText::new("CPU EXPRESS · TESTE WINDOWS").strong())
                        .sense(egui::Sense::drag()),
                );
                if heading.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.label("Arraste o título. O ícone da bandeja permite mostrar, ocultar e sair.");
                ui.label("Esta janela demonstra a integração; o jogo continua em cpu-express.exe.");
                ui.horizontal(|ui| {
                    if ui.button("Mostrar").clicked() {
                        self.notice = self.desktop.show().err();
                    }
                    if ui
                        .add_enabled(self.desktop.tray_available(), egui::Button::new("Ocultar"))
                        .clicked()
                    {
                        self.notice = self.desktop.hide().err();
                    }
                    if ui.button("Minimizar").clicked() {
                        self.notice = self.desktop.minimize().err();
                    }
                    let mut topmost = self.desktop.always_on_top();
                    if ui.checkbox(&mut topmost, "Sempre no topo").changed() {
                        self.notice = self.desktop.set_topmost(topmost).err();
                    }
                    if ui.button("Sair").clicked() {
                        self.notice = self.desktop.close().err();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Reposicionar:");
                    for (label, edge) in [
                        ("Auto", DockEdge::Auto),
                        ("Baixo", DockEdge::Bottom),
                        ("Cima", DockEdge::Top),
                        ("Esquerda", DockEdge::Left),
                        ("Direita", DockEdge::Right),
                    ] {
                        if ui.button(label).clicked() {
                            self.notice = self.desktop.dock_at(edge).err();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    let tray = if self.desktop.tray_available() {
                        "disponível"
                    } else {
                        "indisponível"
                    };
                    ui.label(format!(
                        "Estado: {:?} · bandeja {tray}",
                        self.desktop.state()
                    ));
                    if ui.button("Desativar bandeja").clicked() {
                        self.notice = self.desktop.disable_tray().err();
                    }
                    if ui.button("Ativar bandeja").clicked() {
                        self.notice = self.desktop.enable_tray().err();
                    }
                });
                if let Some(message) = &self.notice {
                    ui.colored_label(egui::Color32::from_rgb(180, 65, 30), message);
                }
            });
            if ui.input(|input| input.key_pressed(egui::Key::Escape))
                && self.desktop.state() != DesktopState::Closed
            {
                self.notice = self.desktop.close().err();
            }
        }

        fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
            if let Err(error) = self.desktop.shutdown() {
                eprintln!("Falha ao encerrar demonstração: {error}");
            }
        }
    }
}
