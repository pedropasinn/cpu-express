//! Native renderer crops: rows Steam/Diesel/Electric, columns Aurora/Ocean/
//! Sunset/Midnight. `cargo run --example engine_gallery -- target/engines.ppm`
use cpu_express::{
    config::Config,
    rendering::{EngineStyle, SceneRenderer, WIDTH, Weather},
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
    let mut sheet = vec![Color32::BLACK; 240 * 87];
    for (row, engine) in EngineStyle::ALL.into_iter().enumerate() {
        renderer.set_engine(engine);
        for (column, livery) in Livery::ALL.into_iter().enumerate() {
            renderer.set_livery(livery);
            let pixels = renderer.render(&train, &world);
            for y in 0..29 {
                let dst = (row * 29 + y) * 240 + column * 60;
                let src = (30 + y) * WIDTH + 180;
                sheet[dst..dst + 60].copy_from_slice(&pixels[src..src + 60]);
            }
        }
    }
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(output, "P6\n240 87\n255\n")?;
    for pixel in sheet {
        output.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
    }
    output.flush()?;
    Ok(())
}
