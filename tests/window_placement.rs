use cpu_express::platform::{PhysicalRect, WindowPlacement, dock_margin_for_dpi, dock_placement};

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> PhysicalRect {
    PhysicalRect {
        left,
        top,
        right,
        bottom,
    }
}

#[test]
fn taskbar_on_each_edge_keeps_the_whole_window_in_work_area() {
    let monitor = rect(0, 0, 1920, 1080);
    for (work, expected_x, expected_y) in [
        (rect(0, 0, 1920, 1040), 1272, 892),
        (rect(0, 40, 1920, 1080), 1272, 48),
        (rect(40, 0, 1920, 1080), 48, 932),
        (rect(0, 0, 1880, 1080), 1232, 932),
    ] {
        let placement = dock_placement(monitor, work, [640, 140], 8).expect("valid work area");
        assert_eq!(
            placement,
            WindowPlacement {
                x: expected_x,
                y: expected_y,
                width: 640,
                height: 140,
            }
        );
        assert!(placement.x >= work.left && placement.y >= work.top);
        assert!(placement.x + placement.width <= work.right);
        assert!(placement.y + placement.height <= work.bottom);
    }
}

#[test]
fn a_negative_monitor_origin_is_preserved() {
    let placement = dock_placement(
        rect(-1920, -400, 0, 680),
        rect(-1920, -400, 0, 640),
        [640, 140],
        8,
    )
    .expect("negative origins are valid");
    assert_eq!((placement.x, placement.y), (-648, 492));
}

#[test]
fn mixed_dpi_uses_physical_size_without_rescaling_global_origin() {
    let placement = dock_placement(
        rect(1920, 0, 4480, 1440),
        rect(1920, 0, 4480, 1380),
        [960, 210], // 640 x 140 logical points at 150%.
        dock_margin_for_dpi(144),
    )
    .expect("valid scaled monitor");
    assert_eq!((placement.x, placement.y), (3508, 1158));
    assert_eq!((placement.width, placement.height), (960, 210));
    for (dpi, expected) in [(0, 8), (96, 8), (120, 10), (144, 12), (192, 16)] {
        assert_eq!(dock_margin_for_dpi(dpi), expected);
    }
}

#[test]
fn an_oversized_window_is_fitted_and_margins_give_way() {
    let work = rect(-600, 40, 0, 140);
    let placement = dock_placement(rect(-600, 0, 0, 140), work, [640, 140], 8)
        .expect("small work area is valid");
    assert_eq!(
        placement,
        WindowPlacement {
            x: -600,
            y: 40,
            width: 600,
            height: 100,
        }
    );
}

#[test]
fn no_reserved_edge_defaults_to_bottom_right() {
    let monitor = rect(0, 0, 1920, 1080);
    let placement = dock_placement(monitor, monitor, [640, 140], 8).expect("valid monitor");
    assert_eq!((placement.x, placement.y), (1272, 932));
}

#[test]
fn invalid_rectangles_and_window_sizes_are_rejected() {
    let monitor = rect(0, 0, 1920, 1080);
    assert!(dock_placement(monitor, rect(0, 0, 0, 1080), [640, 140], 8).is_none());
    assert!(dock_placement(monitor, rect(-1, 0, 1920, 1080), [640, 140], 8).is_none());
    assert!(dock_placement(monitor, monitor, [0, 140], 8).is_none());
    assert!(dock_placement(monitor, monitor, [640, -1], 8).is_none());
}

#[test]
fn extreme_desktop_coordinates_do_not_overflow() {
    let monitor = rect(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    let placement = dock_placement(monitor, monitor, [640, 140], 8).expect("valid integer range");
    assert_eq!(placement.x, i32::MAX - 640 - 8);
    assert_eq!(placement.y, i32::MAX - 140 - 8);
}
