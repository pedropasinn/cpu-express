//! Original fictional people. Pure integer drawing, no stored animation state.
use super::{Palette, StationStyle, Weather, sprites::Canvas};
use eframe::egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassengerKind {
    Worker,
    Student,
    Parent,
    Child,
    Tourist,
    Elder,
    Traveler,
    Porter,
    StationMaster,
    Vendor,
    Cyclist,
}

impl PassengerKind {
    pub const KIND: &'static str = "fictional";
    pub const ALL: [Self; 11] = [
        Self::Worker,
        Self::Student,
        Self::Parent,
        Self::Child,
        Self::Tourist,
        Self::Elder,
        Self::Traveler,
        Self::Porter,
        Self::StationMaster,
        Self::Vendor,
        Self::Cyclist,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Worker => "Trabalhador",
            Self::Student => "Estudante",
            Self::Parent => "Familiar",
            Self::Child => "Criança",
            Self::Tourist => "Turista",
            Self::Elder => "Idoso",
            Self::Traveler => "Viajante",
            Self::Porter => "Carregador",
            Self::StationMaster => "Chefe de estação",
            Self::Vendor => "Vendedor ambulante",
            Self::Cyclist => "Ciclista",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassengerAction {
    Wait,
    CheckWatch,
    Walk,
    Wave,
    Sit,
    Board,
    Alight,
    Carry,
}

impl PassengerAction {
    pub const ALL: [Self; 8] = [
        Self::Wait,
        Self::CheckWatch,
        Self::Walk,
        Self::Wave,
        Self::Sit,
        Self::Board,
        Self::Alight,
        Self::Carry,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wait => "Esperar",
            Self::CheckWatch => "Olhar relógio",
            Self::Walk => "Andar",
            Self::Wave => "Acenar",
            Self::Sit => "Sentar",
            Self::Board => "Embarcar",
            Self::Alight => "Desembarcar",
            Self::Carry => "Carregar bagagem",
        }
    }
}

/// Presentation only. The adapter supplies the existing motion preference and
/// an optional cold-weather indication; Snow also selects warm clothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PassengerOptions {
    pub reduce_motion: bool,
    pub cold: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassengerSprite {
    pub kind: PassengerKind,
    pub action: PassengerAction,
    pub frame: u8,
    pub outfit: u8,
    pub weather: Weather,
    pub options: PassengerOptions,
}

/// Gallery entry point using the same sprite routine as the live platform.
/// The buffer has 70 rows, like SceneRenderer; coordinates are logical pixels.
pub fn draw_passenger_preview(
    pixels: &mut [Color32],
    position: (i32, i32),
    sprite: PassengerSprite,
    palette: &Palette,
) {
    draw(&mut Canvas::new(pixels), position, sprite, palette);
}

pub(super) fn draw(
    c: &mut Canvas<'_>,
    (x, bottom): (i32, i32),
    sprite: PassengerSprite,
    p: &Palette,
) {
    use PassengerAction as A;
    use PassengerKind as K;
    let f = if sprite.options.reduce_motion {
        0
    } else {
        i32::from(sprite.frame % 4)
    };
    let moving = matches!(sprite.action, A::Walk | A::Board | A::Alight | A::Carry);
    let stride = if moving { [0, 1, 0, -1][f as usize] } else { 0 };
    let seated = sprite.action == A::Sit;
    let child = sprite.kind == K::Child;
    let rise = i32::from(matches!(sprite.action, A::Board | A::Alight) && f >= 2);
    let y = bottom - 10 + if child { 3 } else { 0 } + if seated { 2 } else { 0 } - rise;
    let coat = [p.roof, p.foliage, p.water, p.trim][usize::from(sprite.outfit % 4)];
    let skin = if sprite.outfit.is_multiple_of(2) {
        p.wall
    } else {
        p.trim
    };
    let cold = sprite.options.cold || sprite.weather == Weather::Snow;
    let rainy = matches!(sprite.weather, Weather::Rain | Weather::Thunderstorm);
    if seated {
        // Each seated pose includes its bench, so no architecture is assumed.
        c.rect(x - 4, bottom - 5, 9, 1, p.roof);
        c.rect(x - 4, bottom - 2, 9, 1, p.ink);
        c.rect(x - 4, bottom - 1, 1, 2, p.ink);
        c.rect(x + 4, bottom - 1, 1, 2, p.ink);
    }
    c.rect(x - 1, y, 4, 4, p.ink);
    c.rect(x, y + 1, 3, 2, skin);
    c.pixel(x + if seated && f == 2 { 1 } else { 2 }, y + 1, p.ink);
    c.rect(x - 1, y + 4, 4, if child { 3 } else { 4 }, p.ink);
    c.rect(x, y + 4, 2, if child { 2 } else { 3 }, coat);
    let hem = y + if child { 6 } else { 7 };
    if seated {
        c.line((x, hem), (x + 3, hem), p.ink);
        c.line((x + 3, hem), (x + 3, bottom), p.ink);
        c.pixel(x + 5, bottom - i32::from(f == 2), p.ink);
    } else {
        c.line((x, hem), (x - stride, bottom - rise), p.ink);
        c.line((x + 2, hem), (x + 2 + stride, bottom - rise), p.ink);
    }
    // Gesture silhouettes are independent of clothing and accessories.
    match sprite.action {
        A::Wait => {
            c.line((x - 2, y + 4), (x - 2, y + 6), coat);
            c.pixel(x - 2 + i32::from(f == 2), y + 7, skin);
        }
        A::CheckWatch => {
            c.line((x - 2, y + 4), (x - 2, y + 6), coat);
            c.line((x - 2, y + 6), (x + 2, y + 5 - i32::from(f >= 2)), skin);
            c.pixel(x, y + 5 - i32::from(f >= 2), p.sun);
        }
        A::Wave => {
            c.line((x + 3, y + 5), (x + 4, y + 2), coat);
            c.line((x + 4, y + 2), (x + 4 + f % 2, y), skin);
        }
        A::Carry => {
            let lift = i32::from(f >= 2);
            c.rect(x + 3, y + 5 - lift, 4, 4, p.ink);
            c.rect(x + 4, y + 6 - lift, 2, 2, p.trim);
            c.line((x + 2, y + 4), (x + 5, y + 5 - lift), skin);
        }
        A::Board | A::Alight => {
            let direction = if sprite.action == A::Board { -1 } else { 1 };
            c.line(
                (x + direction * 2, y + 4),
                (x + direction * (3 + f % 2), y + 3),
                skin,
            );
        }
        A::Walk => c.line((x - 2, y + 4), (x - 2 - stride, y + 6), coat),
        A::Sit => c.line((x + 2, y + 4), (x + 3, y + 6 - i32::from(f == 1)), skin),
    }
    match sprite.kind {
        K::Worker => {
            c.rect(x - 2, y, 6, 1, p.sun);
            c.rect(x - 1, y - 1, 4, 1, p.trim);
            c.line((x, y + 4), (x + 1, y + 6), p.trim);
        }
        K::Student => {
            c.rect(x - 3, y + 4, 2, 4, p.roof);
            c.rect(x + 3, y + 5, 2, 3, p.wall);
            c.pixel(x + 3, y + 6, p.water);
        }
        K::Parent => {
            c.circle(x - 1, y, 1, p.ink);
            c.rect(x - 3, y + 6, 2, 3, p.water);
            c.line((x + 3, y + 5), (x + 5, y + 7), skin);
        }
        K::Child => {
            c.rect(x - 2, y, 5, 1, p.roof);
            c.rect(x - 3, y + 4, 2, 2, p.water);
        }
        K::Tourist => {
            c.rect(x - 3, y + 3, 2, 5, p.foliage);
            c.rect(x - 2, y, 6, 1, p.trim);
            c.rect(x, y + 5, 3, 2, p.ink);
            c.pixel(x + 1, y + 5, p.water_light);
        }
        K::Elder => {
            c.rect(x - 1, y, 3, 1, p.cloud);
            c.line((x + 4, y + 6), (x + 4, bottom), p.roof);
            c.line((x + 2, y + 5), (x + 4, y + 6), skin);
        }
        K::Traveler => {
            c.rect(x - 4, bottom - 4, 3, 4, p.ink);
            c.rect(x - 4, bottom - 3, 2, 2, p.roof);
            c.line((x - 3, bottom - 6), (x - 3, bottom - 4), p.trim);
        }
        K::Porter => {
            c.rect(x - 2, y, 5, 1, p.water);
            c.rect(x - 4, y + 3, 3, 5, p.roof);
            c.rect(x - 4, y + 5, 3, 1, p.trim);
        }
        K::StationMaster => {
            c.rect(x - 2, y, 6, 1, p.ink);
            c.rect(x - 1, y - 1, 4, 1, p.water);
            c.pixel(x + 1, y + 4, p.sun);
            c.line((x + 5, y + 5), (x + 5, y + 2), p.ink);
            c.rect(x + 5, y + 2, 2, 2, p.foliage_light);
        }
        K::Vendor => {
            c.rect(x - 2, y, 6, 1, p.wall);
            c.rect(x - 1, y + 6, 7, 2, p.roof);
            for dx in [0, 2, 4] {
                c.pixel(x + dx, y + 5, p.sun);
            }
        }
        K::Cyclist => {
            c.rect(x - 1, y - 1, 4, 2, p.water);
            for dx in [-3, 5] {
                c.circle(x + dx, bottom - 1, 2, p.ink);
                c.pixel(x + dx, bottom - 1, p.trim);
            }
            c.line((x - 3, bottom - 1), (x + 1, bottom - 4), p.roof);
            c.line((x + 1, bottom - 4), (x + 5, bottom - 1), p.roof);
            c.line((x - 3, bottom - 1), (x + 5, bottom - 1), p.roof);
        }
    }
    if cold {
        c.rect(x - 1, y - 1, 4, 1, coat);
        c.pixel(x, y - 2, p.wall);
        c.rect(x - 1, y + 3, 4, 1, p.trim);
        c.rect(x - 1, y + 4, 1, 3, p.trim);
    }
    if rainy {
        // Fixed canopy also covers the bicycle and never adds another person.
        c.line((x + 4, y - 3), (x + 4, y + 5), p.ink);
        c.rect(x - 3, y - 4, 10, 1, p.ink);
        c.rect(x - 2, y - 5, 8, 1, coat);
        c.rect(x, y - 6, 4, 1, coat);
        c.rect(x + 1, y - 5, 2, 2, p.trim);
    }
}

#[derive(Clone, Copy)]
pub(super) struct PlatformCast {
    pub seed: u64,
    pub arrival: u64,
    pub options: PassengerOptions,
    pub weather: Weather,
}

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl PlatformCast {
    fn key(self, boarding: bool) -> u64 {
        mix(self.seed ^ self.arrival.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ u64::from(boarding))
    }

    fn kind(
        self,
        index: u32,
        count: u32,
        boarding: bool,
        style: Option<StationStyle>,
    ) -> PassengerKind {
        use PassengerKind as K;
        let key = self.key(boarding);
        // A family consumes TWO existing representatives. Never manufacture a child.
        if count >= 2 && key.is_multiple_of(4) && index < 2 {
            return if index == 0 { K::Parent } else { K::Child };
        }
        let kinds = match style {
            Some(StationStyle::PortTerminal | StationStyle::ServicePost) => [
                K::Worker,
                K::Traveler,
                K::Tourist,
                K::Worker,
                K::Elder,
                K::Cyclist,
            ],
            Some(StationStyle::MountainStone | StationStyle::TimberHalt) => [
                K::Tourist,
                K::Cyclist,
                K::Elder,
                K::Student,
                K::Traveler,
                K::Tourist,
            ],
            _ => [
                K::Worker,
                K::Student,
                K::Traveler,
                K::Tourist,
                K::Elder,
                K::Cyclist,
            ],
        };
        kinds[(mix(key.wrapping_add(u64::from(index))) % kinds.len() as u64) as usize]
    }
}

pub(super) fn platform(
    c: &mut Canvas<'_>,
    x: i32,
    cast: PlatformCast,
    service: super::StationService,
    style: Option<StationStyle>,
    p: &Palette,
    (seconds, detailed, departing): (f64, bool, bool),
) {
    use PassengerAction as A;
    let progress = service.normalized().progress;
    let frame = (super::theme::finite_nonnegative(seconds).rem_euclid(1.0) * 4.0) as u8;
    if detailed {
        let kind = match style {
            Some(StationStyle::PortTerminal | StationStyle::ServicePost) => PassengerKind::Porter,
            Some(StationStyle::TimberHalt | StationStyle::Plateau) => PassengerKind::Vendor,
            _ => PassengerKind::StationMaster,
        };
        draw(
            c,
            (x + 63, 53),
            PassengerSprite {
                kind,
                action: if departing {
                    A::Wave
                } else if kind == PassengerKind::Porter {
                    A::Carry
                } else {
                    A::Wait
                },
                frame,
                outfit: 2,
                weather: cast.weather,
                options: cast.options,
            },
            p,
        );
    }
    for (boarding, count) in [
        (true, service.boarding.min(3)),
        (false, service.alighting.min(3)),
    ] {
        for index in 0..count {
            if boarding && progress >= 1.0 || !boarding && progress <= 0.0 {
                continue;
            }
            let key = mix(cast.key(boarding).wrapping_add(u64::from(index)));
            let kind = cast.kind(index, count, boarding, style);
            let idle = [A::Wait, A::CheckWatch, A::Walk, A::Wave, A::Sit][(key % 5) as usize];
            let slot = if kind == PassengerKind::Child {
                8
            } else {
                index as i32 * 17
            };
            let t = if boarding {
                ((progress - 0.25) / 0.75).clamp(0.0, 1.0)
            } else {
                (progress / 0.5).min(1.0)
            };
            let t = t * t * (3.0 - 2.0 * t);
            let (start, end) = if boarding {
                (4 + slot, -28)
            } else {
                (-28, 4 + slot)
            };
            let offset = if cast.options.reduce_motion {
                if boarding { start } else { end }
            } else {
                (start as f32 + (end - start) as f32 * t).round() as i32
            };
            let action = if cast.options.reduce_motion {
                if boarding { A::Board } else { A::Alight }
            } else if boarding && progress <= 0.25 {
                idle
            } else if boarding && progress >= 0.8 {
                A::Board
            } else if !boarding && progress < 0.5 {
                A::Alight
            } else if !boarding {
                if departing { A::Wave } else { idle }
            } else if matches!(kind, PassengerKind::Traveler | PassengerKind::Worker) {
                A::Carry
            } else {
                A::Walk
            };
            // Walking while waiting is a bounded stroll, not journey progress.
            let stroll = if action == A::Walk
                && (progress <= 0.25 || !boarding)
                && !cast.options.reduce_motion
            {
                [0, 1, 2, 1][frame as usize]
            } else {
                0
            };
            draw(
                c,
                (x + offset + stroll, 53),
                PassengerSprite {
                    kind,
                    action,
                    frame,
                    outfit: (key % 4) as u8,
                    weather: cast.weather,
                    options: cast.options,
                },
                p,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{Biome, HEIGHT, StationService, WIDTH};

    fn sprite(kind: PassengerKind, action: PassengerAction, frame: u8) -> PassengerSprite {
        PassengerSprite {
            kind,
            action,
            frame,
            outfit: 0,
            weather: Weather::Clear,
            options: PassengerOptions::default(),
        }
    }
    fn portrait(sprite: PassengerSprite) -> Vec<Color32> {
        let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
        draw_passenger_preview(
            &mut pixels,
            (160, 53),
            sprite,
            &Palette::for_biome(Biome::GreenValley),
        );
        pixels
    }
    fn cast(seed: u64, arrival: u64) -> PlatformCast {
        PlatformCast {
            seed,
            arrival,
            weather: Weather::Clear,
            options: PassengerOptions::default(),
        }
    }
    fn platform_frame(cast: PlatformCast, service: StationService, seconds: f64) -> Vec<Color32> {
        let mut pixels = vec![Color32::TRANSPARENT; WIDTH * HEIGHT];
        platform(
            &mut Canvas::new(&mut pixels),
            150,
            cast,
            service,
            None,
            &Palette::for_biome(Biome::GreenValley),
            (seconds, false, false),
        );
        pixels
    }

    #[test]
    fn every_kind_has_a_distinct_original_silhouette() {
        let frames = PassengerKind::ALL.map(|k| portrait(sprite(k, PassengerAction::Wait, 0)));
        for (i, frame) in frames.iter().enumerate() {
            assert!(
                frames[..i].iter().all(|other| other != frame),
                "{:?}",
                PassengerKind::ALL[i]
            );
        }
    }

    #[test]
    fn every_action_animates_and_reduced_motion_freezes_all_types_and_weather() {
        for kind in PassengerKind::ALL {
            for action in PassengerAction::ALL {
                let first = portrait(sprite(kind, action, 0));
                assert!(
                    (1..4).any(|frame| portrait(sprite(kind, action, frame)) != first),
                    "{kind:?}/{action:?}"
                );
                for weather in [Weather::Clear, Weather::Rain, Weather::Snow] {
                    let mut still = sprite(kind, action, 0);
                    still.options.reduce_motion = true;
                    still.weather = weather;
                    let reference = portrait(still);
                    for frame in 1..8 {
                        assert_eq!(reference, portrait(PassengerSprite { frame, ..still }));
                    }
                }
            }
        }
    }

    #[test]
    fn real_counts_endpoints_saturation_and_static_positions_are_preserved() {
        let empty = platform_frame(cast(7, 1), StationService::default(), 0.0);
        assert!(empty.iter().all(|p| *p == Color32::TRANSPARENT));
        for progress in [0.0, 0.5, 1.0, f32::NAN] {
            assert_eq!(
                empty,
                platform_frame(
                    cast(7, 1),
                    StationService {
                        progress,
                        ..StationService::default()
                    },
                    2.0
                )
            );
        }
        let service = StationService {
            progress: 0.5,
            boarding: 3,
            alighting: 3,
        };
        let full = platform_frame(cast(7, 1), service, 0.0);
        assert_eq!(
            full,
            platform_frame(
                cast(7, 1),
                StationService {
                    boarding: u32::MAX,
                    alighting: u32::MAX,
                    ..service
                },
                0.0
            )
        );
        assert_eq!(
            empty,
            platform_frame(
                cast(7, 1),
                StationService {
                    progress: 1.0,
                    boarding: 3,
                    alighting: 0
                },
                0.0
            )
        );
        assert_eq!(
            empty,
            platform_frame(
                cast(7, 1),
                StationService {
                    progress: 0.0,
                    boarding: 0,
                    alighting: 3
                },
                0.0
            )
        );
        let mut static_cast = cast(7, 1);
        static_cast.options.reduce_motion = true;
        let reference = platform_frame(static_cast, service, 0.0);
        for progress in [0.01, 0.2, 0.5, 0.99] {
            for time in [0.0, 1.0, 22.5, f64::MAX, f64::NAN] {
                assert_eq!(
                    reference,
                    platform_frame(
                        static_cast,
                        StationService {
                            progress,
                            ..service
                        },
                        time
                    )
                );
            }
        }
    }

    #[test]
    fn seed_arrival_and_station_choose_cast_without_render_history_or_extra_children() {
        let sequence = |cast: PlatformCast| {
            (0..3)
                .map(|i| cast.kind(i, 3, true, None))
                .collect::<Vec<_>>()
        };
        let original = sequence(cast(7, 1));
        assert!((2..20).any(|arrival| sequence(cast(7, arrival)) != original));
        assert!((8..20).any(|seed| sequence(cast(seed, 1)) != original));
        for seed in 0..100 {
            for style in StationStyle::ALL {
                let c = cast(seed, 1);
                assert_ne!(c.kind(0, 1, true, Some(style)), PassengerKind::Child);
                if c.kind(0, 3, true, Some(style)) == PassengerKind::Parent {
                    assert_eq!(c.kind(1, 3, true, Some(style)), PassengerKind::Child);
                }
            }
        }
        assert_eq!(original, sequence(cast(7, 1)));
        let mut differs = false;
        for seed in 8..32 {
            let c = cast(seed, 1);
            let port: Vec<_> = (0..3)
                .map(|i| c.kind(i, 3, true, Some(StationStyle::PortTerminal)))
                .collect();
            differs |= (0..3)
                .map(|i| c.kind(i, 3, true, Some(StationStyle::MountainStone)))
                .collect::<Vec<_>>()
                != port;
        }
        assert!(differs);
    }

    #[test]
    fn platform_clips_at_all_supported_widths_and_never_draws_under_rails() {
        for width in [160, 320, 641, 1920] {
            for style in StationStyle::ALL {
                for x in [-80, 0, 246, width as i32 + 32] {
                    let mut pixels = vec![Color32::TRANSPARENT; width * HEIGHT];
                    platform(
                        &mut Canvas::new(&mut pixels),
                        x,
                        PlatformCast {
                            weather: Weather::Rain,
                            ..cast(7, 1)
                        },
                        StationService {
                            progress: 0.75,
                            boarding: 3,
                            alighting: 3,
                        },
                        Some(style),
                        &Palette::for_biome(Biome::Forest),
                        (0.5, true, true),
                    );
                    assert!(
                        pixels[55 * width..]
                            .iter()
                            .all(|p| *p == Color32::TRANSPARENT)
                    );
                    if x == -80 || x == width as i32 + 32 {
                        assert!(pixels.iter().all(|p| *p == Color32::TRANSPARENT));
                    }
                }
            }
        }
    }

    #[test]
    fn clothes_follow_palette_rain_and_cold_and_stay_inside_authored_bounds() {
        for kind in PassengerKind::ALL {
            let base = sprite(kind, PassengerAction::Wait, 0);
            assert_ne!(
                portrait(base),
                portrait(PassengerSprite {
                    weather: Weather::Rain,
                    ..base
                })
            );
            assert_ne!(
                portrait(base),
                portrait(PassengerSprite {
                    weather: Weather::Snow,
                    ..base
                })
            );
            assert_ne!(
                portrait(base),
                portrait(PassengerSprite { outfit: 1, ..base })
            );
            for action in PassengerAction::ALL {
                for frame in 0..4 {
                    let s = PassengerSprite {
                        action,
                        frame,
                        weather: Weather::Rain,
                        options: PassengerOptions {
                            cold: true,
                            reduce_motion: false,
                        },
                        ..base
                    };
                    for (index, pixel) in portrait(s).iter().enumerate() {
                        if *pixel != Color32::TRANSPARENT {
                            assert!((153..=167).contains(&(index % WIDTH)));
                            assert!((35..=54).contains(&(index / WIDTH)));
                        }
                    }
                }
            }
        }
    }
}
