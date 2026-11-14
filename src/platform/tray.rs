//! A Windows notification icon owned by a dedicated, event-driven thread.
//!
//! This module never stores or manipulates the eframe window. Its helper window
//! exists only to receive shell messages; the UI consumes [`TrayEvent`] values.
//! The wake callback must be short and thread-safe (`egui::Context::request_repaint`
//! is suitable). No global hooks, subclassing, appbar, or continuous polling is used. Shell recovery uses a bounded retry timer.

use std::{
    cell::Cell,
    io, mem,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use windows_sys::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, SetLastError, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Shell::{
            NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETFOCUS,
            NIM_SETVERSION, NIN_SELECT, NINF_KEY, NOTIFYICON_VERSION_4, NOTIFYICONDATAW,
            Shell_NotifyIconW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CREATESTRUCTW, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
            DestroyMenu, DestroyWindow, DispatchMessageW, EndMenu, GWLP_USERDATA, GetCursorPos,
            GetMessageW, GetWindowLongPtrW, HICON, HMENU, IDI_APPLICATION, LoadIconW, MF_SEPARATOR,
            MF_STRING, MSG, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
            SPI_SETWORKAREA, SendNotifyMessageW, SetForegroundWindow, SetWindowLongPtrW,
            TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage, UnregisterClassW,
            WM_APP, WM_CLOSE, WM_CONTEXTMENU, WM_DESTROY, WM_DISPLAYCHANGE, WM_ENDSESSION,
            WM_NCCREATE, WM_NCDESTROY, WM_NULL, WM_QUERYENDSESSION, WM_SETTINGCHANGE, WNDCLASSW,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
};

const ICON_ID: u32 = 1;
const ICON_MESSAGE: u32 = WM_APP + 41;
const STOP_MESSAGE: u32 = WM_APP + 42;
const RECOVERY_TIMER: usize = 1;
const RECOVERY_ATTEMPTS: u8 = 5;
const KEY_SELECT: u32 = NIN_SELECT | NINF_KEY;
static NEXT_CLASS: AtomicU64 = AtomicU64::new(1);

/// Intent received from the notification icon, not an action already applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrayEvent {
    Show,
    Hide,
    Dock,
    ToggleTopmost,
    Exit,
    /// Broadcast display/work-area change; the UI decides whether moving is needed.
    DisplayChanged,
    /// Restore a hidden main window and disable hiding until a tray is available.
    Unavailable(String),
}

/// Owns one notification icon and its helper thread. Dropping it removes both.
///
/// Creation waits for an initial shell acknowledgement. Failure leaves the main
/// window untouched. Successful creation is not a guarantee that Explorer will
/// remain alive; applications must process `Unavailable` and check availability
/// before every hide operation. `TaskbarCreated` starts bounded re-registration.
pub struct SystemTray {
    events: Receiver<TrayEvent>,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl SystemTray {
    /// Create the icon, then return when it is ready to receive user input.
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Result<Self, String> {
        let (events_tx, events) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let shared = Arc::new(Shared {
            events: events_tx,
            wake: Arc::new(wake),
            available: AtomicBool::new(false),
            stopping: AtomicBool::new(false),
            endpoint: Mutex::new(None),
        });
        let worker_shared = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("cpu-express-tray".to_owned())
            .spawn(move || {
                let run = catch_unwind(AssertUnwindSafe(|| {
                    run_tray(Arc::clone(&worker_shared), ready_tx);
                }));
                if run.is_err() {
                    worker_shared.fail("A thread da bandeja foi interrompida.".to_owned());
                }
                worker_shared.available.store(false, Ordering::Release);
            })
            .map_err(|error| format!("Não foi possível iniciar a bandeja: {error}"))?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                events,
                shared,
                thread: Some(thread),
            }),
            result => {
                // Startup failure always drops the native resources before joining.
                shared.stopping.store(true, Ordering::Release);
                let _ = thread.join();
                match result {
                    Ok(Err(error)) => Err(error),
                    _ => Err("A bandeja terminou antes de concluir a inicialização.".to_owned()),
                }
            }
        }
    }

    /// Nonblocking: consume at most one event. Drain this from `App::logic`.
    pub fn poll(&self) -> Option<TrayEvent> {
        self.events.try_recv().ok()
    }

    /// Last acknowledged shell state. This does not synchronously probe Explorer.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.shared.available.load(Ordering::Acquire)
            && !self.shared.stopping.load(Ordering::Acquire)
    }

    /// Remove the icon, close the owned helper window, and join its thread.
    /// Calling this more than once is harmless. The main window is never closed.
    pub fn shutdown(&mut self) -> Result<(), String> {
        let Some(thread) = self.thread.take() else {
            return Ok(());
        };
        if thread.thread().id() == thread::current().id() {
            self.thread = Some(thread);
            return Err(
                "Encerre a bandeja pela thread da aplicação, fora do callback de wake.".to_owned(),
            );
        }
        self.shared.available.store(false, Ordering::Release);
        self.shared.stopping.store(true, Ordering::Release);
        let mut signal_error = None;
        {
            // Destruction clears this slot under the same lock *before* the HWND
            // can be released. We therefore never post to a recycled window ID.
            let endpoint = self.shared.endpoint();
            if let Some(hwnd) = *endpoint {
                // SAFETY: the endpoint lock keeps this owned HWND alive through
                // the asynchronous post. The message carries no borrowed pointers.
                let posted = unsafe { PostMessageW(hwnd as HWND, STOP_MESSAGE, 0, 0) };
                if posted == 0 {
                    let post_error = io::Error::last_os_error();
                    // SAFETY: the endpoint lock still keeps the owned HWND alive.
                    // The thread-id check above guarantees a different recipient
                    // thread, so SendNotifyMessageW returns asynchronously without
                    // waiting on this lock. Its nonqueued delivery reaches WNDPROC
                    // even inside TrackPopupMenu when the posted-message queue is
                    // full. No pointer or borrowed data crosses this boundary.
                    if unsafe { SendNotifyMessageW(hwnd as HWND, STOP_MESSAGE, 0, 0) } == 0 {
                        signal_error = Some(format!(
                            "Não foi possível sinalizar o encerramento da bandeja: PostMessageW: {post_error}; SendNotifyMessageW: {}. Tente novamente.",
                            io::Error::last_os_error()
                        ));
                    }
                }
            }
        }
        if let Some(error) = signal_error {
            // Do not join a possibly modal worker without an accepted stop signal.
            // Keep its JoinHandle so the UI can report the failure and retry. Drop
            // makes one final attempt; the worker still owns all its native state.
            self.thread = Some(thread);
            return Err(error);
        }
        thread
            .join()
            .map_err(|_| "Não foi possível concluir a thread da bandeja.".to_owned())
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

struct Shared {
    events: Sender<TrayEvent>,
    wake: Arc<dyn Fn() + Send + Sync>,
    available: AtomicBool,
    stopping: AtomicBool,
    // A scalar token is sent across threads, never a Rust pointer to native data.
    // Holding the mutex prevents destruction, not dispatch or other UI activity.
    endpoint: Mutex<Option<usize>>,
}

impl Shared {
    fn endpoint(&self) -> MutexGuard<'_, Option<usize>> {
        match self.endpoint.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn emit(&self, event: TrayEvent) {
        if self.events.send(event).is_ok() {
            // User callbacks must never unwind across Win32's WNDPROC boundary.
            if catch_unwind(AssertUnwindSafe(|| (self.wake)())).is_err() {
                self.available.store(false, Ordering::Release);
            }
        }
    }

    fn fail(&self, message: String) {
        self.available.store(false, Ordering::Release);
        self.emit(TrayEvent::Unavailable(message));
    }
}

struct TrayState {
    shared: Arc<Shared>,
    hwnd: Cell<HWND>,
    icon: HICON,
    icon_added: Cell<bool>,
    menu_open: Cell<bool>,
    taskbar_created: u32,
    retries_left: Cell<u8>,
}

impl TrayState {
    fn icon_data(&self, hwnd: HWND) -> NOTIFYICONDATAW {
        let mut data = NOTIFYICONDATAW {
            cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: ICON_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: ICON_MESSAGE,
            hIcon: self.icon,
            ..Default::default()
        };
        for (dst, source) in data
            .szTip
            .iter_mut()
            .take(127)
            .zip("CPU Express — clique para mostrar".encode_utf16())
        {
            *dst = source;
        }
        data
    }

    fn add_icon(&self, hwnd: HWND) -> Result<(), String> {
        let mut data = self.icon_data(hwnd);
        // SAFETY: data has the documented size, an owned live helper HWND, a
        // shared system HICON, and terminated UTF-16 buffers. The call is sync.
        if unsafe { Shell_NotifyIconW(NIM_ADD, &data) } == 0 {
            return Err(
                "O Windows não aceitou o ícone da bandeja; a janela continuará disponível."
                    .to_owned(),
            );
        }
        self.icon_added.set(true);
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        // SAFETY: the same initialized icon data identifies the registered icon;
        // writing uVersion selects the documented version-4 callback layout.
        if unsafe { Shell_NotifyIconW(NIM_SETVERSION, &data) } == 0 {
            self.remove_icon(hwnd);
            return Err("O Windows não aceitou o protocolo da bandeja.".to_owned());
        }
        self.shared.available.store(true, Ordering::Release);
        Ok(())
    }

    // One immediate attempt and at most four one-second retries. No sleeps on
    // either UI thread, and no timer remains after success or exhaustion.
    fn recover_icon(&self, hwnd: HWND) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};
        // SAFETY: timer belongs to our live helper window on this thread.
        unsafe { KillTimer(hwnd, RECOVERY_TIMER) };
        let remaining = self.retries_left.get();
        if remaining == 0 {
            return;
        }
        self.retries_left.set(remaining - 1);
        if self.add_icon(hwnd).is_ok() {
            self.retries_left.set(0);
            self.shared.emit(TrayEvent::DisplayChanged);
        } else if remaining > 1 {
            // SAFETY: window timer with no callback, handled by this WNDPROC.
            if unsafe { SetTimer(hwnd, RECOVERY_TIMER, 1000, None) } == 0 {
                self.retries_left.set(0);
                self.shared.fail(
                    "Tray recovery timer unavailable; show the window to recover.".to_owned(),
                );
            }
        } else {
            self.shared
                .fail("Tray recovery exhausted; the window remains available.".to_owned());
        }
    }

    fn remove_icon(&self, hwnd: HWND) {
        self.shared.available.store(false, Ordering::Release);
        if self.icon_added.replace(false) {
            let data = self.icon_data(hwnd);
            // SAFETY: data identifies only our icon. NIM_DELETE is safe even when
            // Explorer has restarted and no longer remembers that icon.
            unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
        }
    }

    fn close_window(&self) {
        *self.shared.endpoint() = None;
        let hwnd = self.hwnd.get();
        if !hwnd.is_null() {
            self.remove_icon(hwnd);
            if self.menu_open.get() {
                // SAFETY: this runs only on the thread that entered TrackPopupMenu.
                unsafe { EndMenu() };
            }
            // SAFETY: hwnd is a helper owned by this thread, never the eframe HWND.
            // State remains alive until WM_NCDESTROY clears its stored pointer.
            unsafe { DestroyWindow(hwnd) };
        }
    }

    fn context_menu(&self, hwnd: HWND, anchor: WPARAM) {
        if self.menu_open.replace(true) {
            return;
        }
        let result = show_context_menu(self, hwnd, anchor);
        self.menu_open.set(false);
        if let Err(error) = result {
            self.shared.fail(error);
        }
    }
}

struct RegisteredClass {
    name: Vec<u16>,
    instance: HINSTANCE,
}

impl Drop for RegisteredClass {
    fn drop(&mut self) {
        // SAFETY: this class was registered by this thread with this instance.
        // The helper window is destroyed before the guard is dropped. The shared
        // LoadIconW icon is system-owned and must not be passed to DestroyIcon.
        unsafe { UnregisterClassW(self.name.as_ptr(), self.instance) };
    }
}

struct TrayRuntime {
    // Option permits a conservative leak if Win32 refuses to detach user data,
    // avoiding a dangling pointer on an exceptional destruction failure.
    state: Option<Box<TrayState>>,
    _class: RegisteredClass,
}

impl TrayRuntime {
    fn create(shared: Arc<Shared>) -> Result<Self, String> {
        // SAFETY: null requests the module of the current executable; no owned
        // reference is acquired, so FreeLibrary must not be called on the result.
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        if instance.is_null() {
            return Err(os_error("Não foi possível identificar o executável"));
        }
        // SAFETY: IDI_APPLICATION is a valid system resource, null requests the
        // system module. LoadIconW returns a shared icon with process lifetime.
        let icon = unsafe { LoadIconW(ptr::null_mut(), IDI_APPLICATION) };
        if icon.is_null() {
            return Err(os_error("Não foi possível carregar o ícone da bandeja"));
        }
        let broadcast = wide("TaskbarCreated");
        // SAFETY: broadcast is NUL-terminated and lives for the synchronous call.
        let taskbar_created = unsafe { RegisterWindowMessageW(broadcast.as_ptr()) };
        if taskbar_created == 0 {
            return Err(os_error("Não foi possível registrar o aviso do Explorer"));
        }
        let class_name = wide(&format!(
            "CPUExpress.Tray.{}.{}",
            std::process::id(),
            NEXT_CLASS.fetch_add(1, Ordering::Relaxed)
        ));
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        // SAFETY: class is initialized; its callback has the Win32 ABI and the
        // UTF-16 class name lives until unregistering. No global class is used.
        if unsafe { RegisterClassW(&class) } == 0 {
            return Err(os_error("Não foi possível registrar a janela da bandeja"));
        }
        let runtime = Self {
            state: Some(Box::new(TrayState {
                shared,
                hwnd: Cell::new(ptr::null_mut()),
                icon,
                icon_added: Cell::new(false),
                retries_left: Cell::new(0),
                menu_open: Cell::new(false),
                taskbar_created,
            })),
            _class: RegisteredClass {
                name: class_name,
                instance,
            },
        };
        let Some(state) = runtime.state.as_deref() else {
            return Err("Estado da bandeja indisponível.".to_owned());
        };
        // SAFETY: the boxed State has a stable address, remains owned by Runtime,
        // and is only used by this same thread. The top-level window is invisible
        // (no WS_VISIBLE), excluded from task switching, and never activated.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                runtime._class.name.as_ptr(),
                ptr::null(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                (state as *const TrayState).cast(),
            )
        };
        if hwnd.is_null() {
            return Err(os_error("Não foi possível criar a janela da bandeja"));
        }
        state.add_icon(hwnd)?;
        *state.shared.endpoint() = Some(hwnd as usize);
        Ok(runtime)
    }
}

impl Drop for TrayRuntime {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            state.close_window();
            let hwnd = state.hwnd.get();
            if !hwnd.is_null() {
                // SAFETY: destruction failed for our still-owned helper. Clear
                // GWLP_USERDATA before freeing Rust state; no other Rust pointer
                // is ever stored in that slot by this class.
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
                // SAFETY: same live owned helper. Reading confirms that future
                // native callbacks cannot access state after this scope exits.
                if unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } != 0 {
                    eprintln!("CPU Express: Windows recusou liberar a janela auxiliar da bandeja.");
                    mem::forget(state);
                }
            }
        }
    }
}

fn run_tray(shared: Arc<Shared>, ready: Sender<Result<(), String>>) {
    let runtime = match TrayRuntime::create(Arc::clone(&shared)) {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(())).is_err() {
        return;
    }
    let mut message = MSG::default();
    while !shared.stopping.load(Ordering::Acquire) {
        // SAFETY: message is initialized writable storage. A null HWND receives
        // all messages on this dedicated thread, including WM_QUIT.
        let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
        if result <= 0 {
            if !shared.stopping.load(Ordering::Acquire) {
                shared.fail(if result == -1 {
                    os_error("O loop da bandeja falhou")
                } else {
                    "A janela auxiliar da bandeja foi encerrada.".to_owned()
                });
            }
            break;
        }
        if shared.stopping.load(Ordering::Acquire) {
            break;
        }
        // SAFETY: message was produced by GetMessageW on this thread. Only this
        // thread dispatches its helper window, so boxed state is never shared.
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    drop(runtime);
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let handled = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: only the registered class uses this callback. Win32 supplies
        // the message-specific pointer lifetimes documented below.
        unsafe { handle_message(hwnd, message, wparam, lparam) }
    }));
    match handled {
        Ok(result) => result,
        Err(_) => {
            // SAFETY: we are executing on the helper's own message thread. Posting
            // quit makes the runtime drop its resources after dispatch returns.
            unsafe { PostQuitMessage(1) };
            0
        }
    }
}

unsafe fn handle_message(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCCREATE {
        if lparam == 0 {
            return 0;
        }
        // SAFETY: WM_NCCREATE's lParam points to CREATESTRUCTW for this call.
        let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
        let state_ptr = create.lpCreateParams.cast::<TrayState>();
        if state_ptr.is_null() {
            return 0;
        }
        // SAFETY: CreateWindowExW supplied our live boxed State as lpParam.
        // This thread is its sole accessor; the Box outlives DestroyWindow.
        let state = unsafe { &*state_ptr };
        // SAFETY: GWLP_USERDATA belongs to this class. Resetting last error is
        // required because successful replacement of zero also returns zero.
        unsafe {
            SetLastError(0);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
        }
        // SAFETY: no intervening API has replaced this thread's last-error value.
        if unsafe { GetLastError() } != 0 {
            return 0;
        }
        state.hwnd.set(hwnd);
        return 1;
    }
    // SAFETY: all windows of this private class use the state installed above.
    // The pointer is cleared at WM_NCDESTROY before its backing Box is released.
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const TrayState;
    if state_ptr.is_null() {
        // SAFETY: default handling is valid when creation has not installed state
        // or after destruction has removed it; parameters came from Win32.
        return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
    }
    // SAFETY: the Box is alive for the entire native window lifetime. Shared
    // references plus Cell support reentrant Win32 callbacks on this one thread.
    let state = unsafe { &*state_ptr };

    if message == WM_NCDESTROY {
        *state.shared.endpoint() = None;
        state.shared.available.store(false, Ordering::Release);
        state.hwnd.set(ptr::null_mut());
        // SAFETY: clear the private pointer while State is still alive, then allow
        // the OS to finish its normal nonclient destruction.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
    }
    if message == WM_CLOSE && !state.shared.stopping.load(Ordering::Acquire) {
        // This is an unexpected external close, not our private shutdown signal.
        // Notify and wake the UI before suppressing events so a hidden main window
        // can be restored instead of silently losing its recovery path.
        state.shared.fail(
            "A janela auxiliar da bandeja recebeu uma solicitação de fechamento inesperada."
                .to_owned(),
        );
    }
    if message == STOP_MESSAGE || message == WM_CLOSE {
        state.shared.stopping.store(true, Ordering::Release);
        state.close_window();
        // SAFETY: this quits only the dedicated helper thread, never eframe's loop.
        unsafe { PostQuitMessage(0) };
        return 0;
    }
    if message == WM_DESTROY {
        // SAFETY: the helper is being destroyed on its owning thread.
        unsafe { PostQuitMessage(0) };
        return 0;
    }
    if message == WM_QUERYENDSESSION {
        return 1;
    }
    if message == WM_ENDSESSION && wparam != 0 {
        state.shared.stopping.store(true, Ordering::Release);
        state.close_window();
        return 0;
    }
    if !state.shared.stopping.load(Ordering::Acquire) {
        if message == state.taskbar_created {
            // The shell may broadcast before its notification area is ready.
            state.remove_icon(hwnd);
            state
                .shared
                .fail("Explorer restarted; recovering the notification icon.".to_owned());
            state.retries_left.set(RECOVERY_ATTEMPTS);
            state.recover_icon(hwnd);
            return 0;
        }
        if message == windows_sys::Win32::UI::WindowsAndMessaging::WM_TIMER
            && wparam == RECOVERY_TIMER
        {
            state.recover_icon(hwnd);
            return 0;
        }
        if message == WM_DISPLAYCHANGE
            || (message == WM_SETTINGCHANGE && wparam == SPI_SETWORKAREA as usize)
        {
            state.shared.emit(TrayEvent::DisplayChanged);
            return 0;
        }
        if message == ICON_MESSAGE {
            if let Some(event) = decode_callback(lparam) {
                match event {
                    NIN_SELECT | KEY_SELECT => state.shared.emit(TrayEvent::Show),
                    WM_CONTEXTMENU => state.context_menu(hwnd, wparam),
                    _ => {}
                }
            }
            return 0;
        }
    }
    // SAFETY: messages not consumed by this class retain their native parameters.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

fn decode_callback(lparam: LPARAM) -> Option<u32> {
    let packed = lparam as u32;
    ((packed >> 16) == ICON_ID).then_some(packed & 0xffff)
}

fn menu_anchor(packed: WPARAM) -> POINT {
    POINT {
        x: (packed as u16 as i16) as i32,
        y: ((packed >> 16) as u16 as i16) as i32,
    }
}

struct PopupMenu(HMENU);

impl Drop for PopupMenu {
    fn drop(&mut self) {
        // SAFETY: this owns a non-null popup created on this thread. TrackPopupMenu
        // has returned before the guard is dropped; no submenu handles are shared.
        unsafe { DestroyMenu(self.0) };
    }
}

fn show_context_menu(state: &TrayState, hwnd: HWND, anchor: WPARAM) -> Result<(), String> {
    // SAFETY: CreatePopupMenu has no pointer inputs and returns an owned handle.
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return Err(os_error("Não foi possível abrir o menu da bandeja"));
    }
    let menu = PopupMenu(menu);
    for (id, label) in [
        (1, "Mostrar trem"),
        (2, "Ocultar trem"),
        (3, "Reposicionar na área útil"),
        (4, "Alternar sempre no topo"),
        (5, "Sair do CPU Express"),
    ] {
        let label = wide(label);
        if id == 5 {
            // SAFETY: menu is owned and valid. A separator has no label or command.
            if unsafe { AppendMenuW(menu.0, MF_SEPARATOR, 0, ptr::null()) } == 0 {
                return Err(os_error("Não foi possível preparar o menu da bandeja"));
            }
        }
        // SAFETY: menu is owned and label is NUL-terminated; AppendMenuW copies
        // this string during the synchronous call. IDs are local numeric commands.
        if unsafe { AppendMenuW(menu.0, MF_STRING, id, label.as_ptr()) } == 0 {
            return Err(os_error("Não foi possível preparar o menu da bandeja"));
        }
    }
    let mut point = menu_anchor(anchor);
    if point.x == -1 && point.y == -1 {
        // SAFETY: point is initialized writable storage. This fallback is used
        // only for an explicit keyboard menu request without an anchor position.
        if unsafe { GetCursorPos(&mut point) } == 0 {
            return Err(os_error("Não foi possível localizar o menu da bandeja"));
        }
    }
    // SAFETY: this is our helper HWND on its owner thread. Foreground is requested
    // ONLY after an explicit tray context-menu action, as required by the Win32
    // menu dismissal protocol. Startup, docking and topmost never call this API.
    unsafe { SetForegroundWindow(hwnd) };
    // SAFETY: owned menu and helper remain alive throughout this modal call.
    // RETURNCMD returns an ID instead of sending application commands elsewhere.
    let selected = unsafe {
        TrackPopupMenu(
            menu.0,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            ptr::null(),
        )
    };
    if !state.hwnd.get().is_null() {
        // SAFETY: still our live helper. A harmless asynchronous message makes
        // repeated tray menus dismiss correctly according to Microsoft's recipe.
        unsafe { PostMessageW(hwnd, WM_NULL, 0, 0) };
        if selected == 0 && state.icon_added.get() {
            let data = state.icon_data(hwnd);
            // SAFETY: initialized data identifies the registered icon. This gives
            // keyboard focus back to the notification area after menu cancellation.
            unsafe { Shell_NotifyIconW(NIM_SETFOCUS, &data) };
        }
    }
    if !state.shared.stopping.load(Ordering::Acquire) {
        let event = match selected {
            1 => Some(TrayEvent::Show),
            2 => Some(TrayEvent::Hide),
            3 => Some(TrayEvent::Dock),
            4 => Some(TrayEvent::ToggleTopmost),
            5 => Some(TrayEvent::Exit),
            _ => None,
        };
        if let Some(event) = event {
            state.shared.emit(event);
        }
    }
    Ok(())
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn os_error(operation: &str) -> String {
    format!("{operation}: {}", io::Error::last_os_error())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_four_callbacks_decode_icon_and_event_separately() {
        assert_eq!(
            decode_callback(((ICON_ID << 16) | NIN_SELECT) as LPARAM),
            Some(NIN_SELECT)
        );
        assert_eq!(
            decode_callback(((ICON_ID << 16) | KEY_SELECT) as LPARAM),
            Some(KEY_SELECT)
        );
        assert_eq!(
            decode_callback(((2 << 16) | WM_CONTEXTMENU) as LPARAM),
            None
        );
    }

    #[test]
    fn tray_menu_coordinates_preserve_negative_monitor_origins() {
        let x = -1820_i16;
        let y = -100_i16;
        let point = menu_anchor(((u32::from(y as u16) << 16) | u32::from(x as u16)) as WPARAM);
        assert_eq!((point.x, point.y), (-1820, -100));
    }

    #[test]
    fn keyboard_menu_sentinel_remains_signed() {
        let point = menu_anchor(u32::MAX as WPARAM);
        assert_eq!((point.x, point.y), (-1, -1));
    }
}
