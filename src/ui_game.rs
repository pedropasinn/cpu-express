//! Original, asset-free railway game widgets. Call `apply_style` inside `ui.scope`
//! to theme a panel without changing the application's global context style.

use eframe::egui::{self, Color32, Rect, Response, Stroke, Vec2};
use std::hash::Hash;

pub const INK: Color32 = Color32::from_rgb(57, 39, 29);
pub const PAPER: Color32 = Color32::from_rgb(235, 218, 181);
pub const CARD: Color32 = Color32::from_rgb(255, 244, 217);
pub const MUTED: Color32 = Color32::from_rgb(113, 87, 63);
pub const EDGE: Color32 = Color32::from_rgb(91, 62, 40);
pub const GOLD: Color32 = Color32::from_rgb(191, 141, 55);
pub const GREEN: Color32 = Color32::from_rgb(38, 107, 91);
pub const BLUE: Color32 = Color32::from_rgb(47, 100, 126);

/// Session-only menu preference. Does not alter scene art or simulation time.
pub fn set_reduce_motion(ctx: &egui::Context, enabled: bool) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique("game-reduce-menu-motion"), enabled));
}

/// Read by custom menu widgets; absent preference defaults to full feedback.
pub fn reduce_motion(ctx: &egui::Context) -> bool {
    ctx.data(|data| {
        data.get_temp(egui::Id::unique("game-reduce-menu-motion"))
            .unwrap_or(false)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Train,
    Map,
    Paint,
    Settings,
    Help,
    Star,
    Lock,
    Check,
    Clock,
    Station,
    Cpu,
    Memory,
    Pause,
    Play,
    Save,
    Close,
    Weather,
    Book,
    Passenger,
    Cargo,
    Ticket,
    Mail,
}

/// Hand-drawn 12 × 12 silhouettes, independent of fonts and external textures.
/// Each cell lands on physical pixels, including at fractional display scales.
pub fn icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let rows: [&str; 12] = match icon {
        Icon::Passenger => [
            "....####....",
            "...######...",
            "...######...",
            "....####....",
            ".....##.....",
            "..########..",
            "..########..",
            "..#.##.#.#..",
            "....####....",
            "....#..#....",
            "...##..##...",
            "...##..##...",
        ],
        Icon::Cargo => [
            "............",
            ".##########.",
            ".##......##.",
            ".###....###.",
            ".#.##..##.#.",
            ".#..####..#.",
            ".#..####..#.",
            ".#.##..##.#.",
            ".###....###.",
            ".##......##.",
            ".##########.",
            "............",
        ],
        Icon::Ticket => [
            "............",
            ".##########.",
            ".#.....#..#.",
            "##........##",
            "#......#...#",
            "..###.......",
            "..###..#....",
            "#..........#",
            "##.....#..##",
            ".#........#.",
            ".##########.",
            "............",
        ],
        Icon::Mail => [
            "............",
            "############",
            "##........##",
            "###......###",
            "#.##....##.#",
            "#..##..##..#",
            "#...####...#",
            "#....##....#",
            "#..........#",
            "#..........#",
            "############",
            "............",
        ],
        Icon::Train => [
            "............",
            ".......##...",
            ".#####.##...",
            ".#..##......",
            ".#..######..",
            ".#####...##.",
            ".##########.",
            ".##########.",
            "..##...##...",
            ".####.####..",
            "..##...##...",
            "############",
        ],
        Icon::Map => [
            "....#....#..",
            ".####...###.",
            ".#..####..#.",
            ".#..#..#..#.",
            ".#..#..#..#.",
            ".#..#..#..#.",
            ".#..#..#..#.",
            ".#..#..#..#.",
            ".#..#..#..#.",
            ".###..####..",
            "..#....#....",
            "............",
        ],
        Icon::Paint => [
            "........##..",
            ".......###..",
            "......###...",
            ".....###....",
            "....###.....",
            "...###......",
            "..####......",
            ".##..#......",
            ".##..#......",
            ".#####......",
            "####........",
            "............",
        ],
        Icon::Settings => [
            ".....##.....",
            "..#..##..#..",
            ".##########.",
            "..########..",
            "..##....##..",
            "####....####",
            "####....####",
            "..##....##..",
            "..########..",
            ".##########.",
            "..#..##..#..",
            ".....##.....",
        ],
        Icon::Help => [
            "....####....",
            "..##....##..",
            ".#..####..#.",
            ".#.....#..#.",
            "#.....#....#",
            "#....#.....#",
            "#....#.....#",
            "#..........#",
            ".#...#....#.",
            ".#........#.",
            "..##....##..",
            "....####....",
        ],
        Icon::Star => [
            ".....##.....",
            ".....##.....",
            "....####....",
            "....####....",
            "############",
            ".##########.",
            "..########..",
            "...######...",
            "..########..",
            "..###..###..",
            ".##......##.",
            "............",
        ],
        Icon::Lock => [
            "....####....",
            "...##..##...",
            "..##....##..",
            "..##....##..",
            "..##....##..",
            ".##########.",
            ".##########.",
            ".####..####.",
            ".####..####.",
            ".##########.",
            ".##########.",
            "............",
        ],
        Icon::Check => [
            "............",
            "..........##",
            ".........###",
            "........###.",
            ".......###..",
            "##....###...",
            "###..###....",
            ".######.....",
            "..####......",
            "...##.......",
            "............",
            "............",
        ],
        Icon::Clock => [
            "....####....",
            "..##....##..",
            ".#....#...#.",
            ".#....#...#.",
            "#.....#....#",
            "#.....#....#",
            "#.....###..#",
            "#..........#",
            ".#........#.",
            ".#........#.",
            "..##....##..",
            "....####....",
        ],
        Icon::Station => [
            ".....##.....",
            "....####....",
            "...######...",
            "..########..",
            ".##########.",
            "############",
            "..#......#..",
            "..#.##.#.#..",
            "..#.##...#..",
            "..#....#.#..",
            "..######.#..",
            "############",
        ],
        Icon::Cpu => [
            "..#.#.#.#...",
            "..#.#.#.#...",
            "############",
            "..#......#..",
            "###.####.###",
            "..#.#..#.#..",
            "###.#..#.###",
            "..#.####.#..",
            "###......###",
            "..########..",
            "..#.#.#.#...",
            "..#.#.#.#...",
        ],
        Icon::Memory => [
            "............",
            "............",
            ".##########.",
            ".#........#.",
            ".#.##.##..#.",
            ".#.##.##..#.",
            ".#.##.##..#.",
            ".#........#.",
            ".##########.",
            "..#.#.#.#...",
            "..#.#.#.#...",
            "............",
        ],
        Icon::Pause => [
            "............",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "..###..###..",
            "............",
        ],
        Icon::Play => [
            "............",
            "...#........",
            "...###......",
            "...#####....",
            "...######...",
            "...########.",
            "...########.",
            "...######...",
            "...#####....",
            "...###......",
            "...#........",
            "............",
        ],
        Icon::Save => [
            ".#########..",
            ".##.....###.",
            ".##...#.###.",
            ".##...#.###.",
            ".##########.",
            ".##########.",
            ".##......##.",
            ".##.####.##.",
            ".##......##.",
            ".##.####.##.",
            ".##########.",
            "............",
        ],
        Icon::Close => [
            "............",
            ".##......##.",
            ".###....###.",
            "..###..###..",
            "...######...",
            "....####....",
            "....####....",
            "...######...",
            "..###..###..",
            ".###....###.",
            ".##......##.",
            "............",
        ],
        Icon::Weather => [
            "...#........",
            ".#...#......",
            "..###.......",
            "#######.....",
            "..###.###...",
            ".#..##...#..",
            "...##....##.",
            "..#........#",
            "..##########",
            "............",
            "....#..#..#.",
            "...#..#..#..",
        ],
        Icon::Book => [
            "............",
            ".####..####.",
            "#....##....#",
            "#.##.##.##.#",
            "#....##....#",
            "#.##.##.##.#",
            "#....##....#",
            "#.##.##.##.#",
            "#....##....#",
            "#....##....#",
            ".####..####.",
            "............",
        ],
    };
    if !rect.is_finite() || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    // Enabled icons use the coloured artwork; a muted colour keeps the flat silhouette,
    // which is how a disabled control is told apart.
    // A selected button already has a coloured face: its icon stays a light silhouette.
    if color != MUTED
        && color != CARD
        && let Some(art) = sprite_for(icon)
        && sprite(painter, rect, art)
    {
        return;
    }
    let ppp = painter.ctx().pixels_per_point();
    let cell = (rect.width().min(rect.height()) * ppp / 12.0).floor() / ppp;
    // Below twelve physical pixels there is no legible pixel grid.
    if cell <= 0.0 {
        return;
    }
    let origin = rect.center() - Vec2::splat(cell * 6.0);
    let origin = egui::pos2(
        (origin.x * ppp).round() / ppp,
        (origin.y * ppp).round() / ppp,
    );
    // A mesh avoids the anti-aliased rectangle tessellator erasing one-pixel cells.
    let mut mesh = egui::Mesh::default();
    for (y, row) in rows.iter().enumerate() {
        for (x, pixel) in row.bytes().enumerate() {
            if pixel == b'#' {
                mesh.add_colored_rect(
                    Rect::from_min_size(
                        origin + egui::vec2(x as f32, y as f32) * cell,
                        Vec2::splat(cell),
                    ),
                    color,
                );
            }
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn mix(a: Color32, b: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let channel =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * amount).round() as u8;
    Color32::from_rgba_unmultiplied(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
        channel(a.a(), b.a()),
    )
}

/// Raised walnut/brass control; selected buttons have a teal face.
/// `id` must be stable and unique within the parent UI. The label is drawn as
/// its own, unmodified text shape so accessibility and UI harnesses can find it.
pub fn button(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    icon_kind: Icon,
    label: &str,
    selected: bool,
    size: Vec2,
) -> Response {
    let id = ui.make_persistent_id(id);
    let (_, rect) = ui.allocate_space(size.max(egui::vec2(28.0, 28.0)));
    // Sense::click participates in egui's standard Tab/Space/Enter handling.
    let response = ui.interact(rect, id, egui::Sense::click());
    let enabled = ui.is_enabled();
    let reduced_motion = reduce_motion(ui.ctx());
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, enabled, selected, label));
    let hover = ui.ctx().animate_bool_with_time(
        id.with("hover"),
        enabled && (response.hovered() || response.has_focus()),
        if reduced_motion { 0.0 } else { 0.12 },
    );
    let pressed = ui.ctx().animate_bool_with_time(
        id.with("press"),
        enabled && response.is_pointer_button_down_on(),
        if reduced_motion { 0.0 } else { 0.07 },
    );
    if ui.is_rect_visible(rect) {
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let face = Rect::from_min_max(
            rect.min + egui::vec2(0.0, pressed * 2.0),
            rect.max - egui::vec2(0.0, 3.0 - pressed * 2.0),
        );
        let fill = if !enabled {
            PAPER
        } else if selected {
            mix(RIBBON, CREAM, hover * 0.14)
        } else {
            mix(BUTTER, CREAM, hover * 0.45)
        };
        let foreground = if !enabled {
            MUTED
        } else if selected {
            CARD
        } else {
            INK
        };
        // A lip in a darker shade of the face reads as a physical key.
        painter.rect_filled(rect.expand(1.0), 4, if enabled { WOOD_DARK } else { MUTED });
        painter.rect_filled(rect, 3, mix(fill, WOOD_DARK, 0.5));
        painter.rect_filled(face, 3, fill);
        painter.rect_stroke(
            face,
            3,
            Stroke::new(1.0, mix(fill, WOOD_DARK, 0.45)),
            egui::StrokeKind::Inside,
        );
        let inset = face.shrink(3.0);
        let glint = mix(fill, CARD, 0.55);
        painter.line_segment(
            [inset.left_bottom(), inset.left_top()],
            Stroke::new(1.0, glint),
        );
        painter.line_segment(
            [inset.left_top(), inset.right_top()],
            Stroke::new(1.0, glint),
        );
        if enabled && response.has_focus() {
            painter.rect_stroke(
                face.shrink(4.0),
                1,
                Stroke::new(2.0, GOLD),
                egui::StrokeKind::Inside,
            );
        }
        // Lit upper half, as on a moulded key.
        painter.rect_filled(
            Rect::from_min_max(
                face.min + egui::vec2(2.0, 2.0),
                egui::pos2(face.right() - 2.0, face.center().y),
            ),
            2,
            mix(fill, CREAM, 0.22),
        );
        let stacked = face.width() < 100.0 && rect.height() >= 46.0;
        let icon_size = if stacked {
            18.0
        } else if face.height() >= 40.0 {
            32.0
        } else {
            24.0_f32.min(face.height() - 8.0)
        };
        let gap = if label.is_empty() { 0.0 } else { 8.0 };
        // Pixel capitals are the voice of the game; long labels fall back to the text font.
        let cell = 2.0;
        let lettering = pixel_text_size(label, cell);
        let room = face.width() - if stacked { 8.0 } else { icon_size + gap + 14.0 };
        let fits = !label.is_empty() && face.height() >= 26.0;
        let with_icon = fits && lettering.x <= room;
        let alone = fits && !stacked && !with_icon && lettering.x <= face.width() - 14.0;
        let pixel = with_icon || alone;
        let (icon_size, gap) = if alone { (0.0, 0.0) } else { (icon_size, gap) };
        let font = egui::FontId::proportional(15.0);
        let galley = painter.layout_no_wrap(label.to_owned(), font.clone(), foreground);
        let label_width = if pixel { lettering.x } else { galley.size().x };
        let width = icon_size + gap + label_width;
        let start = (face.center().x - width * 0.5).max(face.left() + 7.0);
        let icon_center = if stacked {
            egui::pos2(face.center().x, face.top() + 13.0)
        } else {
            egui::pos2(start + icon_size * 0.5, face.center().y)
        };
        if !alone {
            icon(
                &painter,
                Rect::from_center_size(icon_center, Vec2::splat(icon_size)),
                icon_kind,
                foreground,
            );
        }
        let (anchor, align) = if stacked {
            (
                egui::pos2(face.center().x, face.bottom() - 11.0),
                egui::Align2::CENTER_CENTER,
            )
        } else {
            (
                egui::pos2(start + icon_size + gap, face.center().y),
                egui::Align2::LEFT_CENTER,
            )
        };
        // The text shape stays for UI harnesses even when the pixel lettering replaces it.
        painter.text(
            anchor,
            align,
            label,
            font,
            if pixel {
                Color32::TRANSPARENT
            } else {
                foreground
            },
        );
        if pixel {
            let origin = egui::pos2(
                if stacked {
                    anchor.x - lettering.x * 0.5
                } else {
                    anchor.x
                },
                anchor.y - cell * 5.5,
            );
            pixel_text(
                &painter,
                origin + shadow_offset(cell),
                label,
                cell,
                mix(fill, WOOD_DARK, 0.35),
            );
            pixel_text(&painter, origin, label, cell, foreground);
        }
    }
    response
}

/// Determinate, full-width 22-point bar. Values are fractions in `0..=1`;
/// non-finite inputs display zero. This widget never schedules animation.
pub fn progress(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    value: f32,
    label: &str,
    color: Color32,
) -> Response {
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let id = ui.make_persistent_id(id);
    let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width().max(0.0), 22.0));
    let response = ui.interact(rect, id, egui::Sense::hover());
    response.widget_info(|| {
        let mut info =
            egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, ui.is_enabled(), label);
        info.value = Some(f64::from(value) * 100.0);
        info
    });
    if ui.is_rect_visible(rect) && rect.width() >= 8.0 {
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.rect_filled(rect, 3, WOOD_DARK);
        painter.rect_filled(rect.shrink(2.0), 2, mix(WOOD_DARK, WOOD, 0.45));
        let inside = rect.shrink(3.0);
        if value > 0.0 {
            let fill = Rect::from_min_size(
                inside.min,
                egui::vec2(inside.width() * value, inside.height()),
            );
            // Saturated fill with a lit upper edge; the caption is light on both sides.
            let tint = mix(if ui.is_enabled() { color } else { MUTED }, CREAM, 0.12);
            painter.rect_filled(fill, 1, tint);
            painter.rect_filled(
                Rect::from_min_size(fill.min, egui::vec2(fill.width(), 3.0)),
                1,
                mix(tint, CREAM, 0.45),
            );
        }
        // One unmodified text shape: harnesses and assistive tools find exactly this caption.
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            if ui.is_enabled() { CREAM } else { PAPER },
        );
    }
    response
}

/// Changes this UI and its descendants only. Use `ui.scope` at the call site
/// when siblings must retain their previous style. Built-in popup widgets
/// inherit these visuals; no fonts, textures or global context are replaced.
pub fn apply_style(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    for (text_style, size) in [
        (egui::TextStyle::Body, 14.5),
        (egui::TextStyle::Button, 15.0),
        (egui::TextStyle::Small, 12.0),
        (egui::TextStyle::Heading, 20.0),
    ] {
        style
            .text_styles
            .insert(text_style, egui::FontId::proportional(size));
    }
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.interact_size.y = 28.0;
    style.spacing.window_margin = egui::Margin::same(12);
    style.animation_time = 0.12;
    let visuals = &mut style.visuals;
    visuals.dark_mode = false;
    visuals.override_text_color = Some(INK);
    visuals.weak_text_color = Some(MUTED);
    visuals.panel_fill = PAPER;
    visuals.window_fill = CARD;
    visuals.window_stroke = Stroke::new(2.0, EDGE);
    visuals.window_corner_radius = egui::CornerRadius::same(4);
    visuals.menu_corner_radius = egui::CornerRadius::same(3);
    visuals.faint_bg_color = PAPER;
    visuals.extreme_bg_color = CARD;
    visuals.text_edit_bg_color = Some(CARD);
    visuals.code_bg_color = PAPER;
    visuals.hyperlink_color = BLUE;
    visuals.selection.bg_fill = mix(CARD, GREEN, 0.25);
    visuals.selection.stroke = Stroke::new(1.5, GREEN);
    visuals.slider_trailing_fill = true;
    for (widget, fill, border) in [
        (&mut visuals.widgets.noninteractive, PAPER, EDGE),
        (&mut visuals.widgets.inactive, CARD, EDGE),
        (&mut visuals.widgets.hovered, mix(CARD, GOLD, 0.18), GOLD),
        (&mut visuals.widgets.active, mix(CARD, GREEN, 0.22), GREEN),
        (&mut visuals.widgets.open, PAPER, GOLD),
    ] {
        widget.bg_fill = fill;
        widget.weak_bg_fill = fill;
        widget.bg_stroke = Stroke::new(1.5, border);
        widget.fg_stroke = Stroke::new(1.5, INK);
        widget.corner_radius = egui::CornerRadius::same(3);
        widget.expansion = 0.0;
    }
}

/// Parchment card with a walnut outline and restrained, squared corners.
pub fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PAPER)
        .stroke(Stroke::new(2.0, EDGE))
        .inner_margin(12)
        .corner_radius(4)
}

// ---------------------------------------------------------------------------
// Game shell: wooden frame, ribbon title, bottom dock and an authored pixel font.
// Everything is drawn from rectangles and polygons; no texture or font file.
// ---------------------------------------------------------------------------

pub const WOOD_DARK: Color32 = Color32::from_rgb(52, 32, 22);
pub const WOOD: Color32 = Color32::from_rgb(121, 78, 45);
pub const WOOD_LIGHT: Color32 = Color32::from_rgb(178, 124, 72);
pub const RIBBON: Color32 = Color32::from_rgb(28, 112, 100);
pub const RIBBON_DARK: Color32 = Color32::from_rgb(16, 70, 64);
pub const CREAM: Color32 = Color32::from_rgb(255, 246, 222);
pub const BUTTER: Color32 = Color32::from_rgb(246, 208, 128);
pub const RED: Color32 = Color32::from_rgb(178, 58, 48);

/// Thickness of the wooden frame around a game window, in points.
pub const FRAME: f32 = 12.0;

/// Five by seven pixel capitals. Lowercase maps to capitals and accents are
/// drawn above the base letter, so Portuguese titles need no extra glyphs.
fn glyph(letter: char) -> [&'static str; 7] {
    match letter {
        'A' => [
            ".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        'B' => [
            "####.", "#...#", "#...#", "####.", "#...#", "#...#", "####.",
        ],
        'C' => [
            ".####", "#....", "#....", "#....", "#....", "#....", ".####",
        ],
        'D' => [
            "####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####.",
        ],
        'E' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#####",
        ],
        'F' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#....",
        ],
        'G' => [
            ".####", "#....", "#....", "#.###", "#...#", "#...#", ".###.",
        ],
        'H' => [
            "#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        'I' => [
            "#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####",
        ],
        'J' => [
            "..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##..",
        ],
        'K' => [
            "#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#",
        ],
        'L' => [
            "#....", "#....", "#....", "#....", "#....", "#....", "#####",
        ],
        'M' => [
            "#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#",
        ],
        'N' => [
            "#...#", "##..#", "#.#.#", "#.#.#", "#..##", "#...#", "#...#",
        ],
        'O' => [
            ".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        'P' => [
            "####.", "#...#", "#...#", "####.", "#....", "#....", "#....",
        ],
        'Q' => [
            ".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#",
        ],
        'R' => [
            "####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#",
        ],
        'S' => [
            ".####", "#....", "#....", ".###.", "....#", "....#", "####.",
        ],
        'T' => [
            "#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#..",
        ],
        'U' => [
            "#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        'V' => [
            "#...#", "#...#", "#...#", "#...#", ".#.#.", ".#.#.", "..#..",
        ],
        'W' => [
            "#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#",
        ],
        'X' => [
            "#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#",
        ],
        'Y' => [
            "#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#..",
        ],
        'Z' => [
            "#####", "....#", "...#.", "..#..", ".#...", "#....", "#####",
        ],
        '0' => [
            ".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###.",
        ],
        '1' => [
            "..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###.",
        ],
        '2' => [
            ".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####",
        ],
        '3' => [
            "####.", "....#", "....#", ".###.", "....#", "....#", "####.",
        ],
        '4' => [
            "...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#.",
        ],
        '5' => [
            "#####", "#....", "####.", "....#", "....#", "#...#", ".###.",
        ],
        '6' => [
            ".###.", "#....", "#....", "####.", "#...#", "#...#", ".###.",
        ],
        '7' => [
            "#####", "....#", "...#.", "..#..", "..#..", "..#..", "..#..",
        ],
        '8' => [
            ".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###.",
        ],
        '9' => [
            ".###.", "#...#", "#...#", ".####", "....#", "....#", ".###.",
        ],
        '.' => [
            ".....", ".....", ".....", ".....", ".....", ".##..", ".##..",
        ],
        ',' => [
            ".....", ".....", ".....", ".....", ".##..", ".##..", ".#...",
        ],
        ':' => [
            ".....", ".##..", ".##..", ".....", ".##..", ".##..", ".....",
        ],
        '-' | '–' => [
            ".....", ".....", ".....", ".###.", ".....", ".....", ".....",
        ],
        '/' => [
            "....#", "....#", "...#.", "..#..", ".#...", "#....", "#....",
        ],
        '!' => [
            "..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#..",
        ],
        '?' => [
            ".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#..",
        ],
        '%' => [
            "##..#", "##..#", "...#.", "..#..", ".#...", "#..##", "#..##",
        ],
        '+' => [
            ".....", "..#..", "..#..", "#####", "..#..", "..#..", ".....",
        ],
        '·' => [
            ".....", ".....", ".....", "..#..", ".....", ".....", ".....",
        ],
        _ => [
            ".....", ".....", ".....", ".....", ".....", ".....", ".....",
        ],
    }
}

/// Base capital and the mark drawn above it (`a` acute, `g` grave, `c` circumflex,
/// `t` tilde) or below it (`d` cedilla).
fn decompose(letter: char) -> (char, Option<char>) {
    match letter {
        'á' | 'Á' => ('A', Some('a')),
        'à' | 'À' => ('A', Some('g')),
        'â' | 'Â' => ('A', Some('c')),
        'ã' | 'Ã' => ('A', Some('t')),
        'é' | 'É' => ('E', Some('a')),
        'ê' | 'Ê' => ('E', Some('c')),
        'í' | 'Í' => ('I', Some('a')),
        'ó' | 'Ó' => ('O', Some('a')),
        'ô' | 'Ô' => ('O', Some('c')),
        'õ' | 'Õ' => ('O', Some('t')),
        'ú' | 'Ú' => ('U', Some('a')),
        'ç' | 'Ç' => ('C', Some('d')),
        other => (other.to_ascii_uppercase(), None),
    }
}

/// A drop shadow of a full cell would fill the counters of one-cell strokes and make
/// the letters look hollow; a third of a cell, straight down, keeps them solid.
fn shadow_offset(cell: f32) -> Vec2 {
    egui::vec2(0.0, (cell / 3.0).max(1.0))
}

fn snapped_cell(painter: &egui::Painter, cell: f32) -> f32 {
    let ppp = painter.ctx().pixels_per_point();
    (cell * ppp).round().max(1.0) / ppp
}

/// Size of `text` in the pixel font. Two rows above the capitals are kept for accents.
#[must_use]
pub fn pixel_text_size(text: &str, cell: f32) -> Vec2 {
    let letters = text.chars().count() as f32;
    egui::vec2((letters * 6.0 - 1.0).max(0.0) * cell, 10.0 * cell)
}

/// Paints `text` with its top-left corner at `pos`. Purely decorative for assistive
/// technology: callers pair it with a labelled response (see [`pixel_label`]).
pub fn pixel_text(painter: &egui::Painter, pos: egui::Pos2, text: &str, cell: f32, color: Color32) {
    let cell = snapped_cell(painter, cell);
    let ppp = painter.ctx().pixels_per_point();
    let origin = egui::pos2((pos.x * ppp).round() / ppp, (pos.y * ppp).round() / ppp);
    let mut mesh = egui::Mesh::default();
    let mut dot = |column: f32, row: f32| {
        mesh.add_colored_rect(
            Rect::from_min_size(origin + egui::vec2(column, row) * cell, Vec2::splat(cell)),
            color,
        );
    };
    for (index, letter) in text.chars().enumerate() {
        let (base, mark) = decompose(letter);
        let left = index as f32 * 6.0;
        for (row, line) in glyph(base).iter().enumerate() {
            for (column, pixel) in line.bytes().enumerate() {
                if pixel == b'#' {
                    dot(left + column as f32, row as f32 + 2.0);
                }
            }
        }
        match mark {
            Some('a') => {
                dot(left + 3.0, 0.0);
                dot(left + 2.0, 1.0);
            }
            Some('g') => {
                dot(left + 1.0, 0.0);
                dot(left + 2.0, 1.0);
            }
            Some('c') => {
                dot(left + 2.0, 0.0);
                dot(left + 1.0, 1.0);
                dot(left + 3.0, 1.0);
            }
            Some('t') => {
                dot(left + 1.0, 1.0);
                dot(left + 2.0, 0.0);
                dot(left + 3.0, 1.0);
                dot(left + 4.0, 0.0);
            }
            Some('d') => {
                dot(left + 2.0, 9.0);
                dot(left + 3.0, 9.0);
            }
            _ => {}
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Size of `text` in the display style: strokes two cells wide, seven cells per letter.
#[must_use]
pub fn display_text_size(text: &str, cell: f32) -> Vec2 {
    let letters = text.chars().count() as f32;
    egui::vec2((letters * 7.0 - 1.0).max(0.0) * cell, 10.0 * cell)
}

/// Title lettering: heavy strokes, dark outline and a drop shadow, so it reads over
/// any background. Decorative for assistive technology, like [`pixel_text`].
pub fn display_text(
    painter: &egui::Painter,
    pos: egui::Pos2,
    text: &str,
    cell: f32,
    fill: Color32,
    outline: Color32,
) {
    let cell = snapped_cell(painter, cell);
    let ppp = painter.ctx().pixels_per_point();
    let origin = egui::pos2((pos.x * ppp).round() / ppp, (pos.y * ppp).round() / ppp);
    let edge = (cell * 0.5 * ppp).round().max(1.0) / ppp;
    let mut dots = Vec::new();
    for (index, letter) in text.chars().enumerate() {
        let (base, mark) = decompose(letter);
        let left = index as f32 * 7.0;
        for (row, line) in glyph(base).iter().enumerate() {
            for (column, pixel) in line.bytes().enumerate() {
                if pixel == b'#' {
                    dots.push((left + column as f32, row as f32 + 2.0));
                }
            }
        }
        let marks: &[(f32, f32)] = match mark {
            Some('a') => &[(3.0, 0.0), (2.0, 1.0)],
            Some('g') => &[(1.0, 0.0), (2.0, 1.0)],
            Some('c') => &[(2.0, 0.0), (1.0, 1.0), (3.0, 1.0)],
            Some('t') => &[(1.0, 1.0), (2.0, 0.0), (3.0, 1.0), (4.0, 0.0)],
            Some('d') => &[(2.0, 9.0), (3.0, 9.0)],
            _ => &[],
        };
        dots.extend(marks.iter().map(|(column, row)| (left + column, *row)));
    }
    let mut mesh = egui::Mesh::default();
    // Outline and shadow first, then the letters, in one mesh.
    for (column, row) in &dots {
        let dot = Rect::from_min_size(
            origin + egui::vec2(*column, *row) * cell,
            egui::vec2(cell * 2.0, cell),
        );
        mesh.add_colored_rect(
            Rect::from_min_max(
                dot.min - Vec2::splat(edge),
                dot.max + egui::vec2(edge, edge + cell * 0.5),
            ),
            outline,
        );
    }
    for (column, row) in &dots {
        mesh.add_colored_rect(
            Rect::from_min_size(
                origin + egui::vec2(*column, *row) * cell,
                egui::vec2(cell * 2.0, cell),
            ),
            fill,
        );
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Small round marker that keeps an explanation one hover away instead of on the page.
pub fn hint(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> Response {
    let text: egui::WidgetText = text.into();
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
    let plain = text.text().to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &plain));
    if ui.is_rect_visible(rect) {
        let lit = response.hovered();
        let painter = ui.painter();
        painter.circle_filled(rect.center(), 8.0, WOOD_DARK);
        painter.circle_filled(rect.center(), 6.5, if lit { GOLD } else { BUTTER });
        pixel_text(
            painter,
            rect.center() - egui::vec2(2.5, 5.5),
            "i",
            1.0,
            WOOD_DARK,
        );
    }
    response.on_hover_text(text)
}

/// Pixel-font heading with a drop shadow, exposed to assistive technology as a label.
pub fn pixel_label(ui: &mut egui::Ui, text: &str, cell: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        pixel_text_size(text, cell) + Vec2::splat(cell),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, text));
    if ui.is_rect_visible(rect) {
        let shadow = mix(color, WOOD_DARK, 0.75);
        pixel_text(
            ui.painter(),
            rect.min + shadow_offset(cell),
            text,
            cell,
            shadow,
        );
        pixel_text(ui.painter(), rect.min, text, cell, color);
    }
    response
}

/// Thick bevelled wooden frame with a parchment field, for a whole game window.
pub fn wood_frame(painter: &egui::Painter, rect: Rect) {
    if !rect.is_finite() || rect.width() < FRAME * 3.0 || rect.height() < FRAME * 3.0 {
        painter.rect_filled(rect, 0, PAPER);
        return;
    }
    painter.rect_filled(rect, 4, WOOD_DARK);
    painter.rect_filled(rect.shrink(2.0), 3, WOOD);
    // Light from the top-left: bright upper edges, dark lower edges.
    let bevel = rect.shrink(3.0);
    let shade = mix(WOOD, WOOD_DARK, 0.6);
    painter.line_segment(
        [bevel.left_top(), bevel.right_top()],
        Stroke::new(2.0, WOOD_LIGHT),
    );
    painter.line_segment(
        [bevel.left_top(), bevel.left_bottom()],
        Stroke::new(2.0, WOOD_LIGHT),
    );
    painter.line_segment(
        [bevel.left_bottom(), bevel.right_bottom()],
        Stroke::new(2.0, shade),
    );
    painter.line_segment(
        [bevel.right_top(), bevel.right_bottom()],
        Stroke::new(2.0, shade),
    );
    let field = rect.shrink(FRAME);
    painter.rect_filled(field.expand(2.0), 1, WOOD_DARK);
    painter.rect_filled(field, 0, PAPER);
    painter.line_segment(
        [field.left_top(), field.right_top()],
        Stroke::new(2.0, mix(PAPER, WOOD, 0.35)),
    );
    painter.line_segment(
        [field.left_bottom(), field.right_bottom()],
        Stroke::new(1.0, CREAM),
    );
    // Brass bolts hold the corners.
    for corner in [
        rect.left_top() + Vec2::splat(FRAME * 0.5),
        rect.right_top() + egui::vec2(-FRAME * 0.5, FRAME * 0.5),
        rect.left_bottom() + egui::vec2(FRAME * 0.5, -FRAME * 0.5),
        rect.right_bottom() - Vec2::splat(FRAME * 0.5),
    ] {
        painter.rect_filled(
            Rect::from_center_size(corner, Vec2::splat(6.0)),
            1,
            WOOD_DARK,
        );
        painter.rect_filled(Rect::from_center_size(corner, Vec2::splat(4.0)), 1, GOLD);
        painter.rect_filled(
            Rect::from_center_size(corner - Vec2::splat(1.0), Vec2::splat(2.0)),
            0,
            BUTTER,
        );
    }
}

/// Title banner across the top of a window: folded tails, medallion and pixel title.
pub fn ribbon(ui: &mut egui::Ui, title: &str, emblem: Icon, compact: bool) -> Response {
    let height = if compact { 34.0 } else { 46.0 };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height + 8.0),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, title));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let tail = height * 0.55;
    let body = Rect::from_min_max(
        egui::pos2(rect.left() + tail, rect.top()),
        egui::pos2(rect.right() - tail, rect.top() + height),
    );
    // Tails sit behind and below the body, each with a swallow-tail notch.
    for side in [-1.0_f32, 1.0] {
        let (outer, inner, fold) = if side < 0.0 {
            (rect.left(), body.left() + 8.0, body.left())
        } else {
            (rect.right(), body.right() - 8.0, body.right())
        };
        let top = body.top() + 7.0;
        let bottom = body.bottom() + 7.0;
        let middle = (top + bottom) * 0.5;
        let notch = outer - side * tail * 0.45;
        for quad in [
            [
                egui::pos2(outer, top),
                egui::pos2(inner, top),
                egui::pos2(inner, middle),
                egui::pos2(notch, middle),
            ],
            [
                egui::pos2(notch, middle),
                egui::pos2(inner, middle),
                egui::pos2(inner, bottom),
                egui::pos2(outer, bottom),
            ],
        ] {
            painter.add(egui::Shape::convex_polygon(
                quad.to_vec(),
                RIBBON_DARK,
                Stroke::NONE,
            ));
        }
        // The fold that joins tail and body.
        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(fold, body.bottom()),
                egui::pos2(fold - side * 8.0, body.bottom()),
                egui::pos2(fold - side * 8.0, body.bottom() + 7.0),
            ],
            WOOD_DARK,
            Stroke::NONE,
        ));
    }
    painter.rect_filled(body.expand(1.0), 2, WOOD_DARK);
    painter.rect_filled(body, 2, RIBBON);
    painter.rect_filled(
        Rect::from_min_max(egui::pos2(body.left(), body.bottom() - 5.0), body.max),
        0,
        mix(RIBBON, RIBBON_DARK, 0.55),
    );
    for y in [body.top() + 4.0, body.bottom() - 8.0] {
        painter.line_segment(
            [
                egui::pos2(body.left() + 4.0, y),
                egui::pos2(body.right() - 4.0, y),
            ],
            Stroke::new(1.0, GOLD),
        );
    }
    let mut cell = snapped_cell(&painter, if compact { 2.0 } else { 3.0 });
    let medallion = height - 14.0;
    // Shrink the letters until the title fits beside the medallion.
    while cell > 1.0 && display_text_size(title, cell).x + medallion + 26.0 > body.width() {
        cell = snapped_cell(&painter, cell - 0.5);
    }
    let text = display_text_size(title, cell);
    let start = body.center().x - (text.x + medallion + 10.0) * 0.5;
    let badge = Rect::from_min_size(
        egui::pos2(start, body.center().y - medallion * 0.5 - 2.0),
        Vec2::splat(medallion),
    );
    painter.rect_filled(badge.expand(2.0), 4, WOOD_DARK);
    painter.rect_filled(badge, 3, GOLD);
    painter.rect_stroke(
        badge.shrink(1.0),
        2,
        Stroke::new(1.0, BUTTER),
        egui::StrokeKind::Inside,
    );
    icon(
        &painter,
        if medallion >= 32.0 {
            badge
        } else {
            badge.shrink(4.0)
        },
        emblem,
        WOOD_DARK,
    );
    // Capitals occupy rows two to eight of the ten-row line.
    let origin = egui::pos2(badge.right() + 10.0, body.center().y - 2.0 - cell * 5.5);
    display_text(&painter, origin, title, cell, CREAM, WOOD_DARK);
    response
}

/// Square tile of the bottom dock: large icon over a short caption. The selected
/// tile is lifted, lit and marked, so the state never depends on colour alone.
pub fn dock_button(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    icon_kind: Icon,
    label: &str,
    selected: bool,
    size: Vec2,
) -> Response {
    let id = ui.make_persistent_id(id);
    let (_, rect) = ui.allocate_space(size.max(egui::vec2(44.0, 44.0)));
    let response = ui.interact(rect, id, egui::Sense::click());
    let enabled = ui.is_enabled();
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, enabled, selected, label));
    let reduced_motion = reduce_motion(ui.ctx());
    let hover = ui.ctx().animate_bool_with_time(
        id.with("hover"),
        enabled && (response.hovered() || response.has_focus()),
        if reduced_motion { 0.0 } else { 0.1 },
    );
    let lift = ui.ctx().animate_bool_with_time(
        id.with("lift"),
        selected,
        if reduced_motion { 0.0 } else { 0.12 },
    );
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let pressed = if response.is_pointer_button_down_on() {
        2.0
    } else {
        0.0
    };
    let tile = Rect::from_min_max(
        rect.min + egui::vec2(1.0, 7.0 - lift * 4.0 + pressed),
        rect.max - egui::vec2(1.0, 1.0),
    );
    let face_color = if selected {
        mix(RIBBON, CREAM, 0.08 + hover * 0.08)
    } else {
        mix(WOOD, WOOD_LIGHT, 0.25 + hover * 0.45)
    };
    painter.rect_filled(tile.expand(1.0), 4, WOOD_DARK);
    painter.rect_filled(tile, 3, mix(face_color, WOOD_DARK, 0.55));
    let face = Rect::from_min_max(tile.min, tile.max - egui::vec2(0.0, 4.0));
    painter.rect_filled(face, 3, face_color);
    painter.line_segment(
        [
            face.left_top() + egui::vec2(3.0, 2.0),
            face.right_top() + egui::vec2(-3.0, 2.0),
        ],
        Stroke::new(1.0, mix(face_color, CREAM, 0.5)),
    );
    if selected {
        painter.rect_stroke(face, 3, Stroke::new(2.0, GOLD), egui::StrokeKind::Inside);
        // Marker above the tile.
        let tip = egui::pos2(face.center().x, face.top() - 1.0);
        painter.add(egui::Shape::convex_polygon(
            vec![
                tip + egui::vec2(-5.0, -5.0),
                tip + egui::vec2(5.0, -5.0),
                tip,
            ],
            GOLD,
            Stroke::NONE,
        ));
    } else if enabled && response.has_focus() {
        painter.rect_stroke(face, 3, Stroke::new(2.0, BUTTER), egui::StrokeKind::Inside);
    }
    let foreground = if enabled {
        CREAM
    } else {
        mix(CREAM, WOOD, 0.6)
    };
    let caption = face.height() >= 34.0 && !label.is_empty();
    let pixel_caption =
        caption && pixel_text_size(label, 2.0).x <= face.width() - 6.0 && face.height() >= 48.0;
    let reserved = if pixel_caption {
        26.0
    } else if caption {
        20.0
    } else {
        10.0
    };
    let icon_size = (face.height() - reserved).clamp(12.0, 36.0);
    let icon_center = egui::pos2(
        face.center().x,
        if caption {
            face.top() + 5.0 + icon_size * 0.5
        } else {
            face.center().y
        },
    );
    icon(
        &painter,
        Rect::from_center_size(icon_center, Vec2::splat(icon_size)),
        icon_kind,
        foreground,
    );
    if caption {
        let lettering = pixel_text_size(label, 2.0);
        let pixel = pixel_caption;
        let anchor = egui::pos2(
            face.center().x,
            face.bottom() - if pixel { 12.0 } else { 8.0 },
        );
        painter.text(
            anchor,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.5),
            if pixel {
                Color32::TRANSPARENT
            } else {
                foreground
            },
        );
        if pixel {
            let origin = egui::pos2(anchor.x - lettering.x * 0.5, anchor.y - 11.0);
            pixel_text(&painter, origin + shadow_offset(2.0), label, 2.0, WOOD_DARK);
            pixel_text(&painter, origin, label, 2.0, foreground);
        }
    }
    response
}

/// Card with a wooden title plate, used for each block of a screen.
pub fn section_frame(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    let outer = egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(2.0, WOOD_DARK))
        .corner_radius(4)
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 0,
            bottom: 10,
        });
    outer.show(ui, |ui| {
        ui.set_width(ui.available_width());
        let (plate, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), egui::Sense::hover());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, title));
        if ui.is_rect_visible(plate) {
            // The plate bleeds to the card's side borders.
            let band = Rect::from_min_max(
                egui::pos2(plate.left() - 10.0, plate.top()),
                egui::pos2(plate.right() + 10.0, plate.bottom()),
            );
            let painter = ui.painter().with_clip_rect(band.intersect(ui.clip_rect()));
            painter.rect_filled(band, 0, WOOD);
            painter.line_segment(
                [
                    band.left_top() + egui::vec2(0.0, 1.0),
                    band.right_top() + egui::vec2(0.0, 1.0),
                ],
                Stroke::new(2.0, WOOD_LIGHT),
            );
            painter.line_segment(
                [band.left_bottom(), band.right_bottom()],
                Stroke::new(2.0, WOOD_DARK),
            );
            let mut size = snapped_cell(&painter, 2.0);
            while size > 1.0 && display_text_size(title, size).x > plate.width() - 8.0 {
                size = snapped_cell(&painter, size - 0.5);
            }
            let origin = egui::pos2(plate.left() + 2.0, plate.center().y - size * 5.5);
            display_text(&painter, origin, title, size, CREAM, WOOD_DARK);
        }
        ui.add_space(6.0);
        content(ui);
    });
}

pub use crate::ui_sprites::Sprite;

/// Paints a coloured 16 x 16 sprite centred in `rect`, on whole physical pixels.
/// Returns false, drawing nothing, when `rect` is too small for the grid.
pub fn sprite(painter: &egui::Painter, rect: Rect, sprite: Sprite) -> bool {
    if !rect.is_finite() {
        return false;
    }
    let ppp = painter.ctx().pixels_per_point();
    let cell = (rect.width().min(rect.height()) * ppp / 16.0).floor() / ppp;
    if cell <= 0.0 {
        return false;
    }
    let origin = rect.center() - Vec2::splat(cell * 8.0);
    let origin = egui::pos2(
        (origin.x * ppp).round() / ppp,
        (origin.y * ppp).round() / ppp,
    );
    let mut mesh = egui::Mesh::default();
    for &(x, y, width, height, r, g, b) in sprite.cells() {
        mesh.add_colored_rect(
            Rect::from_min_size(
                origin + egui::vec2(f32::from(x), f32::from(y)) * cell,
                egui::vec2(f32::from(width), f32::from(height)) * cell,
            ),
            Color32::from_rgb(r, g, b),
        );
    }
    painter.add(egui::Shape::mesh(mesh));
    true
}

/// Coloured artwork for an interface icon, when one exists.
const fn sprite_for(icon: Icon) -> Option<Sprite> {
    Some(match icon {
        Icon::Train => Sprite::Locomotiva,
        Icon::Map => Sprite::Mapa,
        Icon::Paint => Sprite::Oficina,
        Icon::Settings => Sprite::Engrenagem,
        Icon::Help => Sprite::Ajuda,
        Icon::Star => Sprite::Estrela,
        Icon::Lock => Sprite::Cadeado,
        Icon::Check => Sprite::Confirmar,
        Icon::Clock => Sprite::Relogio,
        Icon::Station => Sprite::Estacao,
        Icon::Cpu => Sprite::Cpu,
        Icon::Memory => Sprite::Memoria,
        Icon::Pause => Sprite::Pausar,
        Icon::Play => Sprite::Retomar,
        Icon::Save => Sprite::Salvar,
        Icon::Close => Sprite::Fechar,
        Icon::Weather => Sprite::Nuvem,
        Icon::Passenger => Sprite::Passageiro,
        Icon::Cargo => Sprite::Caixote,
        Icon::Ticket => Sprite::Bilhete,
        Icon::Mail => Sprite::Encomenda,
        Icon::Book => return None,
    })
}
