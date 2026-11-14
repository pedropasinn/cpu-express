//! Export real frames from the application renderer without a window.
//!
//! Gallery:  cargo run --release --locked --example scene_gallery -- gallery assets/gallery
//! Sequence: cargo run --release --locked --example scene_gallery -- sequence frames 240 20
//!
//! All CPU percentages below are explicitly simulated inputs, never system load.
//! PPM output is RGB 640x140 using the application's nearest-neighbor 2x scale.

use std::{
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use cpu_express::{
    config::Config,
    rendering::{
        BIOME_LENGTH_PX, Biome, CoachStyle, DISPLAY_HEIGHT, DISPLAY_WIDTH, HEIGHT, SceneContext,
        SceneOptions, SceneRenderer, SceneTheme, TRANSITION_LENGTH_PX, WIDTH, Weather,
    },
    simulation::{MAX_TRAVEL_UNITS_PER_SECOND, Train, World},
};
use eframe::egui::Color32;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn state(seek_distance: f64, seek_seconds: f64, manual_cpu: f32) -> (Train, World) {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    if seek_seconds > 0.0 || seek_distance > 0.0 {
        // Respect World's active-step and speed bounds. Artificial catch-up
        // calls are intentionally rejected by the integrated travel domain.
        let seconds = seek_seconds.max(seek_distance / MAX_TRAVEL_UNITS_PER_SECOND);
        let steps = seconds.ceil() as usize;
        let dt = seconds / steps as f64;
        let distance = seek_distance / steps as f64;
        for _ in 0..steps {
            world.update(dt, distance, train.speed_kmh());
        }
    }
    // Five seconds evaluated immediately, so smoke and wheels come from the
    // actual simulation. No timer, collector, or artificial CPU workload runs.
    for _ in 0..150 {
        let distance = train.update(manual_cpu, 1.0 / 30.0, &config);
        world.update(1.0 / 30.0, distance, train.speed_kmh());
    }
    (train, world)
}

fn save_ppm(path: &Path, pixels: &[Color32]) -> Result<()> {
    let mut output = BufWriter::new(File::create(path)?);
    write!(output, "P6\n{DISPLAY_WIDTH} {DISPLAY_HEIGHT}\n255\n")?;
    let mut row_rgb = Vec::with_capacity(DISPLAY_WIDTH * 3);
    for row in pixels.as_chunks::<WIDTH>().0 {
        row_rgb.clear();
        for color in row {
            for _ in 0..DISPLAY_WIDTH / WIDTH {
                row_rgb.extend_from_slice(&[color.r(), color.g(), color.b()]);
            }
        }
        for _ in 0..DISPLAY_HEIGHT / HEIGHT {
            output.write_all(&row_rgb)?;
        }
    }
    output.flush()?;
    Ok(())
}

fn gallery(directory: &Path) -> Result<()> {
    fs::create_dir_all(directory)?;
    let cases = [
        ("01_legacy", 0.0, 0.0, Biome::GreenValley),
        ("02_green_valley_morning", 0.0, 0.0, Biome::GreenValley),
        ("03_coast_day", BIOME_LENGTH_PX + 48.0, 175.0, Biome::Coast),
        (
            "04_high_desert_sunset",
            BIOME_LENGTH_PX * 2.0 + 160.0,
            345.0,
            Biome::HighDesert,
        ),
        ("05_valley_dusk_rain", 64.0, 445.0, Biome::GreenValley),
        (
            "06_valley_to_coast",
            BIOME_LENGTH_PX - TRANSITION_LENGTH_PX * 0.5,
            175.0,
            Biome::GreenValley,
        ),
        ("07_pine_forest", 2_375.0, 175.0, Biome::Forest),
    ];
    let mut manifest = Vec::new();
    for (index, (name, seek_distance, seek_seconds, biome)) in cases.into_iter().enumerate() {
        let (train, world) = state(seek_distance, seek_seconds, 50.0);
        let mut renderer = SceneRenderer::new();
        renderer.set_scene_context(Some(SceneContext::new(biome)));
        if index == 0 {
            renderer.set_theme(SceneTheme::Legacy);
        }
        if index == 3 {
            renderer.set_coaches([
                CoachStyle::Freight,
                CoachStyle::Tank,
                CoachStyle::Observation,
            ]);
        }
        if index == 4 {
            renderer.set_options(SceneOptions {
                weather: Weather::Rain,
                ..SceneOptions::default()
            });
            renderer.set_coaches([
                CoachStyle::Sleeper,
                CoachStyle::Sleeper,
                CoachStyle::Observation,
            ]);
        }
        if index == 5 {
            renderer.set_scene_context(Some(SceneContext {
                biome: Biome::GreenValley,
                next_biome: Biome::Coast,
                blend: 0.5,
            }));
        }
        let file = format!("{name}.ppm");
        save_ppm(&directory.join(&file), renderer.render(&train, &world))?;
        let context = renderer.scene_context(&world);
        manifest.push(serde_json::json!({
            "file": file,
            "manual_cpu_percent": 50.0,
            "seek_distance_px_before_warmup": seek_distance,
            "seek_seconds_before_warmup": seek_seconds,
            "warmup_seconds": 5.0,
            "elapsed_seconds": world.elapsed_seconds(),
            "world_scroll_px": world.scroll_px(),
            "biome": format!("{:?}", context.biome),
            "next_biome": format!("{:?}", context.next_biome),
            "blend": context.blend,
            "theme": if index == 0 { "Legacy" } else { "Journey" },
            "weather": if index == 4 { "Rain" } else { "Clear" },
        }));
    }
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "source": "Actual Rust SceneRenderer used by the app; not a mockup or Windows screenshot",
            "native_dimensions": [WIDTH, HEIGHT],
            "export_dimensions": [DISPLAY_WIDTH, DISPLAY_HEIGHT],
            "scaling": "nearest-neighbor 2x",
            "frames": manifest,
        }))?,
    )?;
    println!("Exported seven renderer frames to {}", directory.display());
    Ok(())
}

fn sequence(directory: &Path, parameters: &[String]) -> Result<()> {
    let parse = |index: usize, default: f64| -> Result<f64> {
        parameters
            .get(index)
            .map_or(Ok(default), |value| Ok(value.parse()?))
    };
    let frames_value = parse(0, 240.0)?;
    let fps = parse(1, 20.0)?;
    let seek_distance = parse(2, BIOME_LENGTH_PX - TRANSITION_LENGTH_PX - 320.0)?;
    let seek_seconds = parse(3, 175.0)?;
    let manual_cpu = parse(4, 100.0)?;
    if !frames_value.is_finite()
        || !(1.0..=3_600.0).contains(&frames_value)
        || frames_value.fract() != 0.0
        || !fps.is_finite()
        || !(1.0..=60.0).contains(&fps)
        || !seek_distance.is_finite()
        || !(0.0..=1_000_000.0).contains(&seek_distance)
        || !seek_seconds.is_finite()
        || !(0.0..=86_400.0).contains(&seek_seconds)
        || !manual_cpu.is_finite()
        || !(0.0..=100.0).contains(&manual_cpu)
        || parameters.len() > 5
    {
        return Err("sequence parameters: frames 1..3600 integer; fps 1..60; distance 0..1000000; seconds 0..86400; manual CPU 0..100".into());
    }
    let frames = frames_value as usize;
    fs::create_dir_all(directory)?;
    let config = Config::default();
    let (mut train, mut world) = state(seek_distance, seek_seconds, manual_cpu as f32);
    let mut renderer = SceneRenderer::new();
    let initial_distance = world.scroll_px();
    for index in 0..frames {
        renderer.set_scene_context(Some(SceneContext::from_journey(&world.journey_snapshot())));
        save_ppm(
            &directory.join(format!("frame_{index:04}.ppm")),
            renderer.render(&train, &world),
        )?;
        let distance = train.update(manual_cpu as f32, 1.0 / fps, &config);
        world.update(1.0 / fps, distance, train.speed_kmh());
    }
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "source": "Actual Rust SceneRenderer and World; deterministic instant simulation",
            "frames": frames,
            "fps": fps,
            "duration_seconds": frames as f64 / fps,
            "manual_cpu_percent": manual_cpu,
            "seek_distance_px_before_warmup": seek_distance,
            "seek_seconds_before_warmup": seek_seconds,
            "initial_world_scroll_px": initial_distance,
            "final_world_scroll_px": world.scroll_px(),
            "route": "Circuito Aurora; actual JourneySnapshot drives the scene",
            "native_dimensions": [WIDTH, HEIGHT],
            "export_dimensions": [DISPLAY_WIDTH, DISPLAY_HEIGHT],
        }))?,
    )?;
    println!(
        "Exported {frames} frames at {fps} FPS to {}",
        directory.display()
    );
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "gallery".to_owned());
    if mode == "--help" || mode == "-h" {
        println!(
            "scene_gallery gallery [output-directory]\n\
             scene_gallery sequence [output-directory] [frames=240] [fps=20] \
             [seek-distance=1216] [seek-seconds=175] [manual-cpu=100]\n\
             Writes native-renderer RGB PPM frames enlarged exactly 2x.\n\
             Sequence time is simulated; no CPU sampler or waiting is involved."
        );
        return Ok(());
    }
    let directory = PathBuf::from(args.next().unwrap_or_else(|| "scene-output".to_owned()));
    let extra: Vec<_> = args.collect();
    match mode.as_str() {
        "gallery" if extra.is_empty() => gallery(&directory),
        "sequence" => sequence(&directory, &extra),
        _ => Err("expected gallery or sequence; see --help".into()),
    }
}
