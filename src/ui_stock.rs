//! Setup-only thumbnails of the original rolling stock, at native pixel scale.
//!
//! Keep one gallery in the app. Accessors only borrow uploaded textures: they do
//! not render, allocate, or upload. The renderer's opaque daytime scenery is
//! retained rather than guessing a transparency mask from shared sprite colors.

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::{
    config::Config,
    rendering::{
        Biome, CoachStyle, EngineStyle, RenderQuality, SceneContext, SceneOptions, SceneRenderer,
        SceneTheme, WIDTH, Weather,
    },
    simulation::{Train, World, objectives::Livery},
};

const COACH_STYLES: [CoachStyle; CoachStyle::ALL.len()] = CoachStyle::ALL;

// Native 320 x 70 coordinates, [left, top, width, height]. rolling_stock draws
// the first (tail) coach at x=58: couplers span 56..=97, canopy starts at y=32,
// and both radius-three wheels end at y=57. Keep one row above and below.
const COACH_CROP: [usize; 4] = [56, 31, 42, 28];
// sprites::locomotive spans 180..=239, including the rear socket and pilot.
// The chimney starts at y=31; all four wheels finish at y=57.
const ENGINE_CROP: [usize; 4] = [180, 30, 60, 29];

/// Cached textures made by the same renderer as the live train.
pub struct StockGallery {
    coaches: [TextureHandle; CoachStyle::ALL.len()],
    liveries: [TextureHandle; 4],
    engines: [TextureHandle; EngineStyle::ALL.len()],
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
        renderer.set_engine(EngineStyle::Steam);

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
        // Engine silhouettes always use Aurora, regardless of the final livery
        // rendered above. Uploads happen only at gallery creation.
        renderer.set_livery(Livery::Aurora);
        let engines = EngineStyle::ALL.map(|style| {
            renderer.set_engine(style);
            let image = crop(renderer.render(&train, &world), ENGINE_CROP);
            ctx.load_texture(
                format!("stock-engine-{style:?}"),
                image,
                TextureOptions::NEAREST,
            )
        });
        Self {
            coaches,
            liveries,
            engines,
        }
    }

    /// A 42 x 28 native-pixel image of the first coach, including both wheels.
    #[must_use]
    pub fn coach(&self, style: CoachStyle) -> &TextureHandle {
        &self.coaches[CoachStyle::ALL.iter().position(|s| *s == style).unwrap()]
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

    /// A cached 60 x 29 engine portrait in Aurora paint. Borrowing the handle
    /// never renders or uploads; unlock rules belong to the caller.
    #[must_use]
    pub fn engine(&self, style: EngineStyle) -> &TextureHandle {
        &self.engines[EngineStyle::ALL.iter().position(|s| *s == style).unwrap()]
    }
}

fn crop(frame: &[Color32], [left, top, width, height]: [usize; 4]) -> ColorImage {
    let mut pixels = Vec::with_capacity(width * height);
    for y in top..top + height {
        pixels.extend_from_slice(&frame[y * WIDTH + left..y * WIDTH + left + width]);
    }
    ColorImage::new([width, height], pixels)
}

/// Presentation only: locked items remain inspectable, but never emit Equip.
pub struct StockCard<'a> {
    pub name: &'a str,
    pub state: &'a str,
    pub texture: &'a TextureHandle,
    pub selected: bool,
    pub locked_reason: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockAction {
    Equip,
    Inspect,
}

pub struct StockCardResponse {
    pub response: egui::Response,
    pub action: Option<StockAction>,
}

/// Responsive catalogue card with keyboard activation and a readable lock reason.
/// The caller owns equipment changes and the inspector opened by `Inspect`.
pub fn stock_card(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    card: &StockCard<'_>,
) -> StockCardResponse {
    use crate::ui_game as game;
    let width = ui.available_width().max(32.0);
    let text_width = (width - 16.0).max(1.0);
    let ink = if ui.is_enabled() {
        game::INK
    } else {
        game::MUTED
    };
    let name = ui.painter().layout(
        card.name.to_owned(),
        egui::FontId::proportional(13.0),
        ink,
        text_width,
    );
    let state_text = card.locked_reason.unwrap_or(card.state);
    let state = ui.painter().layout(
        state_text.to_owned(),
        egui::FontId::proportional(12.0),
        game::MUTED,
        text_width,
    );
    let height = 88.0 + name.size().y + state.size().y;
    let (_, rect) = ui.allocate_space(egui::vec2(width, height));
    let response = ui.interact(rect, ui.make_persistent_id(id), egui::Sense::click());
    let accessible = format!("{} · {}", card.name, state_text);
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::Button,
            ui.is_enabled(),
            card.selected,
            &accessible,
        )
    });
    let hover = ui.ctx().animate_bool_with_time(
        response.id.with("hover"),
        response.hovered() || response.has_focus(),
        if game::reduce_motion(ui.ctx()) {
            0.0
        } else {
            0.12
        },
    );
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3, game::CARD);
        painter.rect_stroke(
            rect,
            3,
            egui::Stroke::new(
                if card.selected { 3.0 } else { 1.0 + hover },
                if card.selected {
                    game::GREEN
                } else {
                    game::EDGE
                },
            ),
            egui::StrokeKind::Inside,
        );
        let ppp = ui.ctx().pixels_per_point();
        let scale = (((width - 20.0).max(1.0) * ppp / card.texture.size_vec2().x)
            .min(60.0 * ppp / card.texture.size_vec2().y)
            .floor()
            .max(1.0))
            / ppp;
        let size = card.texture.size_vec2() * scale;
        let origin = egui::pos2(rect.center().x - size.x * 0.5, rect.top() + 5.0);
        let origin = egui::pos2(
            (origin.x * ppp).round() / ppp,
            (origin.y * ppp).round() / ppp,
        );
        painter.image(
            card.texture.id(),
            egui::Rect::from_min_size(origin, size),
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            if card.locked_reason.is_some() || !ui.is_enabled() {
                Color32::from_gray(155)
            } else {
                Color32::WHITE
            },
        );
        game::icon(
            &painter,
            egui::Rect::from_min_size(rect.min + egui::vec2(6.0, 6.0), egui::vec2(16.0, 16.0)),
            if card.locked_reason.is_some() {
                game::Icon::Lock
            } else if card.selected {
                game::Icon::Check
            } else {
                game::Icon::Train
            },
            ink,
        );
        let name_y = rect.top() + 69.0;
        painter.galley(
            egui::pos2(rect.center().x - name.size().x * 0.5, name_y),
            name.clone(),
            ink,
        );
        painter.galley(
            egui::pos2(
                rect.center().x - state.size().x * 0.5,
                name_y + name.size().y + 5.0,
            ),
            state,
            game::MUTED,
        );
        if response.has_focus() {
            painter.rect_stroke(
                rect.shrink(4.0),
                1,
                egui::Stroke::new(2.0, game::BLUE),
                egui::StrokeKind::Inside,
            );
        }
    }
    let action = response
        .clicked()
        .then_some(if card.locked_reason.is_some() {
            StockAction::Inspect
        } else {
            StockAction::Equip
        });
    StockCardResponse {
        response: response.on_hover_text(accessible),
        action,
    }
}

/// One coach position. Labels and names are supplied by the caller unchanged.
pub struct CompositionSlot<'a> {
    pub label: &'a str,
    pub name: &'a str,
    pub icon: crate::ui_game::Icon,
}

/// Compact tail-to-engine schematic. Returns the clicked slot index, without
/// changing equipment. `selected` outside the slice simply selects nothing.
pub fn composition(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    slots: &[CompositionSlot<'_>],
    selected: usize,
) -> egui::InnerResponse<Option<usize>> {
    use crate::ui_game as game;
    ui.push_id(id, |ui| {
        let mut clicked = None;
        let spacing = 6.0;
        let width = ((ui.available_width() - spacing * slots.len() as f32)
            / (slots.len() + 1) as f32)
            .max(44.0);
        ui.spacing_mut().item_spacing.x = spacing;
        ui.horizontal_wrapped(|ui| {
            for (index, slot) in slots.iter().enumerate() {
                let response = game::button(
                    ui,
                    index,
                    slot.icon,
                    slot.label,
                    selected == index,
                    egui::vec2(width, if width >= 100.0 { 44.0 } else { 52.0 }),
                );
                let accessible = format!("{} · {}", slot.label, slot.name);
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::Button,
                        ui.is_enabled(),
                        selected == index,
                        &accessible,
                    )
                });
                if response.clicked() {
                    clicked = Some(index);
                }
                response.on_hover_text(slot.name);
            }
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(width, if width >= 100.0 { 44.0 } else { 52.0 }),
                egui::Sense::hover(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), "Locomotiva")
            });
            let painter = ui.painter_at(rect);
            game::icon(
                &painter,
                egui::Rect::from_center_size(
                    rect.center() - egui::vec2(0.0, 7.0),
                    egui::Vec2::splat(24.0),
                ),
                game::Icon::Train,
                game::INK,
            );
            painter.text(
                egui::pos2(rect.center().x, rect.bottom() - 10.0),
                egui::Align2::CENTER_CENTER,
                "Locomotiva",
                egui::FontId::proportional(10.0),
                game::MUTED,
            );
        });
        if let Some(slot) = slots.get(selected) {
            ui.label(
                egui::RichText::new(format!("{} · {}", slot.label, slot.name))
                    .size(12.0)
                    .color(game::MUTED),
            );
        }
        clicked
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_uploads_all_distinct_textures_and_reuses_handles() {
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
        for engine in EngineStyle::ALL {
            let texture = gallery.engine(engine);
            assert_eq!(texture.size(), [60, 29]);
            assert!(std::ptr::eq(texture, gallery.engine(engine)));
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
