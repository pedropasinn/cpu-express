use cpu_express::rendering::SceneContext;
use eframe::egui::Color32;
// Station contact sheet following scenery_gallery/art_review's P6 pipeline.
// Six rows in StationStyle::ALL order; day, dusk, night and rain columns.
use cpu_express::{
    config::Config,
    rendering::{
        HEIGHT, SceneRenderer, StationService, StationStyle, StationStyles, WIDTH, Weather,
    },
    simulation::{BranchKind, Journey, Train, World},
};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("informe um novo caminho .ppm")?;
    if std::env::args().nth(2).as_deref() != Some("--estilos") {
        return legacy_gallery(&path);
    }
    let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(out, "P6\n{} {}\n255\n", WIDTH * 4, HEIGHT * 6)?;
    let train = Train::new(&Config::default());
    let mut world = World::default();
    world.set_station_wait(4.0);
    let mut renderer = SceneRenderer::new();
    renderer.set_station_service(Some(StationService {
        progress: 0.5,
        boarding: 2,
        alighting: 1,
    }));
    for style in StationStyle::ALL {
        renderer.set_station_styles(StationStyles {
            departing: Some(style),
            arriving: Some(style),
        });
        let frames = [
            (180.0, Weather::Clear),
            (348.0, Weather::Clear),
            (480.0, Weather::Clear),
            (180.0, Weather::Rain),
        ]
        .map(|(light, weather)| {
            renderer.set_local_lighting(Some(light));
            renderer.set_local_atmosphere(Some((weather, light < 400.0)));
            renderer.render(&train, &world).to_vec()
        });
        for y in 0..HEIGHT {
            for frame in &frames {
                for p in &frame[y * WIDTH..(y + 1) * WIDTH] {
                    out.write_all(&[p.r(), p.g(), p.b()])?;
                }
            }
        }
    }
    out.flush()?;
    println!("Seis arquiteturas fictícias × quatro condições, renderer real.");
    Ok(())
}

fn legacy_gallery(path: &str) -> Result<(), Box<dyn Error>> {
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
