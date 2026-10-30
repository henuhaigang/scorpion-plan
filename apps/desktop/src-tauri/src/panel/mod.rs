#[cfg(target_os = "macos")]
use std::sync::atomic::AtomicI32;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};
#[cfg(not(target_os = "macos"))]
use tauri::{Position, Size, WebviewWindow};

const PANEL_SHOW_GUARD: Duration = Duration::from_millis(150);
const BLUR_CHECK_INTERVAL: Duration = Duration::from_millis(40);

#[derive(Debug, Default)]
pub struct PanelController {
    visible: AtomicBool,
    tray_hover: AtomicBool,
    hide_guard_until_ms: AtomicU64,
    #[cfg(target_os = "macos")]
    previous_app_pid: AtomicI32,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl PanelController {
    pub fn begin_show(&self) {
        self.visible.store(true, Ordering::SeqCst);
        self.hide_guard_until_ms
            .store(
                now_ms().saturating_add(PANEL_SHOW_GUARD.as_millis() as u64),
                Ordering::SeqCst,
            );
    }

    pub fn begin_hide(&self) {
        self.visible.store(false, Ordering::SeqCst);
        self.hide_guard_until_ms.store(0, Ordering::SeqCst);
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::SeqCst)
    }

    pub fn focus_gained(&self) {
        self.hide_guard_until_ms.store(0, Ordering::SeqCst);
    }

    pub fn begin_tray_hover(&self) {
        self.tray_hover.store(true, Ordering::SeqCst);
    }

    pub fn end_tray_hover(&self) {
        self.tray_hover.store(false, Ordering::SeqCst);
    }

    pub fn should_hide_on_blur(&self) -> bool {
        should_hide_on_blur_state(
            self.is_visible(),
            self.tray_hover.load(Ordering::SeqCst),
            now_ms(),
            self.hide_guard_until_ms.load(Ordering::SeqCst),
        )
    }
}

fn should_hide_on_blur_state(
    visible: bool,
    tray_hover: bool,
    now: u64,
    hide_guard_until_ms: u64,
) -> bool {
    visible && !tray_hover && now >= hide_guard_until_ms
}

fn blur_check_delay_ms(now: u64, hide_guard_until_ms: u64) -> u64 {
    hide_guard_until_ms
        .saturating_sub(now)
        .max(BLUR_CHECK_INTERVAL.as_millis() as u64)
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is missing".to_string())?;
    let controller = app.state::<PanelController>();

    if controller.is_visible() && window.is_visible().unwrap_or(false) {
        controller.begin_show();
        #[cfg(target_os = "macos")]
        return show_macos_overlay(window);
        #[cfg(not(target_os = "macos"))]
        return window.set_focus().map_err(|e| e.to_string());
    }

    window.unminimize().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        remember_frontmost_application(&controller);
        center_on_current_monitor(app, &window)?;
    }
    #[cfg(not(target_os = "macos"))]
    if let Err(position_error) = position_near_tray(app, &window) {
        window.center().map_err(|center_error| {
            format!("could not position panel ({position_error}) or center it ({center_error})")
        })?;
    }
    controller.begin_show();

    #[cfg(target_os = "macos")]
    {
        if let Err(error) = show_macos_overlay(window) {
            controller.begin_hide();
            return Err(error);
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Err(e) = window.show() {
            controller.begin_hide();
            return Err(e.to_string());
        }

        if let Err(e) = window.set_focus() {
            let _ = window.hide();
            controller.begin_hide();
            return Err(e.to_string());
        }
        Ok(())
    }
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    let controller = app.state::<PanelController>();
    #[cfg(target_os = "macos")]
    let restore_previous_app = frontmost_application_pid() == Some(std::process::id() as i32);

    controller.begin_hide();
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.hide() {
            controller.begin_show();
            return Err(e.to_string());
        }
    }
    #[cfg(target_os = "macos")]
    if restore_previous_app {
        let previous_pid = controller.previous_app_pid.swap(0, Ordering::SeqCst);
        activate_application(previous_pid);
    }
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<(), String> {
    if app.state::<PanelController>().is_visible() {
        hide(app)
    } else {
        show(app)
    }
}

pub fn quit(app: &AppHandle) {
    app.exit(0);
}

pub fn maybe_hide_if_unfocused(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_focused().unwrap_or(false) {
        return;
    }

    let controller = app.state::<PanelController>();
    if !controller.should_hide_on_blur() {
        return;
    }

    let _ = hide(app);
}

fn hide_after_blur_event(app: &AppHandle) {
    let controller = app.state::<PanelController>();
    if !controller.is_visible() {
        return;
    }
    if controller.tray_hover.load(Ordering::SeqCst) {
        return;
    }

    let delay = Duration::from_millis(blur_check_delay_ms(
        now_ms(),
        controller.hide_guard_until_ms.load(Ordering::SeqCst),
    ));
    let handle = app.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("dayflow-panel-blur-check".into())
        .spawn(move || {
            std::thread::sleep(delay);
            maybe_hide_if_unfocused(&handle);
        })
    {
        eprintln!("failed to schedule panel blur check: {error}");
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "default window icon is missing".to_string())?;

    #[cfg(target_os = "macos")]
    let toggle_label = "显示/隐藏面板 (⌘⇧Space)";
    #[cfg(not(target_os = "macos"))]
    let toggle_label = "显示/隐藏面板";
    let toggle_item = MenuItem::with_id(app, "toggle", toggle_label, true, None::<String>)
        .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, "quit", "退出 DayFlow", true, None::<String>)
        .map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&toggle_item, &quit_item]).map_err(|e| e.to_string())?;

    let handle = app.clone();
    TrayIconBuilder::with_id("dayflow")
        .icon(icon)
        .tooltip("DayFlow")
        .show_menu_on_left_click(false)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => {
                let _ = toggle(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| match event {
            TrayIconEvent::Enter { .. } => {
                handle.state::<PanelController>().begin_tray_hover();
            }
            TrayIconEvent::Leave { .. } => {
                handle.state::<PanelController>().end_tray_hover();
                maybe_hide_if_unfocused(&handle);
            }
            TrayIconEvent::Click {
                button,
                button_state,
                ..
            } if button == MouseButton::Left && button_state == MouseButtonState::Up => {
                let _ = toggle(&handle);
            }
            _ => {}
        })
        .build(app)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn setup_window_events(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is missing".to_string())?;
    let handle = app.clone();

    window.on_window_event(move |event| match event {
        WindowEvent::Focused(true) => {
            handle.state::<PanelController>().focus_gained();
        }
        WindowEvent::Focused(false) => hide_after_blur_event(&handle),
        WindowEvent::CloseRequested { api, .. } => {
            let _ = hide(&handle);
            api.prevent_close();
        }
        _ => {}
    });

    Ok(())
}

#[cfg(target_os = "macos")]
fn center_on_current_monitor(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let monitor = match window.current_monitor().map_err(|e| e.to_string())? {
        Some(monitor) => monitor,
        None => app
            .primary_monitor()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "no display monitor is available".to_string())?,
    };
    let position = centered_panel_position(
        *monitor.position(),
        *monitor.size(),
        window.outer_size().map_err(|e| e.to_string())?,
    )?;

    window.set_position(position).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn centered_panel_position(
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    window_size: PhysicalSize<u32>,
) -> Result<PhysicalPosition<i32>, String> {
    let centered_coordinate = |origin: i32, monitor_extent: u32, window_extent: u32| {
        let coordinate = i64::from(origin)
            + (i64::from(monitor_extent) - i64::from(window_extent)) / 2;
        i32::try_from(coordinate)
            .map_err(|_| "centered panel position is outside the supported range".to_string())
    };

    Ok(PhysicalPosition::new(
        centered_coordinate(
            monitor_position.x,
            monitor_size.width,
            window_size.width,
        )?,
        centered_coordinate(
            monitor_position.y,
            monitor_size.height,
            window_size.height,
        )?,
    ))
}

#[cfg(target_os = "macos")]
fn show_macos_overlay(window: tauri::WebviewWindow) -> Result<(), String> {
    let raw_window = window.ns_window().map_err(|error| error.to_string())? as usize;

    window
        .run_on_main_thread(move || unsafe {
            use objc2_app_kit::{
                NSFloatingWindowLevel, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
            };

            let native_window = &*(raw_window as *const NSWindow);
            native_window
                .setStyleMask(native_window.styleMask() | NSWindowStyleMask::NonactivatingPanel);
            native_window.setCollectionBehavior(
                native_window.collectionBehavior()
                    | NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );
            native_window.setLevel(NSFloatingWindowLevel);
            native_window.orderFrontRegardless();
        })
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn frontmost_application_pid() -> Option<i32> {
    use objc2_app_kit::NSWorkspace;

    NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map(|application| application.processIdentifier())
}

#[cfg(target_os = "macos")]
fn remember_frontmost_application(controller: &PanelController) {
    if let Some(pid) = frontmost_application_pid().filter(|pid| *pid != std::process::id() as i32) {
        controller.previous_app_pid.store(pid, Ordering::SeqCst);
    }
}

#[cfg(target_os = "macos")]
fn activate_application(pid: i32) {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

    if pid > 0 {
        if let Some(application) =
            NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
        {
            application.activateWithOptions(NSApplicationActivationOptions::empty());
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn position_near_tray(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let Some(tray) = app.tray_by_id("dayflow") else {
        return window.center().map_err(|e| e.to_string());
    };

    let Ok(Some(rect)) = tray.rect() else {
        return window.center().map_err(|e| e.to_string());
    };

    let (origin_x, origin_y, width, height) = match (&rect.position, &rect.size) {
        (Position::Physical(position), Size::Physical(size)) => {
            (position.x, position.y, size.width, size.height)
        }
        _ => return window.center().map_err(|e| e.to_string()),
    };

    let size = window.outer_size().map_err(|e| e.to_string())?;
    let monitors = match app.available_monitors() {
        Ok(monitors) => monitors,
        Err(_) => return window.center().map_err(|e| e.to_string()),
    };
    let tray_center_x = origin_x.saturating_add((width / 2) as i32);
    let tray_center_y = origin_y.saturating_add((height / 2) as i32);
    let tray_monitor = monitors.iter().find(|monitor| {
        let position = monitor.position();
        let monitor_size = monitor.size();
        tray_center_x >= position.x
            && tray_center_x < position.x.saturating_add(monitor_size.width as i32)
            && tray_center_y >= position.y
            && tray_center_y < position.y.saturating_add(monitor_size.height as i32)
    });
    let current_monitor = window.current_monitor().ok().flatten();
    let monitor = tray_monitor.or(current_monitor.as_ref());
    let Some(monitor) = monitor else {
        return window.center().map_err(|e| e.to_string());
    };
    let work_area = monitor.work_area();

    let mut x = origin_x as f64 + width as f64 / 2.0 - size.width as f64 / 2.0;
    let mut y = origin_y as f64 + height as f64 + 8.0;

    let min_x = work_area.position.x as f64;
    let max_x =
        (work_area.position.x as f64 + work_area.size.width as f64 - size.width as f64).max(min_x);
    let min_y = work_area.position.y as f64;
    let max_y = (work_area.position.y as f64 + work_area.size.height as f64 - size.height as f64)
        .max(min_y);

    if x < min_x {
        x = min_x;
    }
    if x > max_x {
        x = max_x;
    }
    if y < min_y {
        y = min_y;
    }
    if y > max_y {
        y = max_y;
    }

    window
        .set_position(tauri::PhysicalPosition::new(x as i32, y as i32))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn centers_panel_in_full_monitor_bounds() {
        let position = centered_panel_position(
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(2400, 1350),
            PhysicalSize::new(450, 600),
        )
        .unwrap();

        assert_eq!(position, PhysicalPosition::new(975, 375));
    }

    #[test]
    fn panel_starts_hidden() {
        let controller = PanelController::default();
        assert!(!controller.is_visible());
        assert!(!controller.should_hide_on_blur());
    }

    #[test]
    fn show_is_suppressed_until_focus_is_gained() {
        let controller = PanelController::default();

        controller.begin_show();
        assert!(controller.is_visible());
        assert!(!controller.should_hide_on_blur());

        controller.focus_gained();
        assert!(controller.should_hide_on_blur());
    }

    #[test]
    fn blur_guard_expires_at_its_deadline() {
        assert!(!should_hide_on_blur_state(true, false, 749, 750));
        assert!(should_hide_on_blur_state(true, false, 750, 750));
    }

    #[test]
    fn panel_show_guard_is_shorter_than_a_second() {
        assert_eq!(PANEL_SHOW_GUARD, Duration::from_millis(150));
        assert_eq!(BLUR_CHECK_INTERVAL, Duration::from_millis(40));
    }

    #[test]
    fn blur_recheck_waits_only_for_the_remaining_show_guard() {
        assert_eq!(blur_check_delay_ms(100, 150), 50);
        assert_eq!(blur_check_delay_ms(200, 150), 40);
    }

    #[test]
    fn blur_guard_never_hides_a_hidden_or_hovered_panel() {
        assert!(!should_hide_on_blur_state(false, false, 900, 750));
        assert!(!should_hide_on_blur_state(true, true, 900, 750));
    }

    #[test]
    fn tray_hover_suppresses_blur_hide() {
        let controller = PanelController::default();
        controller.begin_show();
        controller.focus_gained();

        controller.begin_tray_hover();
        assert!(!controller.should_hide_on_blur());

        controller.end_tray_hover();
        assert!(controller.should_hide_on_blur());
    }

    #[test]
    fn hide_clears_visible_state() {
        let controller = PanelController::default();
        controller.begin_show();
        controller.focus_gained();

        controller.begin_hide();

        assert!(!controller.is_visible());
        assert!(!controller.should_hide_on_blur());
    }
}
