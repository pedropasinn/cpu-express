//! Export the actual Rust renderer, without opening a window or sampling CPU.
//! `cargo run --example export_frame -- frame.ppm 50`

use std::{error::Error, fs::File, io::Write};

use cpu_express::{
    config::Config,
    rendering::{DISPLAY_HEIGHT, DISPLAY_WIDTH, HEIGHT, SceneRenderer, WIDTH},
    simulation::{Train, World},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments.next().unwrap_or_else(|| "frame.ppm".to_owned());
    let cpu: f32 = arguments.next().as_deref().unwrap_or("50").parse()?;
    if !cpu.is_finite() || !(0.0..=100.0).contains(&cpu) {
        return Err("CPU must be between 0 and 100".into());
    }
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    // Five seconds of simulated motion, evaluated instantly; no busy waiting,
    // synthetic system load, or background metrics collector is involved.
    for _ in 0..150 {
        let distance = train.update(cpu, 1.0 / 30.0, &config);
        world.update(1.0 / 30.0, distance, train.speed_kmh());
    }
    let mut renderer = SceneRenderer::new();
    let mut output = File::create(path)?;
    write!(output, "P6\n{DISPLAY_WIDTH} {DISPLAY_HEIGHT}\n255\n")?;
    let mut rgb = Vec::with_capacity(DISPLAY_WIDTH * DISPLAY_HEIGHT * 3);
    // Match the UI's nearest-neighbor scaling. The renderer stores each cell
    // once; exported images retain the familiar 640×140 presentation size.
    for row in renderer.render(&train, &world).as_chunks::<WIDTH>().0 {
        for _ in 0..DISPLAY_HEIGHT / HEIGHT {
            for color in row {
                for _ in 0..DISPLAY_WIDTH / WIDTH {
                    rgb.extend_from_slice(&[color.r(), color.g(), color.b()]);
                }
            }
        }
    }
    output.write_all(&rgb)?;
    Ok(())
}
