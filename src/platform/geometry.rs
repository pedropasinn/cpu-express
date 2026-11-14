//! Geometry in physical pixels, independent of Win32 and the UI event loop.
//!
//! All rectangles use half-open bounds: `[left, right) × [top, bottom)`.
//! A desktop origin must never be divided by a monitor's DPI scale. Only a
//! logical length such as the dock margin is converted before entering here.

/// A monitor rectangle in physical virtual-desktop coordinates.
/// Monitors left of or above the primary monitor have negative coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// A physical position and outer size wholly contained by a valid work area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowPlacement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// The edge beside which to place the window, without overlapping that edge's
/// taskbar/appbar reservation. `Auto` preserves the baseline's stable policy.
///
/// For an auto-hidden taskbar, Windows can report `work == monitor`. Its edge
/// cannot be inferred from those rectangles. An explicit edge supplied by the
/// Windows adapter or the user keeps that case deterministic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DockEdge {
    #[default]
    Auto,
    Bottom,
    Top,
    Left,
    Right,
}

/// Convert the eight-point dock margin to physical pixels at the window's DPI.
/// A zero DPI, which Windows returns for an unavailable window, uses 96 DPI.
#[must_use]
pub fn dock_margin_for_dpi(dpi: u32) -> i32 {
    let dpi = if dpi == 0 { 96 } else { dpi };
    // u32::MAX * 8 / 96 is below i32::MAX. Widen before multiplying;
    // the addition rounds to the nearest physical pixel.
    ((u64::from(dpi) * 8 + 48) / 96) as i32
}

/// Intersect a reported work area with its monitor, keeping physical origins.
///
/// An empty/inverted rectangle or a disjoint work area returns `None`. This
/// deliberately does not fall back to the whole monitor: doing so could put a
/// window over a taskbar after a transient or invalid system reading.
///
/// This operation is explicit. `dock_placement` remains strict about the
/// baseline contract and rejects work areas that extend beyond their monitor.
#[must_use]
pub fn intersect_work_area(monitor: PhysicalRect, work: PhysicalRect) -> Option<PhysicalRect> {
    if !valid_rectangle(monitor) || !valid_rectangle(work) {
        return None;
    }
    let intersection = PhysicalRect {
        left: monitor.left.max(work.left),
        top: monitor.top.max(work.top),
        right: monitor.right.min(work.right),
        bottom: monitor.bottom.min(work.bottom),
    };
    valid_rectangle(intersection).then_some(intersection)
}

/// Dock beside the largest reserved edge, with bottom-right as fallback.
///
/// `window_size` and `margin` are already physical pixels. Several appbars are
/// resolved by the largest reservation, with bottom, top, left, right as the
/// stable tie order. An oversized window is reduced to the work area and
/// margins give way when space is tight. Invalid/out-of-monitor work areas or
/// nonpositive window sizes return `None`.
#[must_use]
pub fn dock_placement(
    monitor: PhysicalRect,
    work: PhysicalRect,
    window_size: [i32; 2],
    margin: i32,
) -> Option<WindowPlacement> {
    dock_placement_at_edge(monitor, work, window_size, margin, DockEdge::Auto)
}

/// Dock at an explicit physical edge, or use the baseline policy with `Auto`.
///
/// The right/bottom end is used along the other axis, preserving the original
/// layout. An explicit edge affects alignment only; it never overrides the
/// work-area bounds or reserves system space. A nonnegative inset is retained
/// when possible, including when an auto-hidden taskbar reserves no work area.
#[must_use]
pub fn dock_placement_at_edge(
    monitor: PhysicalRect,
    work: PhysicalRect,
    window_size: [i32; 2],
    margin: i32,
    edge: DockEdge,
) -> Option<WindowPlacement> {
    let limits = placement_limits(monitor, work, window_size, margin)?;
    let edge = if edge == DockEdge::Auto {
        reserved_edge(monitor, work)
    } else {
        edge
    };
    Some(WindowPlacement {
        x: i32::try_from(if edge == DockEdge::Left {
            limits.min_x
        } else {
            limits.max_x
        })
        .ok()?,
        y: i32::try_from(if edge == DockEdge::Top {
            limits.min_y
        } else {
            limits.max_y
        })
        .ok()?,
        width: limits.width,
        height: limits.height,
    })
}

/// Keep an existing placement inside a work area with the smallest translation.
///
/// Useful after a monitor/work-area change when a user has dragged the window:
/// a still-valid position is preserved instead of redocking it every time.
/// Oversized dimensions shrink and margins yield exactly as in docking.
#[must_use]
pub fn constrain_placement(
    monitor: PhysicalRect,
    work: PhysicalRect,
    placement: WindowPlacement,
    margin: i32,
) -> Option<WindowPlacement> {
    let limits = placement_limits(monitor, work, [placement.width, placement.height], margin)?;
    Some(WindowPlacement {
        x: i32::try_from(i64::from(placement.x).clamp(limits.min_x, limits.max_x)).ok()?,
        y: i32::try_from(i64::from(placement.y).clamp(limits.min_y, limits.max_y)).ok()?,
        width: limits.width,
        height: limits.height,
    })
}

/// Fit an existing physical placement into a valid work area without redocking.
///
/// Preserves a user-dragged position already inside the area, without imposing
/// a new dock margin. An oversized window shrinks to the work area; an
/// off-screen window moves only the minimum distance needed to become visible.
/// The caller must obtain this work area from the intended monitor (and may
/// first apply `intersect_work_area` to an anomalous system rectangle).
#[must_use]
pub fn fit_placement_to_work_area(
    work: PhysicalRect,
    current: WindowPlacement,
) -> Option<WindowPlacement> {
    constrain_placement(work, work, current, 0)
}

fn valid_rectangle(rectangle: PhysicalRect) -> bool {
    rectangle.left < rectangle.right && rectangle.top < rectangle.bottom
}

fn reserved_edge(monitor: PhysicalRect, work: PhysicalRect) -> DockEdge {
    // Subtract only after widening: two valid i32 desktop coordinates can
    // have a difference greater than i32::MAX.
    let reservations = [
        (
            DockEdge::Bottom,
            i64::from(monitor.bottom) - i64::from(work.bottom),
        ),
        (DockEdge::Top, i64::from(work.top) - i64::from(monitor.top)),
        (
            DockEdge::Left,
            i64::from(work.left) - i64::from(monitor.left),
        ),
        (
            DockEdge::Right,
            i64::from(monitor.right) - i64::from(work.right),
        ),
    ];
    let mut edge = DockEdge::Bottom;
    let mut largest = 0;
    for (candidate, reserved) in reservations {
        if reserved > largest {
            edge = candidate;
            largest = reserved;
        }
    }
    edge
}

struct PlacementLimits {
    min_x: i64,
    max_x: i64,
    min_y: i64,
    max_y: i64,
    width: i32,
    height: i32,
}

fn placement_limits(
    monitor: PhysicalRect,
    work: PhysicalRect,
    window_size: [i32; 2],
    margin: i32,
) -> Option<PlacementLimits> {
    if intersect_work_area(monitor, work) != Some(work)
        || window_size[0] <= 0
        || window_size[1] <= 0
    {
        return None;
    }
    let left = i64::from(work.left);
    let top = i64::from(work.top);
    let right = i64::from(work.right);
    let bottom = i64::from(work.bottom);
    let width = i64::from(window_size[0]).min(right - left);
    let height = i64::from(window_size[1]).min(bottom - top);
    let margin = i64::from(margin.max(0));
    let margin_x = margin.min((right - left - width) / 2);
    let margin_y = margin.min((bottom - top - height) / 2);
    Some(PlacementLimits {
        min_x: left + margin_x,
        max_x: right - width - margin_x,
        min_y: top + margin_y,
        max_y: bottom - height - margin_y,
        width: i32::try_from(width).ok()?,
        height: i32::try_from(height).ok()?,
    })
}

/// Checked conversion before passing persisted coordinates to Win32.
impl WindowPlacement {
    pub fn checked_bounds(self) -> Option<PhysicalRect> {
        if self.width <= 0 || self.height <= 0 {
            return None;
        }
        Some(PhysicalRect {
            left: self.x,
            top: self.y,
            right: self.x.checked_add(self.width)?,
            bottom: self.y.checked_add(self.height)?,
        })
    }
}

/// Physical geometry of the independent RC2 surface; never reparents a window.
pub fn taskbar_placement(
    area: PhysicalRect,
    offset: i32,
    requested_width: i32,
    requested_height: i32,
) -> Option<WindowPlacement> {
    let height = area.bottom.checked_sub(area.top)?;
    let width = area.right.checked_sub(area.left)?;
    if !(24..=240).contains(&height) || width < height * 5 {
        return None;
    }
    let strip_height = if requested_height == 0 {
        height
    } else {
        requested_height.clamp(24, height)
    };
    let strip_width = if requested_width == 0 {
        (height * 32 / 7).min(width / 3)
    } else {
        requested_width.clamp(160.min(width), width)
    };
    Some(WindowPlacement {
        x: area
            .left
            .checked_add(offset.clamp(0, width - strip_width))?,
        y: area.bottom.checked_sub(strip_height)?,
        width: strip_width,
        height: strip_height,
    })
}
