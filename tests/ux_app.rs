use std::time::Duration;

use cpu_express::{
    app::{AppAction, CpuExpressApp, LaunchOptions, SimulationClock, shortcut_action},
    config::{Config, HudMode},
    simulation::{Train, World},
};
use eframe::{App, egui};

fn seconds(value: f64) -> Duration {
    Duration::from_secs_f64(value)
}

#[test]
fn clock_discards_pause_time_even_without_intermediate_frames() {
    let mut clock = SimulationClock::new(Duration::ZERO);
    assert!((clock.take_step(seconds(0.02)) - 0.02).abs() < 1e-9);
    clock.set_suspended(true, seconds(0.03));
    clock.set_suspended(false, seconds(600.0));
    assert_eq!(clock.take_step(seconds(600.0)), 0.0);
    assert!((clock.take_step(seconds(600.02)) - 0.02).abs() < 1e-9);
}

#[test]
fn suspended_clock_freezes_wheels_scenery_and_smoke() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    let mut clock = SimulationClock::new(Duration::ZERO);
    for index in 1..=120 {
        let dt = clock.take_step(seconds(f64::from(index) / 60.0));
        let distance = train.update(100.0, dt, &config);
        world.update(dt, distance, train.speed_kmh());
    }
    assert!(!world.smoke().is_empty());
    let speed = train.speed_kmh();
    let wheel_phase = train.wheel_phase();
    let world_time = world.elapsed_seconds();
    let scroll = world.scroll_px();
    let smoke: Vec<_> = world
        .smoke()
        .iter()
        .map(|particle| (particle.x, particle.y, particle.age, particle.size))
        .collect();

    clock.set_suspended(true, seconds(2.0));
    for elapsed in [3.0, 10.0, 300.0] {
        let dt = clock.take_step(seconds(elapsed));
        let distance = train.update(0.0, dt, &config);
        world.update(dt, distance, train.speed_kmh());
        assert_eq!(dt, 0.0);
        assert_eq!(distance, 0.0);
    }
    assert_eq!(train.speed_kmh(), speed);
    assert_eq!(train.wheel_phase(), wheel_phase);
    assert_eq!(world.elapsed_seconds(), world_time);
    assert_eq!(world.scroll_px(), scroll);
    assert_eq!(
        world
            .smoke()
            .iter()
            .map(|particle| (particle.x, particle.y, particle.age, particle.size))
            .collect::<Vec<_>>(),
        smoke
    );
    clock.set_suspended(false, seconds(300.0));
    let dt = clock.take_step(seconds(300.02));
    let distance = train.update(0.0, dt, &config);
    world.update(dt, distance, train.speed_kmh());
    assert!((world.elapsed_seconds() - world_time - 0.02).abs() < 1e-8);
    assert!(distance > 0.0 && distance < 2.0);
}

#[test]
fn clock_bounds_long_frames_and_handles_reversed_time() {
    let mut clock = SimulationClock::new(seconds(1.0));
    assert_eq!(clock.take_step(seconds(120.0)), 0.25);
    assert_eq!(clock.take_step(seconds(119.0)), 0.0);
    clock.set_suspended(true, seconds(120.0));
    assert_eq!(clock.take_step(seconds(1000.0)), 0.0);
}

#[test]
fn shortcut_contract_distinguishes_typing_from_commands() {
    let ordinary = [
        (egui::Key::Space, AppAction::TogglePause),
        (egui::Key::M, AppAction::CycleHud),
        (egui::Key::P, AppAction::TogglePin),
        (egui::Key::O, AppAction::ToggleControls),
        (egui::Key::F1, AppAction::ToggleHelp),
        (egui::Key::R, AppAction::Reposition),
    ];
    for (key, action) in ordinary {
        assert_eq!(
            shortcut_action(key, egui::Modifiers::NONE, false, false),
            Some(action)
        );
        assert_eq!(
            shortcut_action(key, egui::Modifiers::NONE, true, false),
            None
        );
    }
    assert_eq!(
        shortcut_action(egui::Key::Q, egui::Modifiers::CTRL, false, false),
        Some(AppAction::Quit)
    );
    assert_eq!(
        shortcut_action(egui::Key::Q, egui::Modifiers::CTRL, true, false),
        None
    );
    assert_eq!(
        shortcut_action(egui::Key::Q, egui::Modifiers::NONE, false, false),
        None
    );
    assert_eq!(
        shortcut_action(egui::Key::M, egui::Modifiers::CTRL, false, false),
        None
    );
}

#[test]
fn escape_closes_panels_and_never_exits() {
    for keyboard_in_use in [false, true] {
        assert_eq!(
            shortcut_action(
                egui::Key::Escape,
                egui::Modifiers::NONE,
                keyboard_in_use,
                true
            ),
            Some(AppAction::ClosePanel)
        );
        assert_eq!(
            shortcut_action(
                egui::Key::Escape,
                egui::Modifiers::NONE,
                keyboard_in_use,
                false
            ),
            None
        );
    }
}

struct Harness {
    ctx: egui::Context,
    app: CpuExpressApp,
    frame: eframe::Frame,
    time: f64,
}

impl Harness {
    fn new(config: Config) -> Self {
        let ctx = egui::Context::default();
        let app = CpuExpressApp::new_demo(
            &ctx,
            LaunchOptions {
                config,
                config_path: None,
                warning: None,
                cpu_override: Some(50.0),
                exit_after: None,
                diagnostics: None,
            },
        );
        Self {
            ctx,
            app,
            frame: eframe::Frame::_new_kittest(),
            time: 0.0,
        }
    }

    fn tick(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
        self.tick_viewport(events, true, false)
    }

    fn tick_viewport(
        &mut self,
        events: Vec<egui::Event>,
        focused: bool,
        minimized: bool,
    ) -> egui::FullOutput {
        self.time += 1.0 / 30.0;
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 140.0),
            )),
            time: Some(self.time),
            events,
            focused,
            ..Default::default()
        };
        let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        viewport.focused = Some(focused);
        viewport.minimized = Some(minimized);
        viewport.occluded = Some(false);
        self.ctx.run_ui(input, |ui| {
            self.app.logic(ui.ctx(), &mut self.frame);
            self.app.ui(ui, &mut self.frame);
        })
    }

    fn key(&mut self, key: egui::Key) -> egui::FullOutput {
        self.tick(vec![
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
        ])
    }

    fn pointer_button(&mut self, pos: egui::Pos2, pressed: bool) -> egui::FullOutput {
        self.tick(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ])
    }

    fn drag_requests_native_move(&mut self, start: egui::Pos2, end: egui::Pos2) -> bool {
        let mut requested = false;
        for output in [
            self.tick(vec![egui::Event::PointerMoved(start)]),
            self.pointer_button(start, true),
            self.tick(vec![egui::Event::PointerMoved(end)]),
            self.pointer_button(end, false),
        ] {
            requested |= output.viewport_output.values().any(|viewport| {
                viewport
                    .commands
                    .iter()
                    .any(|command| matches!(command, egui::ViewportCommand::StartDrag))
            });
            output.drop_without_applying_deltas();
        }
        requested
    }
}

fn text_in(shape: &egui::Shape, result: &mut String) {
    match shape {
        egui::Shape::Text(text) => {
            result.push_str(&text.galley.job.text);
            result.push('\n');
        }
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                text_in(shape, result);
            }
        }
        _ => {}
    }
}

fn painted_text(output: &egui::FullOutput) -> String {
    let mut text = String::new();
    for clipped in &output.shapes {
        text_in(&clipped.shape, &mut text);
    }
    text
}

fn requests_close(output: &egui::FullOutput) -> bool {
    output.viewport_output.values().any(|viewport| {
        viewport
            .commands
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::Close))
    })
}

fn find_text_center(shape: &egui::Shape, prefix: &str) -> Option<egui::Pos2> {
    match shape {
        egui::Shape::Text(text) if text.galley.job.text.trim().starts_with(prefix) => {
            Some(text.visual_bounding_rect().center())
        }
        egui::Shape::Vec(shapes) => shapes
            .iter()
            .find_map(|shape| find_text_center(shape, prefix)),
        _ => None,
    }
}

#[test]
fn demo_ui_stays_explicit_in_clean_hud_and_does_not_invent_ram() {
    let mut config = Config::default();
    config.set_hud_mode(HudMode::Clean);
    let mut harness = Harness::new(config);
    harness.tick(vec![]).drop_without_applying_deltas();
    let output = harness.tick(vec![]);
    let snapshot = harness.app.snapshot();
    assert!(snapshot.demo_mode);
    assert_eq!(snapshot.cpu_percent, 50.0);
    assert_eq!(snapshot.ram_percent, None);
    assert_eq!(snapshot.hud_mode, HudMode::Clean);
    let text = painted_text(&output);
    assert!(text.contains("DEMO"), "Missing demo marker: {text}");
    assert!(!text.contains("RAM 0%"));
    assert!(!requests_close(&output));
    output.drop_without_applying_deltas();
}

#[test]
fn headless_keyboard_pauses_and_esc_only_closes_open_panel() {
    let mut harness = Harness::new(Config::default());
    harness.tick(vec![]).drop_without_applying_deltas();
    harness.key(egui::Key::Space).drop_without_applying_deltas();
    assert!(harness.app.snapshot().paused);
    harness.key(egui::Key::Space).drop_without_applying_deltas();
    assert!(!harness.app.snapshot().paused);
    harness.key(egui::Key::O).drop_without_applying_deltas();
    assert!(harness.app.snapshot().controls_open);
    let output = harness.key(egui::Key::Escape);
    assert!(!harness.app.snapshot().controls_open);
    assert!(!requests_close(&output));
    output.drop_without_applying_deltas();
    let output = harness.key(egui::Key::Escape);
    assert!(!requests_close(&output));
    output.drop_without_applying_deltas();
    harness.key(egui::Key::F1).drop_without_applying_deltas();
    assert!(harness.app.snapshot().help_open);
    harness
        .key(egui::Key::Escape)
        .drop_without_applying_deltas();
    assert!(!harness.app.snapshot().help_open);
}

#[test]
fn start_paused_freezes_the_actual_app_scene_across_ui_frames() {
    let mut harness = Harness::new(Config {
        start_paused: true,
        ..Config::default()
    });
    harness.tick(vec![]).drop_without_applying_deltas();
    let before = harness.app.snapshot();
    for _ in 0..4 {
        harness.tick(vec![]).drop_without_applying_deltas();
    }
    let after = harness.app.snapshot();
    assert!(after.paused && after.effective_paused);
    assert_eq!(after.world_elapsed_seconds, before.world_elapsed_seconds);
    assert_eq!(after.scroll_px, before.scroll_px);
    assert_eq!(after.speed_kmh, before.speed_kmh);
    assert_eq!(after.journey, before.journey);
    assert_eq!(after.rendered_frames, before.rendered_frames);
}

#[test]
fn minimized_app_stops_rasterization_and_journey_then_resumes_without_catch_up() {
    let mut harness = Harness::new(Config::default());
    harness.tick(vec![]).drop_without_applying_deltas();
    let before = harness.app.snapshot();
    for _ in 0..8 {
        harness
            .tick_viewport(vec![], false, true)
            .drop_without_applying_deltas();
    }
    let hidden = harness.app.snapshot();
    assert!(hidden.effective_paused);
    assert_eq!(hidden.rendered_frames, before.rendered_frames);
    assert_eq!(hidden.rendered_frames_hidden, 0);
    assert_eq!(
        hidden.journey.total_distance_units,
        before.journey.total_distance_units
    );
    assert_eq!(hidden.journey.active_seconds, before.journey.active_seconds);
    harness.tick(vec![]).drop_without_applying_deltas();
    let resumed = harness.app.snapshot();
    assert!(!resumed.effective_paused);
    assert!(resumed.rendered_frames > hidden.rendered_frames);
    assert!(resumed.journey.total_distance_units - hidden.journey.total_distance_units < 1.0);
}

#[test]
fn actual_scene_drag_never_steals_toolbar_or_panel_interaction() {
    let mut scene = Harness::new(Config::default());
    scene.tick(vec![]).drop_without_applying_deltas();
    assert!(scene.drag_requests_native_move(egui::pos2(60.0, 80.0), egui::pos2(95.0, 80.0)));

    let mut toolbar = Harness::new(Config::default());
    let output = toolbar.tick(vec![]);
    let pause = output
        .shapes
        .iter()
        .find_map(|shape| find_text_center(&shape.shape, "Pausar"))
        .expect("the actual toolbar pause button is painted");
    output.drop_without_applying_deltas();
    assert!(!toolbar.drag_requests_native_move(pause, pause + egui::vec2(20.0, 0.0)));

    let mut panel = Harness::new(Config::default());
    panel.tick(vec![]).drop_without_applying_deltas();
    panel.app.dispatch(AppAction::ToggleControls, &panel.ctx);
    panel.tick(vec![]).drop_without_applying_deltas();
    assert!(!panel.drag_requests_native_move(egui::pos2(60.0, 80.0), egui::pos2(95.0, 80.0)));
}

#[test]
fn actual_cpu_numeric_editor_owns_keys_until_escape_closes_the_panel() {
    let mut harness = Harness::new(Config::default());
    harness.tick(vec![]).drop_without_applying_deltas();
    harness
        .app
        .dispatch(AppAction::ToggleControls, &harness.ctx);
    harness.tick(vec![]).drop_without_applying_deltas();
    let output = harness.tick(vec![]);
    let value_position = output
        .shapes
        .iter()
        .find_map(|shape| find_text_center(&shape.shape, "50.0"))
        .expect("the actual CPU slider value is painted inside the panel");
    output.drop_without_applying_deltas();
    harness
        .pointer_button(value_position, true)
        .drop_without_applying_deltas();
    harness
        .pointer_button(value_position, false)
        .drop_without_applying_deltas();
    harness.tick(vec![]).drop_without_applying_deltas();
    assert!(harness.ctx.egui_wants_keyboard_input());
    let original = harness.app.snapshot();
    harness.key(egui::Key::Space).drop_without_applying_deltas();
    harness.key(egui::Key::M).drop_without_applying_deltas();
    assert_eq!(harness.app.snapshot().paused, original.paused);
    assert_eq!(harness.app.snapshot().hud_mode, original.hud_mode);
    let output = harness.key(egui::Key::Escape);
    assert!(!harness.app.snapshot().controls_open);
    assert!(!requests_close(&output));
    output.drop_without_applying_deltas();
}

#[test]
fn tab_starts_on_the_toolbar_and_reveals_scrolled_panel_controls() {
    let mut harness = Harness::new(Config::default());
    harness.tick(vec![]).drop_without_applying_deltas();
    let output = harness.key(egui::Key::Tab);
    let pause_position = output
        .shapes
        .iter()
        .find_map(|shape| find_text_center(&shape.shape, "Pausar"))
        .expect("toolbar pause button");
    let focused = harness
        .ctx
        .memory(|memory| memory.focused())
        .and_then(|id| harness.ctx.read_response(id))
        .expect("Tab focuses an interactive widget");
    // A mocked native Frame may report docking unavailable (e.g. when running
    // plain `cargo test` outside Windows); that visible notice is also a valid
    // first header control. Neither path should focus the scenery.
    let focused_notice = output.shapes.iter().any(|shape| {
        find_text_center(&shape.shape, "!").is_some_and(|position| focused.rect.contains(position))
    });
    assert!(
        focused.rect.contains(pause_position) || focused_notice,
        "First Tab must reach a visible header control, not the scenery"
    );
    output.drop_without_applying_deltas();

    harness
        .app
        .dispatch(AppAction::ToggleControls, &harness.ctx);
    harness.tick(vec![]).drop_without_applying_deltas();
    let output = harness.tick(vec![]);
    assert!(output.shapes.iter().all(|shape| {
        find_text_center(&shape.shape, "Reposicionar")
            .is_none_or(|position| !shape.clip_rect.contains(position))
    }));
    output.drop_without_applying_deltas();

    let mut reached_reposition = false;
    for _ in 0..20 {
        harness.key(egui::Key::Tab).drop_without_applying_deltas();
        // Advance egui animation time without depending on wall-clock sleeps.
        for _ in 0..5 {
            harness.tick(vec![]).drop_without_applying_deltas();
        }
        let output = harness.tick(vec![]);
        let focused = harness
            .ctx
            .memory(|memory| memory.focused())
            .and_then(|id| harness.ctx.read_response(id));
        let visible_position = output.shapes.iter().find_map(|shape| {
            find_text_center(&shape.shape, "Reposicionar")
                .filter(|position| shape.clip_rect.contains(*position))
        });
        if let (Some(focused), Some(position)) = (focused, visible_position)
            && focused.rect.contains(position)
        {
            assert!(position.y > 0.0 && position.y < 140.0);
            reached_reposition = true;
        }
        output.drop_without_applying_deltas();
        if reached_reposition {
            break;
        }
    }
    assert!(
        reached_reposition,
        "Tab should scroll the initially hidden Reposicionar control into view"
    );
}
