//! All coaches and locomotives in native scenes: day, dusk, night and rain.
//! Run with a new .ppm path; no profile is opened and no network is used.
use cpu_express::{
    config::Config,
    rendering::{
        Biome, CoachStyle, EngineStyle, HEIGHT, SceneContext, SceneOptions, SceneRenderer, WIDTH,
        Weather,
    },
    simulation::{Train, World},
};
use eframe::egui::{self, Color32};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

const SECOND_WAVE: [CoachStyle; 8] = [
    CoachStyle::Livestock,
    CoachStyle::Container,
    CoachStyle::Autorack,
    CoachStyle::PressureTank,
    CoachStyle::Crane,
    CoachStyle::Bicycle,
    CoachStyle::Coil,
    CoachStyle::Fair,
];

fn label(frame: &mut [Color32], title: &str) {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let painter = ui.ctx().layer_painter(egui::LayerId::background());
        cpu_express::ui_game::pixel_text(
            &painter,
            egui::pos2(4.0, 3.0),
            title,
            1.0,
            Color32::from_rgb(237, 222, 175),
        );
    });
    frame[..WIDTH * 13].fill(Color32::from_rgb(38, 55, 56));
    let shapes = std::mem::take(&mut output.shapes);
    output.drop_without_applying_deltas();
    for shape in shapes {
        if let egui::Shape::Mesh(mesh) = shape.shape {
            for quad in mesh.vertices.as_chunks::<4>().0 {
                let left = quad.iter().map(|v| v.pos.x as usize).min().unwrap();
                let right = quad.iter().map(|v| v.pos.x as usize).max().unwrap();
                let top = quad.iter().map(|v| v.pos.y as usize).min().unwrap();
                let bottom = quad.iter().map(|v| v.pos.y as usize).max().unwrap();
                for y in top..bottom.min(13) {
                    for x in left..right.min(WIDTH) {
                        frame[y * WIDTH + x] = quad[0].color;
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("informe um novo caminho .ppm")?;
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    let train = Train::new(&Config::default());
    let world = World::default();
    let mode = std::env::args().nth(2);
    let second_wave = matches!(mode.as_deref(), Some("--segunda" | "--recortes"));
    let crop = mode.as_deref() == Some("--recortes");
    let styles: &[CoachStyle] = if second_wave {
        &SECOND_WAVE
    } else {
        &CoachStyle::ALL
    };
    let (left, top, width, height) = if crop {
        (56, 31, 42, 28)
    } else {
        (0, 0, WIDTH, HEIGHT)
    };
    write!(output, "P6\n{} {}\n255\n", width * 4, height * styles.len())?;
    let mut renderer = SceneRenderer::new();
    renderer.set_scene_context(Some(SceneContext::new(Biome::GreenValley)));
    renderer.set_options(SceneOptions {
        clouds: false,
        ..SceneOptions::default()
    });
    for (row, &coach) in styles.iter().enumerate() {
        renderer.set_coaches([coach; 3]);
        renderer.set_engine(if second_wave {
            EngineStyle::Steam
        } else {
            EngineStyle::ALL[row % EngineStyle::ALL.len()]
        });
        let frames = [
            (90.0, Weather::Clear, "Dia"),
            (348.0, Weather::Clear, "Entardecer"),
            (450.0, Weather::Clear, "Noite"),
            (90.0, Weather::Rain, "Chuva"),
        ]
        .map(|(light, weather, condition)| {
            renderer.set_local_atmosphere(Some((weather, light < 400.0)));
            renderer.set_local_lighting(Some(light));
            let mut frame = renderer.render(&train, &world).to_vec();
            if second_wave && !crop {
                label(&mut frame, &format!("{} - {condition}", coach.name()));
            }
            frame
        });
        for y in top..top + height {
            for frame in &frames {
                for pixel in &frame[y * WIDTH + left..y * WIDTH + left + width] {
                    output.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
                }
            }
        }
    }
    output.flush()?;
    Ok(())
}
