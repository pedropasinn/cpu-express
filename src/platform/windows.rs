//! Narrow Win32 boundary. All window operations run on the owning eframe thread.
//!
//! Winit creates the window and establishes per-monitor DPI awareness. Geometry
//! calls verify both window and thread awareness, rather than changing process
//! or thread DPI policy. Requires Windows 10 version 1607 or later.
//!
//! Preserve `ViewportBuilder::with_active(false)`: in the locked winit 0.30.13,
//! `set_visible(true)` then uses SW_SHOWNOACTIVATE. Visibility/minimize changes
//! synchronize winit's cached flags; a raw ShowWindow alone is insufficient.
//! Topmost changes belong to the controller's deferred egui viewport commands.

use std::{io, mem, ptr};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
    System::Threading::GetCurrentThreadId,
    UI::{
        HiDpi::{
            DPI_AWARENESS_PER_MONITOR_AWARE, GetAwarenessFromDpiAwarenessContext, GetDpiForWindow,
            GetThreadDpiAwarenessContext, GetWindowDpiAwarenessContext,
        },
        Shell::{
            ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_GETAUTOHIDEBAREX, APPBARDATA,
            SHAppBarMessage,
        },
        WindowsAndMessaging::{
            GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible,
            MB_ICONERROR, MB_OK, MessageBoxW, SW_SHOWMINNOACTIVE, SW_SHOWNOACTIVATE,
            SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, ShowWindow,
        },
    },
};

use super::{
    DesktopState, DockEdge, PhysicalRect, WindowPlacement, dock_margin_for_dpi,
    dock_placement_at_edge, fit_placement_to_work_area, intersect_work_area,
};

fn rect(rect: RECT) -> PhysicalRect {
    PhysicalRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

fn os_error(operation: &str) -> String {
    format!("{operation}: {}", io::Error::last_os_error())
}

/// Position an independent surface within the taskbar rectangle. Explorer's
/// composition bridge can occlude cross-process children despite IsWindowVisible.
/// Keeping our own top-level surface avoids that unsupported embedding path.
pub fn embed_taskbar(
    frame: &eframe::Frame,
    offset: i32,
    requested_width: i32,
    requested_height: i32,
) -> Result<(), String> {
    place_strip(
        frame,
        offset,
        requested_width,
        requested_height,
        true,
        false,
    )
}

pub(super) fn place_strip(
    frame: &eframe::Frame,
    offset: i32,
    requested_width: i32,
    requested_height: i32,
    fit_height: bool,
    rounded: bool,
) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GWL_STYLE, GetParent, GetWindowLongPtrW, SWP_FRAMECHANGED, SetParent,
        SetWindowLongPtrW, WS_CHILD, WS_POPUP,
    };
    with_hwnd(frame, |hwnd| {
        require_physical_coordinates(hwnd)?;
        let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
        // SAFETY: valid class string, initialized rectangle, own window on its UI thread.
        unsafe {
            let taskbar = FindWindowW(class.as_ptr(), ptr::null());
            let mut area = RECT::default();
            if taskbar.is_null() || GetWindowRect(taskbar, &mut area) == 0 {
                return Err(os_error("Taskbar geometry"));
            }
            let placement = super::strip_placement(
                rect(area),
                offset,
                requested_width,
                requested_height,
                fit_height,
            )
            .ok_or_else(|| {
                "Taskbar geometry is unavailable or unsupported; use floating mode.".to_owned()
            })?;
            if GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD as isize != 0 {
                windows_sys::Win32::Foundation::SetLastError(0);
                SetParent(hwnd, ptr::null_mut());
                if !GetParent(hwnd).is_null() || windows_sys::Win32::Foundation::GetLastError() != 0
                {
                    return Err(os_error("Detach taskbar surface"));
                }
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                windows_sys::Win32::Foundation::SetLastError(0);
                SetWindowLongPtrW(
                    hwnd,
                    GWL_STYLE,
                    (style & !(WS_CHILD as isize)) | WS_POPUP as isize,
                );
                if windows_sys::Win32::Foundation::GetLastError() != 0 {
                    return Err(os_error("Taskbar surface style"));
                }
            }
            if SetWindowPos(
                hwnd,
                ptr::null_mut(),
                placement.x,
                placement.y,
                placement.width,
                placement.height,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
            ) == 0
            {
                return Err(os_error("Taskbar surface placement"));
            }
            strip_corners(hwnd, placement.width, placement.height, Some(rounded))?;
            set_tool_window(hwnd, true)?;
            restack_above(hwnd, taskbar)?;
            if outer_placement(hwnd)? != placement {
                return Err("Windows não confirmou a posição e o tamanho da faixa.".to_owned());
            }
        }
        Ok(())
    })
}

/// A tool window has no taskbar button and is left out of Alt+Tab: the strip is
/// part of the bar, not another application in it.
fn set_tool_window(hwnd: HWND, tool: bool) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };
    // SAFETY: style query and update on the validated own window, on its UI thread.
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let wanted = if tool {
            (current | WS_EX_TOOLWINDOW as isize) & !(WS_EX_APPWINDOW as isize)
        } else {
            current & !(WS_EX_TOOLWINDOW as isize)
        };
        if wanted != current {
            windows_sys::Win32::Foundation::SetLastError(0);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, wanted);
            if windows_sys::Win32::Foundation::GetLastError() != 0 {
                return Err(os_error("Estilo da faixa"));
            }
        }
    }
    Ok(())
}

/// Puts the strip directly above the taskbar in the window order, without taking
/// focus. Windows raises the taskbar whenever another window is activated, which
/// would otherwise cover the strip. Following the taskbar, instead of forcing the
/// very top, also lets a full-screen application cover both.
/// Returns whether the order had to change.
fn restack_above(hwnd: HWND, taskbar: HWND) -> Result<bool, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GW_HWNDPREV, GetWindow, HWND_TOPMOST, SWP_NOMOVE,
    };
    // SAFETY: read-only order query, then a reorder of the own window only.
    unsafe {
        let above = GetWindow(taskbar, GW_HWNDPREV);
        if above == hwnd {
            return Ok(false);
        }
        let after = if above.is_null() { HWND_TOPMOST } else { above };
        if SetWindowPos(
            hwnd,
            after,
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        ) == 0
            && SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            ) == 0
        {
            return Err(os_error("Ordem da faixa"));
        }
    }
    Ok(true)
}

/// Cheap periodic check for the strip mode: two window queries when nothing changed.
pub fn keep_strip_above_taskbar(frame: &eframe::Frame) -> Result<bool, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;
    with_hwnd(frame, |hwnd| {
        let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
        // SAFETY: valid NUL-terminated class string; the handle is only compared and queried.
        let taskbar = unsafe { FindWindowW(class.as_ptr(), ptr::null()) };
        // SAFETY: read-only visibility queries on the own window.
        if taskbar.is_null() || unsafe { IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 } {
            return Ok(false);
        }
        restack_above(hwnd, taskbar)
    })
}

/// Confirmação nativa posterior à primeira pintura; não é certificação visual do compositor.
pub fn confirm_strip_surface(frame: &eframe::Frame) -> Result<(), String> {
    use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
    with_hwnd(frame, |hwnd| {
        let mut client = RECT::default();
        let mut cloaked = 0u32;
        // SAFETY: consultas somente leitura do HWND próprio com saídas inicializadas e tamanhos corretos.
        unsafe {
            if IsWindowVisible(hwnd) == 0
                || IsIconic(hwnd) != 0
                || windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client)
                    == 0
                || client.right <= client.left
                || client.bottom <= client.top
                || DwmGetWindowAttribute(
                    hwnd,
                    DWMWA_CLOAKED as u32,
                    (&mut cloaked as *mut u32).cast(),
                    mem::size_of::<u32>() as u32,
                ) < 0
                || cloaked != 0
            {
                return Err(
                    "A superfície da faixa não pôde ser confirmada pelo Windows.".to_owned(),
                );
            }
        }
        Ok(())
    })
}

/// Leitura física evita realimentar o deslocamento local quando a própria janela se move.
pub fn strip_environment(
    frame: &eframe::Frame,
) -> Result<(PhysicalRect, WindowPlacement, [i32; 2]), String> {
    use windows_sys::Win32::{
        Foundation::POINT,
        UI::WindowsAndMessaging::{FindWindowW, GetCursorPos},
    };
    with_hwnd(frame, |hwnd| {
        require_physical_coordinates(hwnd)?;
        let class = [
            83u16, 104, 101, 108, 108, 95, 84, 114, 97, 121, 87, 110, 100, 0,
        ];
        let mut area = RECT::default();
        let mut pointer = POINT::default();
        // SAFETY: consultas somente leitura com buffers inicializados; nenhum handle é retido.
        unsafe {
            let bar = FindWindowW(class.as_ptr(), ptr::null());
            if bar.is_null()
                || GetWindowRect(bar, &mut area) == 0
                || GetCursorPos(&mut pointer) == 0
            {
                return Err(os_error("Não foi possível consultar a faixa"));
            }
        }
        Ok((rect(area), outer_placement(hwnd)?, [pointer.x, pointer.y]))
    })
}

pub fn taskbar_status(frame: &eframe::Frame) -> Result<serde_json::Value, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, GetClassNameW, GetParent};
    with_hwnd(frame, |hwnd| {
        let mut class = [0u16; 256];
        let mut parent_area = RECT::default();
        // SAFETY: live own window, bounded writable buffers, read-only queries.
        let (count, parent) = unsafe {
            let parent = GetParent(hwnd);
            let count = if parent.is_null() {
                0
            } else {
                GetClassNameW(parent, class.as_mut_ptr(), class.len() as i32)
            };
            (count, parent)
        };
        let parent_class = String::from_utf16_lossy(&class[..count.max(0) as usize]);
        let placement = outer_placement(hwnd)?;
        let mut client = RECT::default();
        // SAFETY: validated own HWND, initialized output rectangle.
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut client);
        }

        let taskbar_class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
        // SAFETY: read-only taskbar lookup and initialized output rectangle.
        unsafe {
            let taskbar = FindWindowW(taskbar_class.as_ptr(), ptr::null());
            if !taskbar.is_null() {
                GetWindowRect(taskbar, &mut parent_area);
            }
        }
        let aligned = placement.y >= parent_area.top
            && placement.y + placement.height <= parent_area.bottom
            && placement.x >= parent_area.left
            && placement.x + placement.width <= parent_area.right;
        let title: Vec<u16> = super::PANEL_TITLE.encode_utf16().chain(Some(0)).collect();
        let mut panel_rect = RECT::default();
        // SAFETY: valid title buffer, initialized outputs and read-only own-process queries.
        let panel_visible = unsafe {
            let mut panel = FindWindowW(ptr::null(), title.as_ptr());
            let mut found = false;
            for _ in 0..64 {
                if panel.is_null() {
                    break;
                }
                let mut process = 0;
                GetWindowThreadProcessId(panel, &mut process);
                if process == windows_sys::Win32::System::Threading::GetCurrentProcessId() {
                    found =
                        GetWindowRect(panel, &mut panel_rect) != 0 && IsWindowVisible(panel) != 0;
                    break;
                }
                panel = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowExW(
                    ptr::null_mut(),
                    panel,
                    ptr::null(),
                    title.as_ptr(),
                );
            }
            found
        };

        Ok(
            serde_json::json!({"parent_class":parent_class, "embedded": !parent.is_null() && parent_class == "Shell_TrayWnd",
            "client_width":client.right-client.left, "client_height":client.bottom-client.top, "x": placement.x, "y": placement.y, "width":placement.width, "height":placement.height,
            "taskbar_width":parent_area.right - parent_area.left, "taskbar_height":parent_area.bottom - parent_area.top, "taskbar_aligned": aligned, "surface_mode": "independent",
            "panel": {"visible":panel_visible, "width":panel_rect.right-panel_rect.left,
                "height":panel_rect.bottom-panel_rect.top, "x":panel_rect.left, "y":panel_rect.top}}),
        )
    })
}

pub fn detach_taskbar(frame: &eframe::Frame) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetParent, GetWindowLongPtrW, SWP_FRAMECHANGED, SetParent, SetWindowLongPtrW,
        WS_CHILD, WS_POPUP,
    };
    with_hwnd(frame, |hwnd| {
        // SAFETY: own live window, no ownership transfer; restore its top-level style.
        unsafe {
            windows_sys::Win32::Foundation::SetLastError(0);
            SetParent(hwnd, ptr::null_mut());
            if windows_sys::Win32::Foundation::GetLastError() != 0 || !GetParent(hwnd).is_null() {
                return Err(os_error("SetParent"));
            }
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            windows_sys::Win32::Foundation::SetLastError(0);
            SetWindowLongPtrW(
                hwnd,
                GWL_STYLE,
                (style & !(WS_CHILD as isize)) | WS_POPUP as isize,
            );
            if windows_sys::Win32::Foundation::GetLastError() != 0 {
                return Err(os_error("SetWindowLongPtrW"));
            }
            if SetWindowPos(
                hwnd,
                ptr::null_mut(),
                40,
                40,
                640,
                140,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
            ) == 0
            {
                return Err(os_error("SetWindowPos"));
            }
        }
        dock_window_at_edge(frame, DockEdge::Auto)
    })
}

/// Restore a floating rectangle, constrained to the nearest monitor's work area.
pub fn place_floating(frame: &eframe::Frame, saved: [i32; 4]) -> Result<(), String> {
    with_hwnd(frame, |hwnd| {
        require_physical_coordinates(hwnd)?;
        strip_corners(hwnd, 0, 0, None)?;
        // A floating window stays reachable with Alt+Tab.
        set_tool_window(hwnd, false)?;
        let [x, y, width, height] = saved;
        let requested = WindowPlacement {
            x,
            y,
            width,
            height,
        };
        let checked = requested
            .checked_bounds()
            .ok_or_else(|| "Invalid or overflowing floating rectangle".to_owned())?;
        let bounds = RECT {
            left: checked.left,
            top: checked.top,
            right: checked.right,
            bottom: checked.bottom,
        };
        let mut info = MONITORINFO {
            cbSize: mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        // SAFETY: initialized rectangle and sized output, read-only nearest-monitor lookup.
        unsafe {
            let monitor = windows_sys::Win32::Graphics::Gdi::MonitorFromRect(
                &bounds,
                MONITOR_DEFAULTTONEAREST,
            );
            if GetMonitorInfoW(monitor, &mut info) == 0 {
                return Err(os_error("Floating monitor"));
            }
        }
        let placement = fit_placement_to_work_area(work_area(&info)?, requested)
            .ok_or_else(|| "Invalid floating rectangle".to_owned())?;
        apply_placement(hwnd, outer_placement(hwnd)?, placement)
    })
}

/// O atributo 33 é suportado no Windows 11. Em sistemas anteriores, usa retângulo.
/// A região explícita também cobre janelas sem moldura que o DWM não arredonda sozinho.
fn strip_corners(hwnd: HWND, width: i32, height: i32, rounded: Option<bool>) -> Result<(), String> {
    use windows_sys::Win32::Graphics::{
        Dwm::{
            DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DEFAULT, DWMWCP_DONOTROUND, DWMWCP_ROUND,
            DwmSetWindowAttribute,
        },
        Gdi::{CreateRoundRectRgn, DeleteObject, SetWindowRgn},
    };
    let preference = match rounded {
        Some(true) => DWMWCP_ROUND,
        Some(false) => DWMWCP_DONOTROUND,
        None => DWMWCP_DEFAULT,
    };
    // SAFETY: HWND próprio, preferência inicializada e tamanho correto; a região passa ao Windows somente no sucesso.
    unsafe {
        let supported = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&preference as *const i32).cast(),
            mem::size_of::<i32>() as u32,
        ) >= 0;
        let region = if supported && rounded == Some(true) {
            let region = CreateRoundRectRgn(0, 0, width + 1, height + 1, 12, 12);
            if region.is_null() {
                return Err(os_error("Cantos da faixa"));
            }
            region
        } else {
            ptr::null_mut()
        };
        if SetWindowRgn(hwnd, region, 1) == 0 {
            if !region.is_null() {
                DeleteObject(region);
            }
            return Err(os_error("Região da faixa"));
        }
    }
    Ok(())
}

/// Borrow, validate and use the handle in one synchronous operation. No caller
/// stores it, owns it, destroys it or sends it to another thread.
fn with_hwnd<T>(
    frame: &eframe::Frame,
    operation: impl FnOnce(HWND) -> Result<T, String>,
) -> Result<T, String> {
    if frame.winit_window().is_none() {
        return Err("A janela eframe ainda não está disponível.".to_owned());
    }
    let borrowed = frame
        .window_handle()
        .map_err(|error| format!("Janela nativa indisponível: {error}"))?;
    let RawWindowHandle::Win32(handle) = borrowed.as_raw() else {
        return Err("A janela não forneceu um handle Win32.".to_owned());
    };
    let hwnd = handle.hwnd.get() as HWND;
    // SAFETY: the nonzero HWND is borrowed from the live Frame; IsWindow only
    // queries it. Frame keeps the winit window alive throughout this operation.
    if unsafe { IsWindow(hwnd) } == 0 {
        return Err("O handle da janela já não é válido.".to_owned());
    }
    // SAFETY: hwnd is live; a null process-id output is explicitly allowed.
    let owner = unsafe { GetWindowThreadProcessId(hwnd, ptr::null_mut()) };
    if owner == 0 {
        return Err(os_error("Não foi possível identificar a thread da janela"));
    }
    // SAFETY: GetCurrentThreadId has no pointer arguments or ownership effects.
    if owner != unsafe { GetCurrentThreadId() } {
        return Err("A operação deve ocorrer na thread da interface do eframe.".to_owned());
    }
    operation(hwnd)
}

fn require_physical_coordinates(hwnd: HWND) -> Result<(), String> {
    // SAFETY: these calls only query the current thread and the validated live
    // HWND; they do not change DPI policy or retain any handle.
    let (thread_context, window_context) = unsafe {
        (
            GetThreadDpiAwarenessContext(),
            GetWindowDpiAwarenessContext(hwnd),
        )
    };
    if thread_context.is_null() || window_context.is_null() {
        return Err("Windows não informou o contexto de DPI da janela.".to_owned());
    }
    // SAFETY: contexts are non-null values returned by the documented query
    // functions above. The awareness query has no output pointers or mutations.
    let physical = unsafe {
        GetAwarenessFromDpiAwarenessContext(thread_context) == DPI_AWARENESS_PER_MONITOR_AWARE
            && GetAwarenessFromDpiAwarenessContext(window_context)
                == DPI_AWARENESS_PER_MONITOR_AWARE
    };
    if !physical {
        return Err(
            "Posicionamento requer DPI por monitor, configurado pelo winit ao iniciar a janela."
                .to_owned(),
        );
    }
    Ok(())
}

fn monitor_info(hwnd: HWND) -> Result<MONITORINFO, String> {
    // SAFETY: hwnd belongs to the live window on its owning thread. This is a
    // read-only query; the nearest monitor also recovers off-screen windows.
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return Err("Windows não identificou o monitor da janela.".to_owned());
    }
    let mut info = MONITORINFO {
        cbSize: mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: info is initialized, correctly sized, uniquely borrowed output
    // storage. The monitor was returned above and is not retained.
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return Err(os_error(
            "Não foi possível consultar a área útil do monitor",
        ));
    }
    Ok(info)
}

fn work_area(info: &MONITORINFO) -> Result<PhysicalRect, String> {
    intersect_work_area(rect(info.rcMonitor), rect(info.rcWork)).ok_or_else(|| {
        "Windows retornou uma área útil vazia ou inválida; a janela não foi movida.".to_owned()
    })
}

fn outer_placement(hwnd: HWND) -> Result<WindowPlacement, String> {
    let mut outer = RECT::default();
    // SAFETY: hwnd is validated and outer is writable storage for this
    // synchronous call. The caller has checked per-monitor DPI awareness.
    if unsafe { GetWindowRect(hwnd, &mut outer) } == 0 {
        return Err(os_error("Não foi possível consultar o tamanho da janela"));
    }
    let width = i32::try_from(i64::from(outer.right) - i64::from(outer.left))
        .map_err(|_| "A largura da janela excede o intervalo de coordenadas.".to_owned())?;
    let height = i32::try_from(i64::from(outer.bottom) - i64::from(outer.top))
        .map_err(|_| "A altura da janela excede o intervalo de coordenadas.".to_owned())?;
    if width <= 0 || height <= 0 {
        return Err("Windows retornou um tamanho de janela inválido.".to_owned());
    }
    Ok(WindowPlacement {
        x: outer.left,
        y: outer.top,
        width,
        height,
    })
}

fn apply_placement(
    hwnd: HWND,
    current: WindowPlacement,
    placement: WindowPlacement,
) -> Result<(), String> {
    // Child geometry belongs to embed_taskbar; desktop coordinates are invalid here.
    // SAFETY: read-only style query on the validated own window.
    if unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
            hwnd,
            windows_sys::Win32::UI::WindowsAndMessaging::GWL_STYLE,
        )
    } & windows_sys::Win32::UI::WindowsAndMessaging::WS_CHILD as isize
        != 0
    {
        return Ok(());
    }
    if current == placement {
        return Ok(());
    }
    let mut flags = SWP_NOACTIVATE | SWP_NOZORDER;
    if placement.width == current.width && placement.height == current.height {
        flags |= SWP_NOSIZE;
    }
    // SAFETY: HWND is borrowed from the live Frame on its owner thread. The
    // pure geometry functions validate physical coordinates and positive size.
    // NOZORDER ignores the null insert-after handle; NOACTIVATE preserves focus.
    if unsafe {
        SetWindowPos(
            hwnd,
            ptr::null_mut(),
            placement.x,
            placement.y,
            placement.width,
            placement.height,
            flags,
        )
    } == 0
    {
        return Err(os_error("Não foi possível posicionar a janela"));
    }
    Ok(())
}

fn autohide_edge(monitor: RECT) -> DockEdge {
    // Query the specified monitor, not the primary taskbar. If multiple
    // autohide appbars exist, use the same stable edge order as pure docking.
    for (edge, native_edge) in [
        (DockEdge::Bottom, ABE_BOTTOM),
        (DockEdge::Top, ABE_TOP),
        (DockEdge::Left, ABE_LEFT),
        (DockEdge::Right, ABE_RIGHT),
    ] {
        let mut data = APPBARDATA {
            cbSize: mem::size_of::<APPBARDATA>() as u32,
            uEdge: native_edge,
            rc: monitor,
            ..Default::default()
        };
        // SAFETY: data is initialized with size, physical monitor bounds and
        // a documented edge. GETAUTOHIDEBAREX is read-only: no appbar is created,
        // registered, reserved or altered. The returned HWND is never retained.
        if unsafe { SHAppBarMessage(ABM_GETAUTOHIDEBAREX, &mut data) } != 0 {
            return edge;
        }
    }
    // Zero means absent OR unavailable. Fall back to rcWork reservations;
    // their geometry remains authoritative even when the shell is restarting.
    DockEdge::Auto
}

/// Explicitly dock beside the current monitor's taskbar/appbar without focus.
/// Preserves the existing API. There is no background polling or snapping.
pub fn dock_window(frame: &eframe::Frame) -> Result<(), String> {
    dock_window_at_edge(frame, DockEdge::Auto)
}

/// Dock at a chosen edge, keeping the whole window inside the physical work
/// area. `Auto` first checks per-monitor autohide appbars, then reserved edges.
/// Restore a minimized window before calling this function.
pub fn dock_window_at_edge(frame: &eframe::Frame, edge: DockEdge) -> Result<(), String> {
    with_hwnd(frame, |hwnd| {
        // SAFETY: this is a read-only state query for the validated live HWND.
        if unsafe { IsIconic(hwnd) } != 0 {
            return Err("Restaure a janela antes de reposicioná-la.".to_owned());
        }
        require_physical_coordinates(hwnd)?;
        let info = monitor_info(hwnd)?;
        let work = work_area(&info)?;
        let current = outer_placement(hwnd)?;
        // SAFETY: read-only DPI query for the live per-monitor-aware window.
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        if dpi == 0 {
            return Err("Windows não informou o DPI da janela.".to_owned());
        }
        let edge = if edge == DockEdge::Auto {
            autohide_edge(info.rcMonitor)
        } else {
            edge
        };
        let placement = dock_placement_at_edge(
            rect(info.rcMonitor),
            work,
            [current.width, current.height],
            dock_margin_for_dpi(dpi),
            edge,
        )
        .ok_or_else(|| "Não foi possível calcular uma posição válida na área útil.".to_owned())?;
        apply_placement(hwnd, current, placement)
    })
}

/// After a display/work-area change, move only what is needed to make the
/// window fit again. Already valid user positions remain unchanged. Minimized
/// windows are skipped: their iconic rectangles are not normal window bounds.
/// Call on relevant events, not continuously from every animation frame.
pub fn recover_window_placement(frame: &eframe::Frame) -> Result<bool, String> {
    with_hwnd(frame, |hwnd| {
        // SAFETY: read-only query for the validated window on its UI thread.
        if unsafe { IsIconic(hwnd) } != 0 {
            return Ok(false);
        }
        require_physical_coordinates(hwnd)?;
        let info = monitor_info(hwnd)?;
        let current = outer_placement(hwnd)?;
        let placement = fit_placement_to_work_area(work_area(&info)?, current)
            .ok_or_else(|| "Não foi possível recuperar uma posição válida da janela.".to_owned())?;
        apply_placement(hwnd, current, placement)?;
        Ok(current != placement)
    })
}

fn native_state(hwnd: HWND) -> DesktopState {
    // SAFETY: both queries inspect the validated live HWND and retain nothing.
    let (visible, minimized) = unsafe { (IsWindowVisible(hwnd) != 0, IsIconic(hwnd) != 0) };
    if !visible {
        DesktopState::HiddenByUser
    } else if minimized {
        DesktopState::Minimized
    } else {
        DesktopState::Visible
    }
}

/// Native observation; HiddenByUser denotes nonvisibility, not proof of who
/// hid it. The controller owns intent and distinguishes initial hidden startup.
pub(super) fn window_state(frame: &eframe::Frame) -> Result<DesktopState, String> {
    with_hwnd(frame, |hwnd| Ok(native_state(hwnd)))
}

/// Restore first without activation, then synchronize winit's cached flags.
/// Requires the baseline's `with_active(false)` and the owning eframe thread.
pub(super) fn show_window(frame: &eframe::Frame) -> Result<(), String> {
    let window = frame
        .winit_window()
        .ok_or_else(|| "A janela eframe ainda não está disponível.".to_owned())?;
    with_hwnd(frame, |hwnd| {
        // SAFETY: live HWND on its owner thread. SHOWNOACTIVATE requests normal
        // size/position without taking focus; no handle ownership is transferred.
        // ShowWindow's return value is previous visibility, not a success flag.
        unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
        if native_state(hwnd) != DesktopState::Visible {
            return Err("Windows não conseguiu mostrar/restaurar a janela.".to_owned());
        }
        // In locked winit 0.30.13, this first reads IsIconic and syncs MINIMIZED
        // in place. Confirmed native restoration makes the diff empty, avoiding
        // SW_RESTORE (which would activate). Sync VISIBLE after MINIMIZED.
        window.set_minimized(false);
        window.set_visible(true);
        if native_state(hwnd) != DesktopState::Visible {
            return Err("A janela não permaneceu visível após a sincronização.".to_owned());
        }
        Ok(())
    })
}

pub(super) fn hide_window(frame: &eframe::Frame) -> Result<(), String> {
    let window = frame
        .winit_window()
        .ok_or_else(|| "A janela eframe ainda não está disponível.".to_owned())?;
    with_hwnd(frame, |hwnd| {
        // Use the safe owner API so later viewport style changes cannot undo a
        // raw SW_HIDE because of a stale VISIBLE flag.
        window.set_visible(false);
        if native_state(hwnd) != DesktopState::HiddenByUser {
            return Err("Windows não conseguiu ocultar a janela.".to_owned());
        }
        Ok(())
    })
}

pub(super) fn minimize_window(frame: &eframe::Frame) -> Result<(), String> {
    let window = frame
        .winit_window()
        .ok_or_else(|| "A janela eframe ainda não está disponível.".to_owned())?;
    with_hwnd(frame, |hwnd| {
        if native_state(hwnd) == DesktopState::HiddenByUser {
            return Err("Mostre a janela antes de minimizá-la.".to_owned());
        }
        // SAFETY: live HWND on its UI thread. SHOWMINNOACTIVE minimizes without
        // activating this window. Its return value describes previous visibility.
        unsafe { ShowWindow(hwnd, SW_SHOWMINNOACTIVE) };
        if native_state(hwnd) != DesktopState::Minimized {
            return Err("Windows não conseguiu minimizar a janela.".to_owned());
        }
        // Native IsIconic is now true, making winit's MINIMIZED diff a no-op.
        // Do not call set_visible(true) afterward: it could restore the window.
        window.set_minimized(true);
        if native_state(hwnd) != DesktopState::Minimized {
            return Err("A janela não permaneceu minimizada após a sincronização.".to_owned());
        }
        Ok(())
    })
}

/// A native error remains visible even for the release's GUI subsystem binary.
pub fn show_error(message: &str) {
    let wide_message: Vec<u16> = message
        .replace('\0', "\u{fffd}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let caption: Vec<u16> = "CPU Express".encode_utf16().chain(Some(0)).collect();
    // SAFETY: UTF-16 buffers are NUL-terminated and live for this synchronous
    // call. A null owner is supported, including before window creation fails.
    let shown = unsafe {
        MessageBoxW(
            ptr::null_mut(),
            wide_message.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        )
    };
    if shown == 0 {
        eprintln!("CPU Express: {message}");
    }
}

/// Read-only monitor lookup for a panel that does not yet have an HWND.
pub(super) fn clamp_panel_rect(saved: [f32; 4], scale: f32) -> [f32; 4] {
    let Some(placement) = super::geometry::panel_physical(saved, scale) else {
        return saved;
    };
    let Some(bounds) = placement.checked_bounds() else {
        return saved;
    };
    let bounds = RECT {
        left: bounds.left,
        top: bounds.top,
        right: bounds.right,
        bottom: bounds.bottom,
    };
    let mut info = MONITORINFO {
        cbSize: mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: read-only queries, initialized outputs; no retained monitor handle.
    unsafe {
        let context = GetThreadDpiAwarenessContext();
        if context.is_null()
            || GetAwarenessFromDpiAwarenessContext(context) != DPI_AWARENESS_PER_MONITOR_AWARE
        {
            return saved;
        }
        let monitor =
            windows_sys::Win32::Graphics::Gdi::MonitorFromRect(&bounds, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() || GetMonitorInfoW(monitor, &mut info) == 0 {
            return saved;
        }
    }
    match work_area(&info) {
        Ok(work) => super::geometry::fit_panel_rect(saved, scale, work),
        Err(_) => saved,
    }
}
