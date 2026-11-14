//! Setup-only thumbnails of the original rolling stock, at native pixel scale.
//!
//! Keep one gallery in the app. Accessors only borrow uploaded textures: they do
//! not render, allocate, or upload. The renderer's opaque daytime scenery is
//! retained rather than guessing a transparency mask from shared sprite colors.

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::{
    config::Config,
    rendering::{
        Biome, CoachStyle, RenderQuality, SceneContext, SceneOptions, SceneRenderer, SceneTheme,
        WIDTH, Weather,
    },
    simulation::{Train, World, objectives::Livery},
};

const COACH_STYLES: [CoachStyle; 5] = [
    CoachStyle::Passenger,
    CoachStyle::Sleeper,
    CoachStyle::Freight,
    CoachStyle::Tank,
    CoachStyle::Observation,
];

// Native 320 x 70 coordinates, [left, top, width, height]. rolling_stock draws
// the first (tail) coach at x=58: couplers span 56..=97, canopy starts at y=32,
// and both radius-three wheels end at y=57. Keep one row above and below.
const COACH_CROP: [usize; 4] = [56, 31, 42, 28];
// sprites::locomotive spans 180..=239, including the rear socket and pilot.
// The chimney starts at y=31; all four wheels finish at y=57.
const ENGINE_CROP: [usize; 4] = [180, 30, 60, 29];

/// Nine cached textures made by the same renderer as the live train.
pub struct StockGallery {
    coaches: [TextureHandle; 5],
    liveries: [TextureHandle; 4],
}

impl StockGallery {
    /// Render and upload once during app setup; no clock, network or assets.
    #[must_use]
    pub fn new(ctx: &egui::Context) -> Self {
        let world = World::default();
        let train = Train::new(&Config::default());
        let mut renderer = SceneRenderer::new();
        renderer.set_theme(SceneTheme::Journey);
        renderer.set_quality(RenderQuality::Full);
        renderer.set_scene_context(Some(SceneContext::new(Biome::GreenValley)));
        renderer.set_options(SceneOptions {
            day_cycle: false,
            clouds: false,
            weather: Weather::Clear,
            details: true,
        });
        renderer.set_local_atmosphere(Some((Weather::Clear, true)));
        renderer.set_livery(Livery::Aurora);

        let coaches = COACH_STYLES.map(|style| {
            renderer.set_coaches([style; 3]);
            let image = crop(renderer.render(&train, &world), COACH_CROP);
            ctx.load_texture(
                format!("stock-coach-{style:?}"),
                image,
                TextureOptions::NEAREST,
            )
        });
        renderer.set_coaches([CoachStyle::Passenger; 3]);
        let liveries = Livery::ALL.map(|livery| {
            renderer.set_livery(livery);
            let image = crop(renderer.render(&train, &world), ENGINE_CROP);
            ctx.load_texture(
                format!("stock-livery-{livery:?}"),
                image,
                TextureOptions::NEAREST,
            )
        });
        Self { coaches, liveries }
    }

    /// A 42 x 28 native-pixel image of the first coach, including both wheels.
    #[must_use]
    pub fn coach(&self, style: CoachStyle) -> &TextureHandle {
        &self.coaches[match style {
            CoachStyle::Passenger => 0,
            CoachStyle::Sleeper => 1,
            CoachStyle::Freight => 2,
            CoachStyle::Tank => 3,
            CoachStyle::Observation => 4,
        }]
    }

    /// A 60 x 29 native-pixel engine image with the renderer's actual enamel.
    #[must_use]
    pub fn livery(&self, livery: Livery) -> &TextureHandle {
        &self.liveries[match livery {
            Livery::Aurora => 0,
            Livery::Ocean => 1,
            Livery::Sunset => 2,
            Livery::Midnight => 3,
        }]
    }
}

fn crop(frame: &[Color32], [left, top, width, height]: [usize; 4]) -> ColorImage {
    let mut pixels = Vec::with_capacity(width * height);
    for y in top..top + height {
        pixels.extend_from_slice(&frame[y * WIDTH + left..y * WIDTH + left + width]);
    }
    ColorImage::new([width, height], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_uploads_nine_distinct_textures_and_reuses_handles() {
        let ctx = egui::Context::default();
        let gallery = StockGallery::new(&ctx);
        let mut ids = Vec::new();
        for style in COACH_STYLES {
            let texture = gallery.coach(style);
            assert_eq!(texture.size(), [42, 28]);
            assert!(std::ptr::eq(texture, gallery.coach(style)));
            ids.push(texture.id());
        }
        for livery in Livery::ALL {
            let texture = gallery.livery(livery);
            assert_eq!(texture.size(), [60, 29]);
            assert!(std::ptr::eq(texture, gallery.livery(livery)));
            ids.push(texture.id());
        }
        for (index, id) in ids.iter().enumerate() {
            assert!(!ids[..index].contains(id));
        }
    }

    #[test]
    fn coach_crop_keeps_complete_wheels_for_every_style() {
        let mut renderer = SceneRenderer::new();
        let train = Train::new(&Config::default());
        let world = World::default();
        for style in COACH_STYLES {
            renderer.set_coaches([style; 3]);
            let image = crop(renderer.render(&train, &world), COACH_CROP);
            for x in [65, 88] {
                assert_eq!(
                    image.pixels[(57 - COACH_CROP[1]) * image.size[0] + x - COACH_CROP[0]],
                    Color32::from_rgb(39, 55, 53),
                    "{style:?} wheel at x={x}",
                );
            }
        }
    }

    #[test]
    fn engine_crop_contains_actual_livery_pixels_and_chimney() {
        let mut renderer = SceneRenderer::new();
        let train = Train::new(&Config::default());
        let world = World::default();
        for (livery, rgb) in Livery::ALL.into_iter().zip([
            [42, 105, 88],
            [47, 104, 148],
            [163, 78, 51],
            [79, 65, 122],
        ]) {
            renderer.set_livery(livery);
            let image = crop(renderer.render(&train, &world), ENGINE_CROP);
            let at = |x, y| image.pixels[(y - ENGINE_CROP[1]) * image.size[0] + x - ENGINE_CROP[0]];
            assert_eq!(at(202, 45), Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
            assert_eq!(at(217, 31), Color32::from_rgb(239, 194, 112));
            for x in [201, 213, 225, 233] {
                assert_eq!(at(x, 57), Color32::from_rgb(39, 55, 53));
            }
        }
    }
}
