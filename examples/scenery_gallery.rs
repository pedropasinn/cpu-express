//! Original scenery contact sheet, using the same renderer as art_review.
//! Columns: daylight, sunset, night, rain. Row legend: docs/conteudo/CENARIO.md.
use cpu_express::{
    config::Config,
    rendering::{
        Biome, HEIGHT, SceneContext, SceneRenderer, SceneryContext, SceneryLifeOptions,
        SegmentScenery, WIDTH, Weather,
    },
    simulation::{
        Train, World,
        catalog::{Catalog, SceneryTag as T},
    },
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
    let motion = std::env::args().nth(2).as_deref() == Some("--vida");
    let still = std::env::args().nth(2).as_deref() == Some("--estatico");
    let width = if motion || still { WIDTH * 2 } else { WIDTH };
    let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    let samples: &[(Biome, &[T])] = &[
        (Biome::GreenValley, &[T::Plain]),
        (Biome::GreenValley, &[T::Farmland]),
        (Biome::Forest, &[T::Forest]),
        (Biome::Forest, &[T::Mountain]),
        (Biome::GreenValley, &[T::River]),
        (Biome::Forest, &[T::Bridge]),
        (Biome::Forest, &[T::Viaduct]),
        (Biome::Forest, &[T::Tunnel]),
        (Biome::GreenValley, &[T::Urban]),
        (Biome::Coast, &[T::Port]),
        (Biome::Coast, &[T::Coast]),
        (Biome::GreenValley, &[T::Plateau]),
        (Biome::Coast, &[T::Port, T::Coast, T::Plain]),
        (
            Biome::Forest,
            &[T::Mountain, T::Forest, T::River, T::Bridge],
        ),
        (
            Biome::Forest,
            &[T::Mountain, T::Forest, T::Viaduct, T::Tunnel],
        ),
        (Biome::GreenValley, &[T::Plateau, T::Farmland, T::Urban]),
    ];
    // Catalog access belongs to this offline exporter, never to the renderer.
    let line_rows;
    let rows = if std::env::args().nth(2).as_deref() == Some("--linha") {
        let catalog = Catalog::builtin()
            .as_ref()
            .map_err(|_| "catálogo inválido")?;
        let line = catalog.line("curitiba-paranagua").ok_or("linha ausente")?;
        line_rows = line
            .segments
            .iter()
            .map(|segment| {
                let biome = match segment.biome.as_str() {
                    "meadow" => Biome::GreenValley,
                    "forest" => Biome::Forest,
                    "highland" => Biome::HighDesert,
                    "coast" => Biome::Coast,
                    _ => unreachable!("bioma validado pelo catálogo"),
                };
                (biome, segment.scenery.as_slice())
            })
            .collect::<Vec<_>>();
        line_rows.as_slice()
    } else {
        samples
    };
    write!(out, "P6\n{} {}\n255\n", width * 4, HEIGHT * rows.len())?;
    let train = Train::new(&Config::default());
    let mut world = World::default();
    let distance = if motion || still {
        // The longer second leg keeps both stations outside the 640 px sheet.
        let route = cpu_express::simulation::Route::default();
        route.segments()[0].length_units + route.segments()[1].length_units * 0.5
    } else {
        1280.0
    };
    // A valid in-memory travel snapshot keeps Aurora platforms out of the
    // contact sheet. 1280 also preserves the near layer's authored tile origin.
    world.restore_journey(cpu_express::simulation::Journey::restore_checkpoint(
        cpu_express::simulation::DEFAULT_JOURNEY_SEED,
        distance,
        60.0,
    )?);
    let mut renderer = SceneRenderer::new();
    renderer.set_width(width);
    renderer.set_scenery_life_options(SceneryLifeOptions {
        reduce_motion: still,
    });
    for &(biome, tags) in rows {
        renderer.set_scene_context(Some(SceneContext::new(biome)));
        renderer.set_scenery_context(SceneryContext::new(SegmentScenery::from_tags(tags)));
        let states = if motion || still {
            let light = if tags.contains(&T::Forest)
                || tags.contains(&T::Coast)
                || tags.contains(&T::Mountain)
            {
                480.0
            } else {
                180.0
            };
            [0.0, 2.0, 8.0, 24.0].map(|time| (light, Weather::Clear, time))
        } else {
            [
                (180.0, Weather::Clear, 60.0),
                (348.0, Weather::Clear, 60.0),
                (480.0, Weather::Clear, 60.0),
                (180.0, Weather::Rain, 60.0),
            ]
        };
        let frames = states.map(|(light, weather, time)| {
            world.restore_journey(
                cpu_express::simulation::Journey::restore_checkpoint(
                    cpu_express::simulation::DEFAULT_JOURNEY_SEED,
                    distance,
                    time,
                )
                .expect("valid in-memory gallery snapshot"),
            );
            renderer.set_local_atmosphere(Some((weather, light < 400.0)));
            renderer.set_local_lighting(Some(light));
            renderer.render(&train, &world).to_vec()
        });
        for y in 0..HEIGHT {
            for frame in &frames {
                for pixel in &frame[y * width..(y + 1) * width] {
                    out.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
                }
            }
        }
    }
    out.flush()?;
    println!(
        "Galeria de cenário: {} linhas × 4 condições, pixels nativos.",
        rows.len()
    );
    Ok(())
}
