//! Optional visual evidence from the actual egui app, without a native window.
//!
//! CPU_EXPRESS_VISUAL_DIR=/absolute/path cargo test --test ux_visual -- --ignored
//! emits egui triangles/textures as JSON for an external software rasterizer.
//! This does not validate Windows, GPU output, DPI, or native window management.

use std::{collections::HashMap, fs, path::Path};

use cpu_express::{
    app::{AppAction, CpuExpressApp, LaunchOptions},
    config::{Config, HudMode},
};
use eframe::{App, egui};
use serde_json::json;

struct Texture {
    image: egui::ColorImage,
    linear: bool,
}

fn apply_textures(textures: &mut HashMap<egui::TextureId, Texture>, output: &mut egui::FullOutput) {
    for (id, deltas) in std::mem::take(&mut output.textures_delta.set) {
        for delta in deltas {
            let egui::ImageData::Color(image) = delta.image;
            if let Some([x, y]) = delta.pos {
                let texture = textures.get_mut(&id).expect("existing partial texture");
                for row in 0..image.size[1] {
                    let source = row * image.size[0];
                    let target = (y + row) * texture.image.size[0] + x;
                    texture.image.pixels[target..target + image.size[0]]
                        .copy_from_slice(&image.pixels[source..source + image.size[0]]);
                }
                texture.linear = delta.options.magnification == egui::TextureFilter::Linear;
            } else {
                textures.insert(
                    id,
                    Texture {
                        image: (*image).clone(),
                        linear: delta.options.magnification == egui::TextureFilter::Linear,
                    },
                );
            }
        }
    }
    output.textures_delta.clear();
}

struct Scenario<'a> {
    name: &'a str,
    hud_mode: HudMode,
    paused: bool,
    action: Option<AppAction>,
    idle: bool,
    size: [u32; 2],
    tab: Option<&'a str>,
    scroll: f32,
}

fn text_center(shape: &egui::Shape, text: &str) -> Option<egui::Pos2> {
    match shape {
        egui::Shape::Text(shape) if shape.galley.job.text == text => {
            Some(shape.visual_bounding_rect().center())
        }
        egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| text_center(shape, text)),
        _ => None,
    }
}

#[test]
#[ignore = "optional floating-panel artifact export"]
fn export_floating_panels() {
    let directory = std::env::var("CPU_EXPRESS_VISUAL_DIR").expect("explicit output directory");
    let directory = Path::new(&directory);
    fs::create_dir_all(directory).unwrap();
    for (name, tab) in [
        ("floating_trem", "Trem"),
        ("floating_viagem", "Viagem"),
        ("floating_objetivos", "Viagem/Objetivos"),
        ("floating_diario", "Viagem/Diário"),
        ("floating_visual", "Visual"),
        ("floating_janela", "Janela"),
        ("floating_pinturas", "Visual/Pinturas"),
        ("floating_clima", "Visual/Clima"),
        ("floating_fluidez", "Visual/Fluidez"),
        ("floating_visual_small", "Visual"),
    ] {
        export_scenario(
            directory,
            Scenario {
                name,
                hud_mode: HudMode::Clean,
                paused: true,
                action: Some(AppAction::ToggleControls),
                idle: false,
                size: if name.ends_with("small") {
                    [560, 600]
                } else {
                    [1000, 900]
                },
                tab: Some(tab),
                scroll: 0.0,
            },
        );
    }
}

fn export_scenario(directory: &Path, scenario: Scenario<'_>) {
    let Scenario {
        name,
        hud_mode,
        paused,
        action,
        idle,
        size,
        tab,
        scroll,
    } = scenario;
    let ctx = egui::Context::default();
    if name.starts_with("floating_") {
        ctx.set_embed_viewports(false);
    }
    let mut config = Config {
        start_paused: paused,
        auto_hide_controls: idle,
        ..Config::default()
    };
    config.set_hud_mode(hud_mode);
    let mut app = CpuExpressApp::new_demo(
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
    let mut frame = eframe::Frame::_new_kittest();
    let mut textures = HashMap::new();
    let mut latest: Option<egui::FullOutput> = None;
    let mut tab_position = None;
    for pass in 0..24 {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size[0] as f32, size[1] as f32),
            )),
            time: Some(f64::from(pass) / 30.0),
            focused: !idle,
            ..Default::default()
        };
        let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
        viewport.focused = Some(!idle);
        viewport.minimized = Some(false);
        viewport.occluded = Some(false);
        if pass == 1
            && let Some(action) = action
        {
            app.dispatch(action, &ctx);
        }
        if pass == 4
            && let Some(tab) = tab
        {
            tab_position = latest.as_ref().and_then(|output| {
                output
                    .shapes
                    .iter()
                    .find_map(|shape| text_center(&shape.shape, tab.split('/').next().unwrap()))
            });
            assert!(
                tab_position.is_some(),
                "the actual {tab} tab must be visible"
            );
        }
        if pass == 8
            && let Some(subtab) = tab.and_then(|tab| tab.split('/').nth(1))
        {
            tab_position = latest.as_ref().and_then(|output| {
                output
                    .shapes
                    .iter()
                    .find_map(|shape| text_center(&shape.shape, subtab))
            });
            assert!(tab_position.is_some(), "subtab {subtab} visible");
        }
        if (pass == 8 || pass == 9)
            && tab.is_some_and(|tab| tab.contains('/'))
            && let Some(pos) = tab_position
        {
            input.events.push(egui::Event::PointerMoved(pos));
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: pass == 8,
                modifiers: egui::Modifiers::NONE,
            });
        }
        if (pass == 4 || pass == 5)
            && let Some(pos) = tab_position
        {
            input.events.push(egui::Event::PointerMoved(pos));
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: pass == 4,
                modifiers: egui::Modifiers::NONE,
            });
        }
        if scroll != 0.0 && pass == 8 {
            input.events.push(egui::Event::PointerMoved(egui::pos2(
                size[0] as f32 - 40.0,
                100.0,
            )));
            input.events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -scroll),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let mut output = ctx.run_ui(input, |ui| {
            // Exercise presentation without a fake native docking error. This
            // export intentionally does not claim native lifecycle validation.
            app.ui(ui, &mut frame);
        });
        apply_textures(&mut textures, &mut output);
        latest = Some(output);
    }
    let output = latest.expect("at least one UI frame");
    let ppp = output.pixels_per_point;
    let meshes: Vec<_> = ctx
        .tessellate(output.shapes, ppp)
        .into_iter()
        .filter_map(|primitive| {
            let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else {
                return None;
            };
            Some(json!({
                "clip": [primitive.clip_rect.min.x, primitive.clip_rect.min.y,
                         primitive.clip_rect.max.x, primitive.clip_rect.max.y],
                "texture": format!("{:?}", mesh.texture_id),
                "vertices": mesh.vertices.iter().map(|vertex| json!([
                    vertex.pos.x, vertex.pos.y, vertex.uv.x, vertex.uv.y,
                    vertex.color.r(), vertex.color.g(), vertex.color.b(), vertex.color.a()
                ])).collect::<Vec<_>>(),
                "indices": mesh.indices,
            }))
        })
        .collect();
    let texture_images: Vec<_> = textures
        .into_iter()
        .map(|(id, texture)| {
            json!({
                "id": format!("{id:?}"),
                "size": texture.image.size,
                "linear": texture.linear,
                "pixels": texture.image.pixels.iter().map(|color| color.to_array()).collect::<Vec<_>>()
            })
        })
        .collect();
    fs::write(
        directory.join(format!("{name}.json")),
        serde_json::to_vec(&json!({
            "evidence": "Renderização headless do egui; sem janela nativa",
            "size": size,
            "scenario": name,
            "panel_tab": tab,
            "scrolled": scroll,
            "pixels_per_point": ppp,
            "textures": texture_images,
            "meshes": meshes,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "optional artifact export; requires an explicit output directory"]
fn export_actual_egui_layouts_for_visual_review() {
    let directory = std::env::var_os("CPU_EXPRESS_VISUAL_DIR")
        .expect("set CPU_EXPRESS_VISUAL_DIR to the export directory");
    let directory = Path::new(&directory);
    fs::create_dir_all(directory).unwrap();
    for (name, mode, paused, action, size, tab, scroll, idle) in [
        (
            "01_compacto",
            HudMode::Compact,
            false,
            None,
            [640, 140],
            None,
            0.0,
            false,
        ),
        (
            "02_detalhado",
            HudMode::Detailed,
            false,
            None,
            [640, 140],
            None,
            0.0,
            false,
        ),
        (
            "03_limpo",
            HudMode::Clean,
            false,
            None,
            [640, 140],
            None,
            0.0,
            false,
        ),
        (
            "04_pausa",
            HudMode::Compact,
            true,
            None,
            [640, 140],
            None,
            0.0,
            false,
        ),
        (
            "05_viagem",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Viagem"),
            0.0,
            false,
        ),
        (
            "06_visual",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Visual"),
            0.0,
            false,
        ),
        (
            "07_janela",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Janela"),
            0.0,
            false,
        ),
        (
            "08_compacto_320",
            HudMode::Compact,
            false,
            None,
            [320, 140],
            None,
            0.0,
            false,
        ),
        (
            "09_limpo_sem_foco",
            HudMode::Clean,
            false,
            None,
            [640, 140],
            None,
            0.0,
            true,
        ),
        (
            "10_visual_rolado",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Visual"),
            260.0,
            false,
        ),
        (
            "11_viagem_rolada",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Viagem"),
            140.0,
            false,
        ),
        (
            "12_ajuda",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleHelp),
            [640, 140],
            None,
            0.0,
            false,
        ),
        (
            "13_viagem_320",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [320, 140],
            Some("Viagem"),
            0.0,
            false,
        ),
        (
            "14_trem",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Trem"),
            0.0,
            false,
        ),
        (
            "15_visual_meio",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Visual"),
            140.0,
            false,
        ),
        (
            "16_janela_rolada",
            HudMode::Compact,
            false,
            Some(AppAction::ToggleControls),
            [640, 140],
            Some("Janela"),
            140.0,
            false,
        ),
        (
            "17_detalhado_320",
            HudMode::Detailed,
            false,
            None,
            [320, 140],
            None,
            0.0,
            false,
        ),
        (
            "18_pausa_320",
            HudMode::Compact,
            true,
            None,
            [320, 140],
            None,
            0.0,
            false,
        ),
    ] {
        export_scenario(
            directory,
            Scenario {
                name,
                hud_mode: mode,
                paused,
                action,
                idle,
                size,
                tab,
                scroll,
            },
        );
    }
}
