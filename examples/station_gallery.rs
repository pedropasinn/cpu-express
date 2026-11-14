//! Original-renderer contact sheet: rows Pines/Ridge/Harbor/Aurora/Mirante/Conchas;
//! columns approach/service/departure. Native pixels (960 x 420), no UI or I/O
//! inside the renderer. `cargo run --example station_gallery -- target/stations.ppm`
use std::{error::Error, fs::OpenOptions, io::Write};

use cpu_express::{
    config::Config,
    rendering::{HEIGHT, SceneContext, SceneRenderer, StationService, WIDTH, Weather},
    simulation::{BranchKind, Journey, Train, World},
};
use eframe::egui::Color32;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("provide a new .ppm output path")?;
    let journey = Journey::default();
    let seed = journey.snapshot().seed;
    let train = Train::new(&Config::default());
    let mut renderer = SceneRenderer::new();
    renderer.set_local_atmosphere(Some((Weather::Clear, true)));
    let mut sheet = vec![Color32::BLACK; WIDTH * 3 * HEIGHT * 6];
    let mut distance = 0.0;
    for (row, segment) in journey.route().segments().iter().enumerate() {
        distance += segment.length_units;
        for (col, (offset, progress, wait)) in
            [(-100.0, 0.0, 0.0), (0.0, 0.5, 4.0), (300.0, 1.0, 0.0)]
                .into_iter()
                .enumerate()
        {
            let mut world = World::default();
            world.restore_journey(Journey::restore_checkpoint(seed, distance + offset, 180.5)?);
            world.set_station_wait(wait);
            renderer.set_scene_context(Some(SceneContext::from_journey(&world.journey_snapshot())));
            renderer.set_station_service(Some(StationService {
                progress,
                boarding: 3,
                alighting: 2,
            }));
            for (y, pixels) in renderer
                .render(&train, &world)
                .as_chunks::<WIDTH>()
                .0
                .iter()
                .enumerate()
            {
                let start = (row * HEIGHT + y) * WIDTH * 3 + col * WIDTH;
                sheet[start..start + WIDTH].copy_from_slice(pixels);
            }
        }
    }
    for (index, kind) in [BranchKind::Serra, BranchKind::Litoral]
        .into_iter()
        .enumerate()
    {
        for (col, seconds) in [86, 94, 110].into_iter().enumerate() {
            let config = Config::default();
            let mut branch_train = Train::new(&config);
            let mut world = World::default();
            world.restore_journey(Journey::restore_checkpoint(
                seed,
                journey.route().length_units(),
                1000.0,
            )?);
            world.request_excursion(kind)?;
            for _ in 0..seconds {
                world.advance_expedition(&mut branch_train, 40.0, 1.0, &config);
            }
            for (y, pixels) in renderer
                .render(&branch_train, &world)
                .as_chunks::<WIDTH>()
                .0
                .iter()
                .enumerate()
            {
                let start = ((index + 4) * HEIGHT + y) * WIDTH * 3 + col * WIDTH;
                sheet[start..start + WIDTH].copy_from_slice(pixels);
            }
        }
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    write!(file, "P6\n{} {}\n255\n", WIDTH * 3, HEIGHT * 6)?;
    for pixel in sheet {
        file.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
    }
    Ok(())
}
