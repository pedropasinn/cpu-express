use cpu_express::{
    rendering::EngineStyle,
    ui_game::Icon,
    ui_stock::{self, CompositionSlot, StockAction, StockCard},
};
use eframe::egui;

fn input(time: f64, key: bool) -> egui::RawInput {
    let events = if key {
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]
    } else {
        vec![]
    };
    egui::RawInput {
        time: Some(time),
        focused: true,
        events,
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(480.0, 320.0),
        )),
        ..Default::default()
    }
}

#[test]
fn locked_cards_are_inspectable_but_never_emit_equip() {
    for (locked, enabled) in [(false, true), (true, true), (true, false)] {
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "test-card",
            egui::ColorImage::filled([42, 28], egui::Color32::WHITE),
            egui::TextureOptions::NEAREST,
        );
        let mut result = None;
        for (time, key) in [(0.0, false), (0.1, true)] {
            let output = ctx.run_ui(input(time, key), |ui| {
                if !enabled {
                    ui.disable();
                }
                let card = ui_stock::stock_card(
                    ui,
                    "card",
                    &StockCard {
                        name: "Azul do Porto",
                        state: "Equipar pintura",
                        texture: &texture,
                        selected: false,
                        locked_reason: locked.then_some("Complete uma chegada"),
                    },
                );
                if !key {
                    card.response.request_focus();
                }
                result = Some(card);
            });
            output.drop_without_applying_deltas();
        }
        let result = result.expect("card response");
        assert_eq!(result.response.enabled(), enabled);
        assert_eq!(
            result.action,
            if !enabled {
                None
            } else if locked {
                Some(StockAction::Inspect)
            } else {
                Some(StockAction::Equip)
            }
        );
    }
}

#[test]
fn composition_preserves_slot_labels_and_reports_keyboard_selection() {
    let ctx = egui::Context::default();
    let slots = [
        CompositionSlot {
            label: "Vagão 1",
            name: "Passageiros",
            icon: Icon::Passenger,
        },
        CompositionSlot {
            label: "Vagão 2",
            name: "Carga",
            icon: Icon::Cargo,
        },
        CompositionSlot {
            label: "Vagão 3",
            name: "Correio",
            icon: Icon::Mail,
        },
    ];
    let mut chosen = None;
    let mut shapes = Vec::new();
    let mut bounds = egui::Rect::NOTHING;
    for (time, key) in [(0.0, false), (0.1, false), (0.2, true)] {
        let mut raw = input(time, key);
        if time == 0.1 {
            raw.events.push(egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let mut output = ctx.run_ui(raw, |ui| {
            let result = ui_stock::composition(ui, "composition", &slots, 1);
            bounds = result.response.rect;
            chosen = result.inner;
        });
        shapes = std::mem::take(&mut output.shapes);
        output.drop_without_applying_deltas();
    }
    assert_eq!(chosen, Some(0));
    assert!(bounds.width() <= 480.0);
    let mut labels = Vec::new();
    fn collect(shape: &egui::Shape, labels: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => labels.push(text.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, labels);
                }
            }
            _ => {}
        }
    }
    for shape in shapes {
        collect(&shape.shape, &mut labels);
    }
    for label in [
        "Vagão 1",
        "Vagão 2",
        "Vagão 3",
        "Locomotiva",
        "Vagão 2 · Carga",
    ] {
        assert!(
            labels.iter().any(|actual| actual == label),
            "missing {label}"
        );
    }
}

#[test]
fn engine_gallery_uploads_once_and_reuses_three_portraits() {
    let ctx = egui::Context::default();
    let mut gallery = None;
    let output = ctx.run_ui(input(0.0, false), |ui| {
        if gallery.is_none() {
            gallery = Some(ui_stock::StockGallery::new(ui.ctx()));
        }
    });
    output.drop_without_applying_deltas();
    let gallery = gallery.expect("gallery setup");
    let engines = [
        EngineStyle::Steam,
        EngineStyle::Diesel,
        EngineStyle::Electric,
    ];
    let ids = engines.map(|engine| gallery.engine(engine).id());
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[1], ids[2]);
    assert_ne!(ids[0], ids[2]);
    for engine in engines {
        assert_eq!(gallery.engine(engine).size(), [60, 29]);
        assert!(std::ptr::eq(gallery.engine(engine), gallery.engine(engine)));
    }
    for time in [0.1, 0.2, 0.3] {
        let output = ctx.run_ui(input(time, false), |ui| {
            for engine in engines {
                ui.image((gallery.engine(engine).id(), egui::vec2(60.0, 29.0)));
            }
        });
        let uploads = output.textures_delta.set.len();
        let frees = output.textures_delta.free.len();
        output.drop_without_applying_deltas();
        assert_eq!(
            uploads, 0,
            "drawing cached engines must not upload textures"
        );
        assert_eq!(frees, 0);
        assert_eq!(engines.map(|engine| gallery.engine(engine).id()), ids);
    }
}

#[test]
fn narrow_locked_cards_keep_full_names_and_requirements_inside_the_card() {
    for width in [160.0, 190.0] {
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "narrow-engine",
            egui::ColorImage::filled([60, 29], egui::Color32::WHITE),
            egui::TextureOptions::NEAREST,
        );
        let name = "Locomotiva elétrica da serra";
        let requirement = "Entregue vinte passageiros para liberar esta locomotiva";
        let mut bounds = egui::Rect::NOTHING;
        let mut output = ctx.run_ui(input(0.0, false), |ui| {
            ui.set_width(width);
            bounds = ui_stock::stock_card(
                ui,
                "narrow",
                &StockCard {
                    name,
                    state: "Equipar",
                    texture: &texture,
                    selected: false,
                    locked_reason: Some(requirement),
                },
            )
            .response
            .rect;
        });
        let shapes = std::mem::take(&mut output.shapes);
        output.drop_without_applying_deltas();
        assert!(bounds.width() <= width + 0.01);
        for expected in [name, requirement] {
            let text = shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == expected => Some(text),
                    _ => None,
                })
                .expect("the complete label must remain discoverable");
            let text_bounds = text.galley.rect.translate(text.pos.to_vec2());
            assert!(
                bounds.contains_rect(text_bounds),
                "{expected:?} overflows at width {width}"
            );
        }
    }
}
