//! Reproducible renderer and CPU-side texture preparation timings.
//! `cargo run --release --locked --example scene_bench > scene-benchmark.json`
//!
//! This finite microbenchmark does not sample CPU, open a window, contact a
//! graphics driver, or measure GPU upload/Windows process utilization.

use std::{hint::black_box, time::Instant};

use cpu_express::{
    config::Config,
    rendering::{
        Biome, HEIGHT, RenderQuality, SceneContext, SceneOptions, SceneRenderer, SceneTheme, WIDTH,
        Weather,
    },
    simulation::{Train, World},
};
use eframe::egui::{ColorImage, Context, TextureOptions, Vec2};

const WARMUP: usize = 1_000;
const ITERATIONS: u32 = 10_000;
const ROUNDS: usize = 7;

fn summary(samples: &[f64]) -> serde_json::Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    serde_json::json!({
        "rounds_ns_per_frame": samples,
        "min_us": sorted[0] / 1_000.0,
        "median_us": sorted[sorted.len() / 2] / 1_000.0,
        "max_us": sorted[sorted.len() - 1] / 1_000.0,
    })
}

fn measure(
    name: &str,
    renderer: &mut SceneRenderer,
    train: &Train,
    world: &World,
) -> serde_json::Value {
    for _ in 0..WARMUP {
        black_box(renderer.render(train, world));
    }
    let mut render_ns = Vec::with_capacity(ROUNDS);
    let mut copy_ns = Vec::with_capacity(ROUNDS);
    let mut set_ns = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(renderer.render(black_box(train), black_box(world)));
        }
        render_ns.push(started.elapsed().as_nanos() as f64 / f64::from(ITERATIONS));
        let pixels = renderer.render(train, world);
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(ColorImage {
                size: [WIDTH, HEIGHT],
                source_size: Vec2::new(WIDTH as f32, HEIGHT as f32),
                pixels: black_box(pixels).to_vec(),
            });
        }
        copy_ns.push(started.elapsed().as_nanos() as f64 / f64::from(ITERATIONS));
        let ctx = Context::default();
        let mut texture = ctx.load_texture(
            "scene-benchmark",
            ColorImage::new([WIDTH, HEIGHT], pixels.to_vec()),
            TextureOptions::NEAREST,
        );
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            texture.set(
                ColorImage::new([WIDTH, HEIGHT], pixels.to_vec()),
                TextureOptions::NEAREST,
            );
            // Bound the queue. take_delta hands prepared bytes to the stage
            // that would normally upload them; no GPU backend runs here.
            let _ = black_box(ctx.tex_manager().write().take_delta());
        }
        set_ns.push(started.elapsed().as_nanos() as f64 / f64::from(ITERATIONS));
    }
    serde_json::json!({
        "case": name,
        "render": summary(&render_ns),
        "color_image_copy": summary(&copy_ns),
        "texture_set_and_cpu_delta_including_copy": summary(&set_ns),
    })
}

fn main() {
    let config = Config::default();
    let mut train = Train::new(&config);
    let mut world = World::default();
    for _ in 0..150 {
        let distance = train.update(50.0, 1.0 / 30.0, &config);
        world.update(1.0 / 30.0, distance, train.speed_kmh());
    }
    let mut renderer = SceneRenderer::new();
    let mut results = Vec::new();
    renderer.set_theme(SceneTheme::Legacy);
    results.push(measure("legacy", &mut renderer, &train, &world));
    renderer.set_theme(SceneTheme::Journey);
    for (name, biome) in [
        ("green_valley_full", Biome::GreenValley),
        ("forest_full", Biome::Forest),
        ("coast_full", Biome::Coast),
        ("high_desert_full", Biome::HighDesert),
    ] {
        renderer.set_scene_context(Some(SceneContext::new(biome)));
        results.push(measure(name, &mut renderer, &train, &world));
    }
    renderer.set_scene_context(Some(SceneContext {
        biome: Biome::GreenValley,
        next_biome: Biome::Coast,
        blend: 0.5,
    }));
    results.push(measure("transition_full", &mut renderer, &train, &world));
    renderer.set_options(SceneOptions {
        weather: Weather::Rain,
        ..SceneOptions::default()
    });
    results.push(measure(
        "rain_transition_full",
        &mut renderer,
        &train,
        &world,
    ));
    renderer.set_options(SceneOptions::default());
    renderer.set_quality(RenderQuality::Low);
    results.push(measure("transition_low", &mut renderer, &train, &world));
    renderer.set_scene_context(Some(SceneContext::new(Biome::GreenValley)));
    results.push(measure("green_valley_low", &mut renderer, &train, &world));
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "debug_assertions": cfg!(debug_assertions),
            "iterations_per_round": ITERATIONS,
            "rounds": ROUNDS,
            "warmup_frames_per_case": WARMUP,
            "state": "50 percent explicitly manual; 150 x 1/30 second updates; fixed state per measured case",
            "clock": "monotonic wall time; host scheduling may affect microbenchmarks",
            "native_framebuffer_bytes": WIDTH * HEIGHT * 4,
            "submitted_bytes_per_frame": WIDTH * HEIGHT * 4,
            "gpu_upload_measured": false,
            "windows_process_measured": false,
            "cases": results,
        }))
        .expect("finite measurement JSON")
    );
}
