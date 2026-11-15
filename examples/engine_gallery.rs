//! Native renderer crops: rows EngineStyle::ALL, columns Livery::ALL.
//! --clima shows the second wave in day/dusk/night/rain; --cenas uses full scenes.
use cpu_express::{
    config::Config,
    rendering::{EngineStyle, HEIGHT, SceneRenderer, WIDTH, Weather},
    simulation::{Train, World, objectives::Livery},
};
use eframe::egui::Color32;
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("provide a new .ppm output path")?;
    let train = Train::new(&Config::default());
    let world = World::default();
    let mut renderer = SceneRenderer::new();
    renderer.set_local_atmosphere(Some((Weather::Clear, true)));
    let mode = std::env::args().nth(2);
    let climate = matches!(mode.as_deref(), Some("--clima" | "--cenas"));
    let scenes = mode.as_deref() == Some("--cenas");
    let engines = if climate {
        &EngineStyle::ALL[6..]
    } else {
        &EngineStyle::ALL[..]
    };
    let (left, top, cell_width, cell_height) = if scenes {
        (0, 0, WIDTH, HEIGHT)
    } else {
        (180, 30, 60, 29)
    };
    let columns = if climate { 4 } else { Livery::ALL.len() };
    let width = columns * cell_width;
    let height = engines.len() * cell_height;
    let mut sheet = vec![Color32::BLACK; width * height];
    for (row, &engine) in engines.iter().enumerate() {
        renderer.set_engine(engine);
        for column in 0..columns {
            if climate {
                let (light, weather) = [
                    (90.0, Weather::Clear),
                    (348.0, Weather::Clear),
                    (450.0, Weather::Clear),
                    (90.0, Weather::Rain),
                ][column];
                renderer.set_local_atmosphere(Some((weather, light < 400.0)));
                renderer.set_local_lighting(Some(light));
                renderer.set_livery(Livery::ALL[4 + row]);
            } else {
                renderer.set_livery(Livery::ALL[column]);
            }
            let pixels = renderer.render(&train, &world);
            for y in 0..cell_height {
                let dst = (row * cell_height + y) * width + column * cell_width;
                let src = (top + y) * WIDTH + left;
                sheet[dst..dst + cell_width].copy_from_slice(&pixels[src..src + cell_width]);
            }
        }
    }
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(output, "P6\n{width} {height}\n255\n")?;
    for pixel in sheet {
        output.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
    }
    output.flush()?;
    Ok(())
}
