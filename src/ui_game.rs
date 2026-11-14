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
            mix(GREEN, BLUE, hover * 0.3)
        } else {
            mix(CARD, GOLD, hover * 0.16)
        };
        let foreground = if !enabled {
            MUTED
        } else if selected {
            CARD
        } else {
            INK
        };
        painter.rect_filled(rect, 3, if enabled { INK } else { MUTED });
        painter.rect_filled(face, 3, fill);
        painter.rect_stroke(face, 3, Stroke::new(2.0, EDGE), egui::StrokeKind::Inside);
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
        let font = egui::TextStyle::Button.resolve(ui.style());
        let galley = painter.layout_no_wrap(label.to_owned(), font.clone(), foreground);
        let stacked = face.width() < 100.0 && rect.height() >= 46.0;
        let icon_size = if stacked {
            18.0
        } else {
            24.0_f32.min(face.height() - 8.0)
        };
        let gap = if label.is_empty() { 0.0 } else { 7.0 };
        let width = icon_size + gap + galley.size().x;
        let start = (face.center().x - width * 0.5).max(face.left() + 7.0);
        let icon_center = if stacked {
            egui::pos2(face.center().x, face.top() + 13.0)
        } else {
            egui::pos2(start + icon_size * 0.5, face.center().y)
        };
        icon(
            &painter,
            Rect::from_center_size(icon_center, Vec2::splat(icon_size)),
            icon_kind,
            foreground,
        );
        painter.text(
            if stacked {
                egui::pos2(face.center().x, face.bottom() - 10.0)
            } else {
                egui::pos2(start + icon_size + gap, face.center().y)
            },
            if stacked {
                egui::Align2::CENTER_CENTER
            } else {
                egui::Align2::LEFT_CENTER
            },
            label,
            font,
            foreground,
        );
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
        painter.rect_filled(rect, 3, PAPER);
        painter.rect_stroke(rect, 3, Stroke::new(2.0, EDGE), egui::StrokeKind::Inside);
        painter.rect_stroke(
            rect.shrink(2.0),
            1,
            Stroke::new(1.0, CARD),
            egui::StrokeKind::Inside,
        );
        let inside = rect.shrink(4.0);
        if value > 0.0 {
            let fill = Rect::from_min_size(
                inside.min,
                egui::vec2(inside.width() * value, inside.height()),
            );
            // A light tint keeps dark text readable on both sides of the fill.
            let tint = mix(CARD, if ui.is_enabled() { color } else { MUTED }, 0.42);
            painter.rect_filled(fill, 0, tint);
            painter.line_segment(
                [fill.left_top(), fill.right_top()],
                Stroke::new(1.0, mix(tint, CARD, 0.6)),
            );
        }
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            if ui.is_enabled() { INK } else { MUTED },
        );
    }
    response
}

/// Changes this UI and its descendants only. Use `ui.scope` at the call site
/// when siblings must retain their previous style. Built-in popup widgets
/// inherit these visuals; no fonts, textures or global context are replaced.
pub fn apply_style(ui: &mut egui::Ui) {
    let style = ui.style_mut();
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
