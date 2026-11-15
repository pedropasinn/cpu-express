//! Local geometry proof, not an egui screenshot. Left: catalogue line (14 stops).
//! Right: fictional 29-stop fixture. Each cell is 480 x 480 logical pixels.
use cpu_express::simulation::{
    catalog::{Catalog, ContentKind, LineDef},
    itinerary::Itinerary,
    map_layout::{MapLayout, MapPoint},
};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

const WIDTH: usize = 960;
const HEIGHT: usize = 480;
const PAPER: [u8; 3] = [236, 223, 183];
const INK: [u8; 3] = [37, 56, 52];
const GREEN: [u8; 3] = [42, 105, 88];
const GOLD: [u8; 3] = [191, 110, 42];

struct Sheet(Vec<[u8; 3]>);
impl Sheet {
    fn pixel(&mut self, x: i32, y: i32, color: [u8; 3]) {
        if (0..WIDTH as i32).contains(&x) && (0..HEIGHT as i32).contains(&y) {
            self.0[y as usize * WIDTH + x as usize] = color;
        }
    }
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 3]) {
        for dy in 0..h {
            for dx in 0..w {
                self.pixel(x + dx, y + dy, color);
            }
        }
    }
    fn line(&mut self, a: (i32, i32), b: (i32, i32), color: [u8; 3]) {
        let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
        for i in 0..=steps {
            self.pixel(
                a.0 + (b.0 - a.0) * i / steps,
                a.1 + (b.1 - a.1) * i / steps,
                color,
            );
        }
    }
    fn number(&mut self, x: i32, y: i32, number: usize, color: [u8; 3]) {
        const DIGITS: [[u8; 5]; 10] = [
            [7, 5, 5, 5, 7],
            [2, 6, 2, 2, 7],
            [7, 1, 7, 4, 7],
            [7, 1, 7, 1, 7],
            [5, 5, 7, 1, 1],
            [7, 4, 7, 1, 7],
            [7, 4, 7, 5, 7],
            [7, 1, 1, 1, 1],
            [7, 5, 7, 5, 7],
            [7, 5, 7, 1, 7],
        ];
        for (digit, dx) in [(number / 10 % 10, 0), (number % 10, 8)] {
            for (row, bits) in DIGITS[digit].iter().enumerate() {
                for bit in 0..3 {
                    if bits & (1 << (2 - bit)) != 0 {
                        self.rect(x + dx + bit * 2, y + row as i32 * 2, 2, 2, color);
                    }
                }
            }
        }
    }
    fn cell(&mut self, offset: i32, line: &LineDef, distance: f64) -> Result<(), String> {
        let itinerary = Itinerary::from_line(line, 480.0)?;
        let map = MapLayout::new(line, &itinerary)?;
        self.rect(offset + 12, 12, 456, 36, GREEN);
        self.number(offset + 30, 25, map.stations().len(), PAPER);
        let schematic = |p: MapPoint| {
            (
                offset + 28 + (p.x * 424.0).round() as i32,
                110 + (p.y * 128.0).round() as i32,
            )
        };
        let profile = |p: MapPoint| {
            (
                offset + 28 + (p.x * 424.0).round() as i32,
                315 + (p.y * 60.0).round() as i32,
            )
        };
        for pair in map.stations().windows(2) {
            self.line(schematic(pair[0].point), schematic(pair[1].point), INK);
            if let (Some(a), Some(b)) = (pair[0].profile, pair[1].profile) {
                self.line(profile(a), profile(b), GREEN);
            }
        }
        for s in map.stations() {
            let (x, y) = schematic(s.point);
            self.rect(x - 4, y - 4, 9, 9, INK);
            self.rect(
                x - 2,
                y - 2,
                5,
                5,
                if map.discovered(s.ordinal, distance) {
                    GREEN
                } else {
                    PAPER
                },
            );
            self.number(x - 7, y + 8, s.ordinal + 1, INK);
            if let Some(p) = s.profile {
                let (px, py) = profile(p);
                self.rect(px - 1, py - 1, 3, 3, INK);
            }
        }
        let train = map.train(distance);
        let (x, y) = schematic(train.point);
        self.rect(x - 5, y - 5, 11, 10, GOLD);
        // Travel order is not screen direction on a serpentine map.
        let a = schematic(map.stations()[train.from].point);
        let b = schematic(map.stations()[train.to].point);
        let dx = (b.0 - a.0).signum();
        let dy = (b.1 - a.1).signum();
        let tip = (x + dx * 10, y + dy * 10);
        self.line((x, y), tip, INK);
        self.line(tip, (x + dx * 7 - dy * 3, y + dy * 7 + dx * 3), INK);
        self.line(tip, (x + dx * 7 + dy * 3, y + dy * 7 - dx * 3), INK);
        if let Some(p) = train.profile {
            let (px, py) = profile(p);
            self.rect(px - 3, py - 3, 7, 7, GOLD);
        }
        self.line((offset + 28, 380), (offset + 452, 380), INK);
        self.rect(offset + 12, 426, 456, 2, GREEN);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("informe um novo caminho .ppm")?;
    let catalog = Catalog::builtin()
        .as_ref()
        .map_err(|_| "catálogo inválido")?;
    let real = catalog.line("curitiba-paranagua").ok_or("linha ausente")?;
    let mut fixture = real.clone();
    fixture.id = "map-fixture-29".into();
    fixture.name = "Linha sintética de conferência".into();
    fixture.kind = ContentKind::Fictional;
    fixture.stations = (0..29)
        .map(|i| {
            let mut s = real.stations[0].clone();
            s.id = format!("fixture-{i}");
            s.name = format!("Parada {}", i + 1);
            s.km = Some(i as f64);
            s.elevation_m = Some((14 - i32::abs(i - 14)) as f64 * 40.0);
            s.verified = false;
            s.source_refs.clear();
            s.note = None;
            s
        })
        .collect();
    fixture.segments = (0..28)
        .map(|i| {
            let mut s = real.segments[0].clone();
            s.id = format!("segment-{i}");
            s.from = fixture.stations[i].id.clone();
            s.to = fixture.stations[i + 1].id.clone();
            s.distance_km = Some(1.0);
            s
        })
        .collect();
    let mut sheet = Sheet(vec![PAPER; WIDTH * HEIGHT]);
    sheet.cell(0, real, 30_000.0)?;
    sheet.cell(480, &fixture, 16_000.0)?;
    let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(out, "P6\n{WIDTH} {HEIGHT}\n255\n")?;
    for pixel in sheet.0 {
        out.write_all(&pixel)?;
    }
    out.flush()?;
    println!("Mapas: catálogo de 14 paradas na ida; fixture fictícia de 29 na volta.");
    Ok(())
}
