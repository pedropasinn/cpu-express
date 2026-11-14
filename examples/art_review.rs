//! Real renderer contact sheet: four biomes, all coaches, travel and station service.
//! cargo run --locked --example art_review -- <output.ppm>
use cpu_express::{
    config::Config,
    rendering::{Biome, CoachStyle, HEIGHT, SceneContext, SceneOptions, SceneRenderer, WIDTH},
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
        .unwrap_or_else(|| "art-review.ppm".into());
    let mut output = BufWriter::new(File::create(&path)?);
    // Two columns: ordinary travel and service; four biome rows. Exact 2x pixels.
    write!(output, "P6\n{} {}\n255\n", WIDTH * 4, HEIGHT * 8)?;
    let config = Config::default();
    for biome in [
        Biome::GreenValley,
        Biome::Forest,
        Biome::HighDesert,
        Biome::Coast,
    ] {
        let mut train = Train::new(&config);
        let mut world = World::default();
        for _ in 0..150 {
            let distance = train.update(50.0, 1.0 / 30.0, &config);
            world.update(1.0 / 30.0, distance, train.speed_kmh());
        }
        let mut left = SceneRenderer::new();
        let mut right = SceneRenderer::new();
        for renderer in [&mut left, &mut right] {
            renderer.set_scene_context(Some(SceneContext::new(biome)));
            renderer.set_options(SceneOptions {
                day_cycle: false,
                ..SceneOptions::default()
            });
        }
        left.set_coaches([
            CoachStyle::Observation,
            CoachStyle::Sleeper,
            CoachStyle::Freight,
        ]);
        right.set_coaches([
            CoachStyle::Passenger,
            CoachStyle::Tank,
            CoachStyle::Observation,
        ]);
        let travel = left.render(&train, &world);
        world.set_station_wait(6.0);
        let service = right.render(&train, &world);
        for y in 0..HEIGHT {
            for _ in 0..2 {
                for pixels in [travel, service] {
                    for color in &pixels[y * WIDTH..(y + 1) * WIDTH] {
                        for _ in 0..2 {
                            output.write_all(&[color.r(), color.g(), color.b()])?;
                        }
                    }
                }
            }
        }
    }
    output.flush()?;
    println!("Real renderer review: {path}");
    Ok(())
}
