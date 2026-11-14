use cpu_express::platform::{
    DockEdge, PhysicalRect, WindowPlacement, constrain_placement, dock_margin_for_dpi,
    dock_placement, dock_placement_at_edge, fit_placement_to_work_area, intersect_work_area,
};

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> PhysicalRect {
    PhysicalRect {
        left,
        top,
        right,
        bottom,
    }
}

fn assert_contained(placement: WindowPlacement, work: PhysicalRect) {
    assert!(placement.width > 0 && placement.height > 0);
    assert!(placement.x >= work.left && placement.y >= work.top);
    assert!(i64::from(placement.x) + i64::from(placement.width) <= i64::from(work.right));
    assert!(i64::from(placement.y) + i64::from(placement.height) <= i64::from(work.bottom));
}

#[test]
fn four_edges_across_primary_left_above_and_right_monitors_at_four_dpis() {
    let monitors = [
        rect(0, 0, 1920, 1080),
        rect(-2560, -300, 0, 1140),
        rect(0, -1440, 2560, 0),
        rect(1920, 250, 4480, 1690),
    ];
    for monitor in monitors {
        for dpi in [96_u32, 120, 144, 192] {
            let width = (640 * dpi / 96) as i32;
            let height = (140 * dpi / 96) as i32;
            let reservation = (40 * dpi / 96) as i32;
            let margin = dock_margin_for_dpi(dpi);
            for edge in [
                DockEdge::Bottom,
                DockEdge::Top,
                DockEdge::Left,
                DockEdge::Right,
            ] {
                let mut work = monitor;
                match edge {
                    DockEdge::Bottom => work.bottom -= reservation,
                    DockEdge::Top => work.top += reservation,
                    DockEdge::Left => work.left += reservation,
                    DockEdge::Right => work.right -= reservation,
                    DockEdge::Auto => unreachable!(),
                }
                let placement = dock_placement(monitor, work, [width, height], margin)
                    .expect("valid physical work area");
                assert_contained(placement, work);
                assert_eq!((placement.width, placement.height), (width, height));
                assert_eq!(
                    placement,
                    dock_placement_at_edge(monitor, work, [width, height], margin, edge)
                        .expect("explicit edge"),
                );
                assert_eq!(
                    placement.x,
                    if edge == DockEdge::Left {
                        work.left + margin
                    } else {
                        work.right - width - margin
                    },
                );
                assert_eq!(
                    placement.y,
                    if edge == DockEdge::Top {
                        work.top + margin
                    } else {
                        work.bottom - height - margin
                    },
                );
            }
        }
    }
}

#[test]
fn auto_hidden_taskbar_can_choose_each_edge_with_physical_inset() {
    let monitor = rect(-1920, -1080, 0, 0);
    for (edge, x, y) in [
        (DockEdge::Auto, -648, -148),
        (DockEdge::Bottom, -648, -148),
        (DockEdge::Top, -648, -1072),
        (DockEdge::Left, -1912, -148),
        (DockEdge::Right, -648, -148),
    ] {
        let placement = dock_placement_at_edge(monitor, monitor, [640, 140], 8, edge)
            .expect("auto-hide has no reserved rectangle");
        assert_eq!((placement.x, placement.y), (x, y));
        assert_contained(placement, monitor);
    }
}

#[test]
fn multiple_reservations_choose_largest_and_keep_stable_ties() {
    let monitor = rect(0, 0, 1920, 1080);
    for (work, expected_edge) in [
        (rect(40, 40, 1880, 1040), DockEdge::Bottom),
        (rect(40, 40, 1880, 1080), DockEdge::Top),
        (rect(40, 0, 1880, 1080), DockEdge::Left),
        (rect(0, 0, 1880, 1080), DockEdge::Right),
        (rect(100, 30, 1900, 1040), DockEdge::Left),
        (rect(10, 120, 1900, 1040), DockEdge::Top),
    ] {
        assert_eq!(
            dock_placement(monitor, work, [640, 140], 8),
            dock_placement_at_edge(monitor, work, [640, 140], 8, expected_edge),
        );
    }
}

#[test]
fn explicit_edge_does_not_override_other_reserved_areas() {
    let monitor = rect(0, 0, 1920, 1080);
    let work = rect(120, 50, 1820, 1000);
    for edge in [
        DockEdge::Bottom,
        DockEdge::Top,
        DockEdge::Left,
        DockEdge::Right,
    ] {
        let placement = dock_placement_at_edge(monitor, work, [640, 140], 8, edge)
            .expect("explicit dock edge respects complete work area");
        assert_contained(placement, work);
    }
}

#[test]
fn tiny_work_areas_shrink_each_axis_and_yield_margins() {
    let monitor = rect(-1000, -1000, 1000, 1000);
    for work in [
        rect(-1, -1, 0, 0),
        rect(-600, 40, 0, 140),
        rect(0, 0, 645, 143),
        rect(0, 0, 1, 900),
        rect(0, 0, 900, 1),
    ] {
        for margin in [i32::MIN, -1, 0, 8, i32::MAX] {
            for edge in [
                DockEdge::Auto,
                DockEdge::Bottom,
                DockEdge::Top,
                DockEdge::Left,
                DockEdge::Right,
            ] {
                let placement = dock_placement_at_edge(monitor, work, [640, 140], margin, edge)
                    .expect("positive area, however small");
                assert_contained(placement, work);
                assert_eq!(placement.width, 640.min(work.right - work.left));
                assert_eq!(placement.height, 140.min(work.bottom - work.top));
            }
        }
    }
    let placement = dock_placement_at_edge(
        monitor,
        rect(0, 0, 645, 143),
        [640, 140],
        8,
        DockEdge::Bottom,
    )
    .expect("margins yield independently on each axis");
    assert_eq!((placement.x, placement.y), (3, 2));
}

#[test]
fn work_area_intersection_is_explicit_and_never_fabricates_missing_space() {
    let monitor = rect(-1920, -400, 0, 680);
    let reported = rect(-2000, -450, 80, 640);
    let clipped = rect(-1920, -400, 0, 640);
    assert_eq!(intersect_work_area(monitor, reported), Some(clipped));
    assert!(dock_placement(monitor, reported, [640, 140], 8).is_none());
    let placement = dock_placement(monitor, clipped, [640, 140], 8)
        .expect("caller deliberately uses the clipped system rectangle");
    assert_eq!((placement.x, placement.y), (-648, 492));
    assert_contained(placement, clipped);
    for bad in [
        rect(10, 0, 20, 100),
        rect(0, 0, 100, 100),
        rect(-100, 700, 0, 800),
        rect(-100, 0, -100, 100),
        rect(0, 0, -100, 100),
        rect(-100, 100, 0, 0),
    ] {
        assert_eq!(intersect_work_area(monitor, bad), None);
    }
}

#[test]
fn inverted_or_empty_monitors_and_nonpositive_sizes_are_rejected() {
    let work = rect(0, 0, 1000, 1000);
    for monitor in [
        rect(0, 0, 0, 1000),
        rect(1000, 0, 0, 1000),
        rect(0, 1000, 1000, 0),
    ] {
        assert_eq!(intersect_work_area(monitor, work), None);
        assert_eq!(dock_placement(monitor, work, [640, 140], 8), None);
    }
    for size in [[0, 140], [640, 0], [-1, 140], [640, i32::MIN]] {
        for edge in [
            DockEdge::Auto,
            DockEdge::Bottom,
            DockEdge::Top,
            DockEdge::Left,
            DockEdge::Right,
        ] {
            assert_eq!(dock_placement_at_edge(work, work, size, 8, edge), None);
        }
    }
}

#[test]
fn extreme_integer_coordinates_sizes_margins_and_dpi_remain_bounded() {
    let monitor = rect(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    for work in [
        monitor,
        rect(i32::MIN, i32::MIN, i32::MIN + 1, i32::MIN + 1),
        rect(i32::MAX - 1, i32::MAX - 1, i32::MAX, i32::MAX),
        rect(i32::MIN, -1, 1, i32::MAX),
    ] {
        for size in [[1, 1], [640, 140], [i32::MAX, i32::MAX]] {
            for margin in [i32::MIN, 0, 8, i32::MAX, dock_margin_for_dpi(u32::MAX)] {
                for edge in [
                    DockEdge::Auto,
                    DockEdge::Bottom,
                    DockEdge::Top,
                    DockEdge::Left,
                    DockEdge::Right,
                ] {
                    let placement = dock_placement_at_edge(monitor, work, size, margin, edge)
                        .expect("representable physical placement");
                    assert_contained(placement, work);
                }
            }
        }
    }
    assert_eq!(dock_margin_for_dpi(u32::MAX), 357_913_941);
    assert_eq!(dock_margin_for_dpi(0), 8);
    assert_eq!(dock_margin_for_dpi(101), 8);
    assert_eq!(dock_margin_for_dpi(102), 9);
}

#[test]
fn constraints_preserve_user_drag_and_only_move_as_much_as_needed() {
    let monitor = rect(1920, -500, 4480, 940);
    let work = rect(1920, -500, 4480, 880);
    let dragged = WindowPlacement {
        x: 2010,
        y: -250,
        width: 960,
        height: 210,
    };
    assert_eq!(
        constrain_placement(monitor, work, dragged, 12),
        Some(dragged)
    );
    let lost = WindowPlacement {
        x: i32::MAX,
        y: i32::MIN,
        ..dragged
    };
    let restored = constrain_placement(monitor, work, lost, 12).expect("recover off-screen window");
    assert_eq!((restored.x, restored.y), (3508, -488));
    assert_contained(restored, work);
    assert_eq!(
        constrain_placement(monitor, work, restored, 12),
        Some(restored)
    );
}

#[test]
fn constraint_shrinks_to_new_monitor_and_preserves_relative_valid_axis() {
    let monitor = rect(-600, 0, 0, 140);
    let work = rect(-600, 40, 0, 140);
    let old = WindowPlacement {
        x: 2000,
        y: -500,
        width: 1280,
        height: 280,
    };
    let fitted = constrain_placement(monitor, work, old, 16).expect("small replacement monitor");
    assert_eq!(
        fitted,
        WindowPlacement {
            x: -600,
            y: 40,
            width: 600,
            height: 100
        }
    );
    assert_eq!(
        constrain_placement(monitor, work, WindowPlacement { width: 0, ..old }, 16),
        None
    );
}

#[test]
fn display_change_fit_preserves_a_dragged_position_even_at_the_work_edge() {
    let work = rect(-1920, -400, 0, 640);
    let current = WindowPlacement {
        x: -1920,
        y: -400,
        width: 640,
        height: 140,
    };
    assert_eq!(fit_placement_to_work_area(work, current), Some(current));
    let displaced = WindowPlacement {
        x: -500,
        y: -300,
        ..current
    };
    let fitted = fit_placement_to_work_area(work, displaced).expect("fit each axis independently");
    assert_eq!((fitted.x, fitted.y), (-640, -300));
    assert_contained(fitted, work);
}

#[test]
fn display_change_fit_handles_extreme_origins_sizes_and_invalid_work_areas() {
    let full = rect(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    let huge = WindowPlacement {
        x: i32::MAX,
        y: i32::MIN,
        width: i32::MAX,
        height: i32::MAX,
    };
    let fitted =
        fit_placement_to_work_area(full, huge).expect("all intermediate arithmetic is wide");
    assert_eq!(fitted.x, 0);
    assert_eq!(fitted.y, i32::MIN);
    assert_contained(fitted, full);
    let tiny = rect(i32::MAX - 1, i32::MAX - 1, i32::MAX, i32::MAX);
    let fitted = fit_placement_to_work_area(tiny, huge).expect("shrink to one physical pixel");
    assert_eq!((fitted.width, fitted.height), (1, 1));
    assert_contained(fitted, tiny);
    for invalid in [rect(0, 0, 0, 100), rect(1, 0, 0, 100), rect(0, 100, 100, 0)] {
        assert_eq!(fit_placement_to_work_area(invalid, huge), None);
    }
}

#[test]
fn taskbar_small_width_extremes_and_negative_origins_are_safe() {
    use cpu_express::platform::taskbar_placement;
    let area = rect(-120, -24, 0, 0);
    let fitted = taskbar_placement(area, i32::MAX, 160, 24).unwrap();
    assert_eq!((fitted.x, fitted.width), (-120, 120));
    assert_contained(fitted, area);
    assert!(taskbar_placement(rect(i32::MIN, 0, i32::MAX, 60), 0, 0, 0).is_none());
    assert!(taskbar_placement(rect(0, 0, 60, 1080), 0, 0, 0).is_none());
    assert!(taskbar_placement(rect(0, 0, 0, 0), 0, 0, 0).is_none());
    let area = rect(-1920, 1020, 0, 1080);
    for offset in [i32::MIN, 0, i32::MAX] {
        for size in [i32::MIN, 0, 1, i32::MAX] {
            assert_contained(taskbar_placement(area, offset, size, size).unwrap(), area);
        }
    }
}

#[test]
fn persisted_bounds_reject_overflow_before_native_monitor_lookup() {
    let p = WindowPlacement {
        x: i32::MAX,
        y: 0,
        width: 640,
        height: 140,
    };
    assert!(p.checked_bounds().is_none());
    assert!(
        WindowPlacement {
            x: 0,
            y: i32::MAX,
            ..p
        }
        .checked_bounds()
        .is_none()
    );
    assert!(
        WindowPlacement {
            x: 0,
            width: 0,
            ..p
        }
        .checked_bounds()
        .is_none()
    );
    let negative = WindowPlacement {
        x: -1920,
        y: -400,
        ..p
    };
    assert_eq!(
        negative.checked_bounds(),
        Some(rect(-1920, -400, -1280, -260))
    );
}

#[test]
fn panel_recovery_handles_removed_monitor_and_scaled_negative_origin() {
    use cpu_express::platform::fit_panel_rect;
    let work = rect(-1920, -300, 0, 1040);
    let fitted = fit_panel_rect([3000.0, 200.0, 760.0, 720.0], 1.5, work);
    assert_eq!(fitted[0], -760.0);
    assert!((fitted[1] - (-40.0 / 1.5)).abs() < 0.001);
    let valid = [-1000.25, 0.25, 400.0, 400.0];
    assert_eq!(fit_panel_rect(valid, 1.5, work), valid);
    let tiny = fit_panel_rect(valid, 2.0, rect(0, 0, 640, 480));
    assert_eq!(tiny, [0.0, 0.0, 320.0, 240.0]);
}

#[test]
fn panel_invalid_persistence_scale_and_transient_work_area_are_safe() {
    use cpu_express::platform::{fit_panel_rect, normalize_panel_rect};
    assert_eq!(
        normalize_panel_rect([f32::NAN, f32::INFINITY, -1.0, 0.0]),
        [40.0, 40.0, 760.0, 720.0]
    );
    let valid = [-1000.0, 10.0, 760.0, 720.0];
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::MAX] {
        assert_eq!(fit_panel_rect(valid, scale, rect(0, 0, 1920, 1080)), valid);
    }
    assert_eq!(fit_panel_rect(valid, 1.0, rect(0, 0, 0, 0)), valid);
}
