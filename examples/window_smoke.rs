#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;

struct WindowSmoke {
    opened: std::time::Instant,
    auto_close: bool,
}

impl eframe::App for WindowSmoke {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, egui::Color32::from_rgb(28, 49, 52));
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.heading(
                egui::RichText::new("CPU Express").color(egui::Color32::from_rgb(244, 221, 171)),
            );
            if ui.button("Fechar").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        if self.auto_close && self.opened.elapsed().as_secs() >= 3 {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 140.0])
            .with_resizable(false),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "CPU Express",
        options,
        Box::new(|_| {
            Ok(Box::new(WindowSmoke {
                opened: std::time::Instant::now(),
                auto_close: std::env::args().any(|arg| arg == "--smoke"),
            }))
        }),
    )
}
