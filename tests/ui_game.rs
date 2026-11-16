//! Headless interaction and paint contracts for the original game widgets.
use cpu_express::ui_game::{self, Icon};
use eframe::egui::{self, Event, Key, Modifiers, Rect, Response, Shape};

// Compile the presentation-only module before the integrator registers it.
#[path = "../src/ui_services.rs"]
mod ui_services;

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

#[test]
fn fractional_pixel_labels_reserve_their_painted_width_at_each_scale() {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(scale);
        let mut bounds = Rect::NOTHING;
        let output = frame(&ctx, 0.0, vec![], |ui| {
            bounds = ui_game::pixel_label(ui, "Cargas e oficina", 1.5, ui_game::GREEN).rect;
        });
        let mut painted = 0;
        for shape in output.shapes {
            if let Shape::Mesh(mesh) = shape.shape {
                for vertex in &mesh.vertices {
                    if vertex.color == ui_game::GREEN {
                        painted += 1;
                        assert!(
                            bounds.contains(vertex.pos),
                            "escala {scale}: {:?} fora de {bounds:?}",
                            vertex.pos
                        );
                    }
                }
            }
        }
        assert!(painted > 0);
    }
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

#[test]
fn reduced_menu_motion_is_context_local_and_preserves_keyboard_actions() {
    let ctx = egui::Context::default();
    let other = egui::Context::default();
    assert!(!ui_game::reduce_motion(&ctx));
    ui_game::set_reduce_motion(&ctx, true);
    assert!(ui_game::reduce_motion(&ctx));
    assert!(!ui_game::reduce_motion(&other));
    button_frame(&ctx, 0.0, vec![], true, true);
    let activated = button_frame(&ctx, 0.1, key_events(Key::Enter), true, false);
    assert!(activated.clicked());
    ui_game::set_reduce_motion(&ctx, false);
    assert!(!ui_game::reduce_motion(&ctx));
}

#[test]
fn new_service_icons_have_distinct_finite_pixel_meshes() {
    let ctx = egui::Context::default();
    let mut meshes = Vec::new();
    for icon in [Icon::Passenger, Icon::Cargo, Icon::Ticket, Icon::Mail] {
        let output = frame(&ctx, 0.0, vec![], |ui| {
            ui_game::icon(
                ui.painter(),
                Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(24.0, 24.0)),
                icon,
                ui_game::INK,
            );
        });
        let mesh = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Mesh(mesh) => Some(mesh.clone()),
                _ => None,
            })
            .expect("pixel icon mesh");
        assert!(!mesh.vertices.is_empty());
        assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
        assert!(
            !meshes.contains(&mesh),
            "icons must have distinct silhouettes"
        );
        meshes.push(mesh);
    }
}

#[test]
fn services_show_exact_facts_and_zero_capacity_without_nan() {
    use ui_services::*;
    let ctx = egui::Context::default();
    let output = frame(&ctx, 0.0, vec![], |ui| {
        let items = [ArrivalItem {
            icon: Icon::Passenger,
            label: "Passageiros entregues",
            value: "7",
        }];
        arrival_summary(
            ui,
            "arrival",
            &ArrivalSummary {
                station: "Porto do Sol",
                detail: "Serviço concluído",
                items: &items,
                reward: Some("Azul do Porto"),
                action_label: Some("Ver pintura liberada"),
            },
        );
        capacity_meter(
            ui,
            "capacity",
            &CapacityView {
                label: "Correio",
                icon: Icon::Mail,
                used: 0,
                total: 0,
            },
        );
    });
    let mut labels = Vec::new();
    for shape in &output.shapes {
        text_labels(&shape.shape, &mut labels);
    }
    for expected in [
        "Porto do Sol",
        "Passageiros entregues",
        "7",
        "Azul do Porto",
        "Ver pintura liberada",
        "Correio",
        "0 / 0",
        "Sem capacidade",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "missing {expected:?}"
        );
    }
    for primitive in ctx.tessellate(output.shapes, output.pixels_per_point) {
        if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
            assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
        }
    }
}

fn text_center(shape: &Shape, label: &str) -> Option<egui::Pos2> {
    match shape {
        Shape::Text(text) if text.galley.job.text == label => {
            Some(text.visual_bounding_rect().center())
        }
        Shape::Vec(shapes) => shapes.iter().find_map(|shape| text_center(shape, label)),
        _ => None,
    }
}

#[test]
fn service_dashboard_only_returns_enabled_actions() {
    use ui_services::*;
    for blocked in [false, true] {
        let ctx = egui::Context::default();
        let services = [ServiceView {
            id: "mail",
            icon: Icon::Mail,
            title: "Correio",
            detail: "Porto do Sol",
            status: "Aguardando",
            progress: Some(ServiceProgress {
                fraction: 0.25,
                label: "1 / 4",
            }),
            action_label: Some("Aceitar correio"),
            unavailable_reason: blocked.then_some("Requer vagão de carga"),
        }];
        let mut action = None;
        let output = frame(&ctx, 0.0, vec![], |ui| {
            action = service_dashboard(ui, "services", &services, "Sem serviços").inner;
        });
        assert!(action.is_none());
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| text_center(&shape.shape, "Aceitar correio"))
            .expect("action label");
        for (index, pressed) in [true, false].into_iter().enumerate() {
            frame(
                &ctx,
                0.1 + index as f64 * 0.1,
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                |ui| {
                    action = service_dashboard(ui, "services", &services, "Sem serviços").inner;
                },
            );
        }
        assert_eq!(action, if blocked { None } else { Some(0) });
    }
}

#[test]
fn compact_navigation_label_stays_inside_46_point_button() {
    let ctx = egui::Context::default();
    let mut bounds = egui::Rect::NOTHING;
    let output = frame(&ctx, 0.0, vec![], |ui| {
        bounds = cpu_express::ui_game::button(
            ui,
            "compact",
            cpu_express::ui_game::Icon::Star,
            "Objetivos",
            false,
            egui::vec2(76.0, 46.0),
        )
        .rect;
    });
    fn find(shapes: &[egui::epaint::ClippedShape], label: &str) -> Option<egui::Rect> {
        shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.visual_bounding_rect())
            }
            _ => None,
        })
    }
    let text = find(&output.shapes, "Objetivos").expect("painted label");
    assert!(bounds.contains_rect(text), "{text:?} outside {bounds:?}");
    assert!(
        text.center().y > bounds.center().y,
        "label must sit below the icon"
    );
}
