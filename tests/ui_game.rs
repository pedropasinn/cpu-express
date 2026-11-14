//! Headless interaction and paint contracts for the original game widgets.
use cpu_express::ui_game::{self, Icon};
use eframe::egui::{self, Event, Key, Modifiers, Rect, Response, Shape};

struct PaintOutput {
    shapes: Vec<egui::epaint::ClippedShape>,
    pixels_per_point: f32,
}

fn frame(
    ctx: &egui::Context,
    time: f64,
    events: Vec<Event>,
    draw: impl FnMut(&mut egui::Ui),
) -> PaintOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(640.0, 360.0),
        )),
        time: Some(time),
        events,
        focused: true,
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .focused = Some(true);
    let mut output = ctx.run_ui(input, draw);
    let shapes = std::mem::take(&mut output.shapes);
    let pixels_per_point = output.pixels_per_point;
    // Always consume deltas before returning to any test assertions. Otherwise
    // an assertion panic can cause a second panic in TexturesDelta::drop.
    output.drop_without_applying_deltas();
    PaintOutput {
        shapes,
        pixels_per_point,
    }
}

fn key_events(key: Key) -> Vec<Event> {
    [true, false]
        .into_iter()
        .map(|pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        })
        .collect()
}

fn button_frame(
    ctx: &egui::Context,
    time: f64,
    events: Vec<Event>,
    enabled: bool,
    request_focus: bool,
) -> Response {
    let mut response = None;
    frame(ctx, time, events, |ui| {
        if !enabled {
            ui.disable();
        }
        let current = ui_game::button(
            ui,
            "journey-button",
            Icon::Play,
            "Seguir viagem",
            false,
            egui::vec2(190.0, 40.0),
        );
        if request_focus {
            current.request_focus();
        }
        response = Some(current);
    });
    response.expect("the frame must create a button")
}

fn assert_keyboard_activation(key: Key) {
    let ctx = egui::Context::default();
    let first = button_frame(&ctx, 0.0, vec![], true, true);
    assert!(!first.clicked());
    let focused = button_frame(&ctx, 0.1, vec![], true, false);
    assert!(
        focused.has_focus(),
        "focus must be established before input"
    );
    let activated = button_frame(&ctx, 0.2, key_events(key), true, false);
    assert!(activated.has_focus());
    assert!(activated.clicked(), "{key:?} should activate the button");
    let idle = button_frame(&ctx, 0.3, vec![], true, false);
    assert!(!idle.clicked(), "a released key must not activate again");
}

#[test]
fn enter_activates_focused_custom_button() {
    assert_keyboard_activation(Key::Enter);
}

#[test]
fn space_activates_focused_custom_button() {
    assert_keyboard_activation(Key::Space);
}

#[test]
fn disabled_button_rejects_keyboard_and_pointer_activation() {
    for key in [Key::Enter, Key::Space] {
        let ctx = egui::Context::default();
        button_frame(&ctx, 0.0, vec![], true, true);
        let focused = button_frame(&ctx, 0.1, vec![], true, false);
        assert!(focused.has_focus());
        // Disable a previously focused control in the same frame as the key.
        let disabled = button_frame(&ctx, 0.2, key_events(key), false, false);
        assert!(!disabled.enabled());
        assert!(!disabled.clicked(), "disabled button accepted {key:?}");
        let pos = disabled.rect.center();
        for (index, pressed) in [true, false].into_iter().enumerate() {
            let response = button_frame(
                &ctx,
                0.3 + index as f64 * 0.1,
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                false,
                false,
            );
            assert!(!response.enabled());
            assert!(!response.clicked(), "disabled button accepted a click");
        }
    }
}

fn text_labels(shape: &Shape, labels: &mut Vec<String>) {
    match shape {
        Shape::Text(text) => labels.push(text.galley.job.text.clone()),
        Shape::Vec(shapes) => {
            for shape in shapes {
                text_labels(shape, labels);
            }
        }
        _ => {}
    }
}

#[test]
fn labels_are_exact_separate_discoverable_text_shapes() {
    let ctx = egui::Context::default();
    let labels = ["Salvar viagem agora", "Pausar", "Opções — São Paulo"];
    let output = frame(&ctx, 0.0, vec![], |ui| {
        for (index, label) in labels.iter().enumerate() {
            ui_game::button(
                ui,
                index,
                Icon::Save,
                label,
                index == 1,
                egui::vec2(300.0, 40.0),
            );
        }
        ui_game::progress(ui, "progress", 0.5, "Próxima estação: 50%", ui_game::GREEN);
        ui.add_enabled_ui(false, |ui| {
            ui_game::button(
                ui,
                "disabled",
                Icon::Lock,
                "Pintura bloqueada",
                false,
                egui::vec2(300.0, 40.0),
            );
        });
    });
    let mut painted = Vec::new();
    for clipped in &output.shapes {
        text_labels(&clipped.shape, &mut painted);
    }
    for expected in labels
        .into_iter()
        .chain(["Próxima estação: 50%", "Pintura bloqueada"])
    {
        assert_eq!(
            painted
                .iter()
                .filter(|text| text.as_str() == expected)
                .count(),
            1,
            "expected one standalone, unmodified text shape for {expected:?}; got {painted:?}",
        );
    }
}

fn progress_frame(ctx: &egui::Context, time: f64, value: f32) -> PaintOutput {
    frame(ctx, time, vec![], |ui| {
        ui_game::progress(ui, "progress", value, "Viagem", ui_game::BLUE);
    })
}

#[test]
fn invalid_progress_produces_finite_shapes_and_clamps_the_fill() {
    let ctx = egui::Context::default();
    for (index, (value, expected)) in [
        (f32::NAN, 0.0),
        (f32::INFINITY, 0.0),
        (f32::NEG_INFINITY, 0.0),
        (-1.0, 0.0),
        (f32::MAX, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let actual = progress_frame(&ctx, index as f64, value);
        let reference = progress_frame(&ctx, index as f64 + 0.1, expected);
        assert!(!actual.shapes.is_empty());
        // Compare the entire paint output with a valid endpoint: this catches
        // bogus fill widths as well as NaN geometry hidden by bounding boxes.
        assert_eq!(actual.shapes, reference.shapes, "input {value:?}");
        for clipped in &actual.shapes {
            assert!(clipped.clip_rect.is_finite());
            if !matches!(clipped.shape, Shape::Noop) {
                assert!(clipped.shape.visual_bounding_rect().is_finite());
            }
        }
        let primitives = ctx.tessellate(actual.shapes, actual.pixels_per_point);
        assert!(!primitives.is_empty());
        let mut vertices = 0;
        for primitive in primitives {
            assert!(primitive.clip_rect.is_finite());
            if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                vertices += mesh.vertices.len();
                for vertex in mesh.vertices {
                    assert!(vertex.pos.is_finite(), "non-finite vertex for {value:?}");
                    assert!(vertex.uv.is_finite(), "non-finite UV for {value:?}");
                }
            }
        }
        assert!(
            vertices > 0,
            "the bar must actually produce visible geometry"
        );
    }
}
