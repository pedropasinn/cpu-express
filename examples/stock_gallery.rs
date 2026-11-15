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
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("informe um novo caminho .ppm")?;
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    let train = Train::new(&Config::default());
    let world = World::default();
    let rows = CoachStyle::ALL.len();
    write!(output, "P6\n{} {}\n255\n", WIDTH * 4, HEIGHT * rows)?;
    let mut renderer = SceneRenderer::new();
    renderer.set_scene_context(Some(SceneContext::new(Biome::GreenValley)));
    renderer.set_options(SceneOptions {
        clouds: false,
        ..SceneOptions::default()
    });
    for (row, coach) in CoachStyle::ALL.into_iter().enumerate() {
        renderer.set_coaches([coach; 3]);
        renderer.set_engine(EngineStyle::ALL[row % EngineStyle::ALL.len()]);
        let frames = [
            (90.0, Weather::Clear),
            (348.0, Weather::Clear),
            (450.0, Weather::Clear),
            (90.0, Weather::Rain),
        ]
        .map(|(light, weather)| {
            renderer.set_local_atmosphere(Some((weather, light < 400.0)));
            renderer.set_local_lighting(Some(light));
            renderer.render(&train, &world).to_vec()
        });
        for y in 0..HEIGHT {
            for frame in &frames {
                for pixel in &frame[y * WIDTH..(y + 1) * WIDTH] {
                    output.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
                }
            }
        }
    }
    output.flush()?;
    Ok(())
}
