//! Original sprites via the live drawing routine, following station_gallery's
//! local P6 pipeline. All allocations here belong to the offline exporter.
use cpu_express::{
    rendering::{
        Biome, HEIGHT, Palette, PassengerAction, PassengerKind, PassengerOptions, PassengerSprite,
        Weather, draw_passenger_preview,
    },
    ui_game::pixel_text,
};
use eframe::egui::{self, Color32};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

const CELL: usize = 104;

fn label_row(pixels: &mut [Color32], labels: &[(usize, usize, String)], ink: Color32) {
    let width = pixels.len() / HEIGHT;
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let painter = ui.ctx().layer_painter(egui::LayerId::background());
        for (x, y, label) in labels {
            pixel_text(&painter, egui::pos2(*x as f32, *y as f32), label, 1.0, ink);
        }
    });
    // pixel_text emits one mesh made solely of disjoint, integer pixel quads.
    let shapes = std::mem::take(&mut output.shapes);
    output.drop_without_applying_deltas();
    for shape in shapes {
        if let egui::Shape::Mesh(mesh) = shape.shape {
            for quad in mesh.vertices.as_chunks::<4>().0 {
                let left = quad.iter().map(|v| v.pos.x as usize).min().unwrap();
                let right = quad.iter().map(|v| v.pos.x as usize).max().unwrap();
                let top = quad.iter().map(|v| v.pos.y as usize).min().unwrap();
                let bottom = quad.iter().map(|v| v.pos.y as usize).max().unwrap();
                for y in top..bottom.min(HEIGHT) {
                    for x in left..right.min(width) {
                        pixels[y * width + x] = quad[0].color;
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).ok_or("informe novo caminho .ppm")?;
    let climate = std::env::args().nth(2).as_deref() == Some("--clima");
    let columns = if climate { 6 } else { 8 };
    let width = columns * CELL;
    let palette = Palette::for_biome(Biome::GreenValley);
    let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(
        out,
        "P6\n{} {}\n255\n",
        width,
        HEIGHT * PassengerKind::ALL.len()
    )?;
    for (row, kind) in PassengerKind::ALL.into_iter().enumerate() {
        let mut pixels = vec![palette.cloud; width * HEIGHT];
        let mut labels = vec![(4, 2, format!("{:02} - {}", row + 1, kind.name()))];
        for col in 0..columns {
            let (title, light, weather, cold) = if climate {
                [
                    ("Dia", 180.0, Weather::Clear, false),
                    ("Entardecer", 348.0, Weather::Clear, false),
                    ("Noite", 480.0, Weather::Clear, false),
                    ("Chuva", 180.0, Weather::Rain, false),
                    ("Neve", 180.0, Weather::Snow, false),
                    ("Frio / estático", 180.0, Weather::Clear, true),
                ][col]
            } else {
                (
                    PassengerAction::ALL[col].name(),
                    180.0,
                    Weather::Clear,
                    false,
                )
            };
            let p = Palette::at_time(Biome::GreenValley, light, true);
            for y in 30..57 {
                pixels[y * width + col * CELL..y * width + (col + 1) * CELL].fill(p.sky[2]);
            }
            pixels[55 * width + col * CELL..55 * width + (col + 1) * CELL].fill(p.trim);
            labels.push((col * CELL + 4, 17, title.into()));
            for frame in 0..4u8 {
                let x = col * CELL + 12 + usize::from(frame) * 24;
                draw_passenger_preview(
                    &mut pixels,
                    (x as i32, 53),
                    PassengerSprite {
                        kind,
                        action: if climate {
                            PassengerAction::Wave
                        } else {
                            PassengerAction::ALL[col]
                        },
                        frame,
                        outfit: if climate { frame } else { row as u8 % 4 },
                        weather,
                        options: PassengerOptions {
                            reduce_motion: climate && cold,
                            cold,
                        },
                    },
                    &p,
                );
                labels.push((x, 58, frame.to_string()));
            }
        }
        label_row(&mut pixels, &labels, palette.ink);
        for pixel in pixels {
            out.write_all(&[pixel.r(), pixel.g(), pixel.b()])?;
        }
    }
    out.flush()?;
    println!(
        "11 tipos fictícios; {}. Fonte e sprites originais do jogo.",
        if climate {
            "seis condições e quatro roupas"
        } else {
            "oito ações, quatro quadros por ação"
        }
    );
    Ok(())
}
