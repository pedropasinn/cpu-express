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
    let mode = std::env::args().nth(2).unwrap_or_default();
    if mode.starts_with("--elementos") {
        return elements(&path, mode.contains("noite"));
    }
    let expansion = mode == "--expansao" || mode == "--faixa60" || mode == "--aproximacao";
    let strip = mode == "--faixa60";
    let approach = mode == "--aproximacao";
    let motion = std::env::args().nth(2).as_deref() == Some("--vida");
    let still = std::env::args().nth(2).as_deref() == Some("--estatico");
    let width = if motion || still || expansion {
        WIDTH * 2
    } else {
        WIDTH
    };
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
    let expanded: &[(Biome, &[T])] = &[
        (Biome::GreenValley, &[T::Village]),
        (Biome::GreenValley, &[T::Town]),
        (Biome::GreenValley, &[T::City]),
        (Biome::GreenValley, &[T::Industrial]),
        (Biome::Coast, &[T::Port]),
        (Biome::GreenValley, &[T::Suburb]),
    ];
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
    } else if strip {
        &expanded[2..3]
    } else if expansion {
        expanded
    } else {
        samples
    };
    write!(
        out,
        "P6\n{} {}\n255\n",
        width * 4,
        if strip {
            60 * rows.len()
        } else {
            HEIGHT * rows.len()
        }
    )?;
    let train = Train::new(&Config::default());
    let mut world = World::default();
    let distance = if motion || still || expansion {
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
        let mut column = 0;
        let frames = states.map(|(mut light, mut weather, time)| {
            use cpu_express::rendering::SettlementContext;
            renderer.set_settlement_context(SettlementContext {
                seed: cpu_express::simulation::DEFAULT_JOURNEY_SEED,
                density: if approach {
                    [0.0, 0.35, 0.7, 1.0][column]
                } else {
                    1.0
                },
            });
            column += 1;
            if approach {
                light = 180.0;
                weather = Weather::Clear;
            }
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
        for y in (if strip { 10 } else { 0 })..HEIGHT {
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

fn elements(path: &str, night: bool) -> Result<(), Box<dyn Error>> {
    use cpu_express::rendering::{SettlementElement, draw_settlement_preview};
    use eframe::egui::Color32;
    let width = 64;
    let columns = 8;
    let rows = SettlementElement::ALL.len().div_ceil(columns);
    let mut sheet = vec![Color32::from_rgb(35, 44, 55); columns * width * rows * HEIGHT];
    let mut cell = vec![Color32::BLACK; width * HEIGHT];
    for (index, element) in SettlementElement::ALL.into_iter().enumerate() {
        draw_settlement_preview(
            &mut cell,
            element,
            2026,
            if night { 480.0 } else { 180.0 },
            Weather::Clear,
        );
        for y in 0..HEIGHT {
            let target = (index / columns * HEIGHT + y) * columns * width + index % columns * width;
            sheet[target..target + width].copy_from_slice(&cell[y * width..(y + 1) * width]);
        }
        println!("{}: {}", index + 1, element.name());
    }
    let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(out, "P6\n{} {}\n255\n", columns * width, rows * HEIGHT)?;
    for pixel in sheet {
        out.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
    }
    out.flush()?;
    Ok(())
}
