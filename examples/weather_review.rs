//! Renderer-only weather contact sheet: day/night columns, six conditions.
use cpu_express::{
    config::Config,
    rendering::{HEIGHT, SceneRenderer, WIDTH, Weather},
    simulation::{Train, World},
};
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
};
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "weather-review.ppm".into());
    let mut out = BufWriter::new(File::create(path)?);
    write!(out, "P6\n{} {}\n255\n", WIDTH * 4, HEIGHT * 12)?;
    let config = Config::default();
    let train = Train::new(&config);
    let mut world = World::default();
    world.update(12.0, 80.0, 30.0);
    for weather in [
        Weather::Clear,
        Weather::Cloudy,
        Weather::Fog,
        Weather::Rain,
        Weather::Thunderstorm,
        Weather::Snow,
    ] {
        let mut day = SceneRenderer::new();
        let mut night = SceneRenderer::new();
        day.set_local_atmosphere(Some((weather, true)));
        night.set_local_atmosphere(Some((weather, false)));
        let a = day.render(&train, &world);
        let b = night.render(&train, &world);
        for y in 0..HEIGHT {
            for _ in 0..2 {
                for pixels in [a, b] {
                    for p in &pixels[y * WIDTH..(y + 1) * WIDTH] {
                        for _ in 0..2 {
                            out.write_all(&[p.r(), p.g(), p.b()])?;
                        }
                    }
                }
            }
        }
    }
    out.flush()?;
    Ok(())
}
