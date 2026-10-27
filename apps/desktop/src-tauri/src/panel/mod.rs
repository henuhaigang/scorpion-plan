use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Position, Size, WebviewWindow, WindowEvent,
};

#[derive(Debug, Default)]
pub struct PanelController {
    visible: AtomicBool,
    tray_hover: AtomicBool,
    suppress_until_focused: AtomicBool,
}

impl PanelController {
    pub fn begin_show(&self) {
        self.suppress_until_focused.store(true, Ordering::SeqCst);
        self.visible.store(true, Ordering::SeqCst);
    }

    pub fn begin_hide(&self) {
        self.visible.store(false, Ordering::SeqCst);
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::SeqCst)
    }

    pub fn focus_gained(&self) {
        self.suppress_until_focused.store(false, Ordering::SeqCst);
    }

    pub fn begin_tray_hover(&self) {
        self.tray_hover.store(true, Ordering::SeqCst);
    }

    pub fn end_tray_hover(&self) {
        self.tray_hover.store(false, Ordering::SeqCst);
    }

    pub fn should_hide_on_blur(&self) -> bool {
        self.is_visible()
            && !self.tray_hover.load(Ordering::SeqCst)
            && !self.suppress_until_focused.load(Ordering::SeqCst)
    }
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is missing".to_string())?;
    let controller = app.state::<PanelController>();
    if controller.is_visible() {
        return Ok(());
    }

    controller.begin_show();
    position_near_tray(app, &window)?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    let controller = app.state::<PanelController>();
    if !controller.is_visible() {
        return Ok(());
    }

    controller.begin_hide();
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
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
    let controller = app.state::<PanelController>();
    if !controller.should_hide_on_blur() {
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        if window.is_focused().unwrap_or(false) {
            return;
        }
        let _ = hide(app);
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "default window icon is missing".to_string())?;

    let toggle_item = MenuItem::with_id(app, "toggle", "显示/隐藏面板", true, None::<String>)
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
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
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
        WindowEvent::Focused(false) => maybe_hide_if_unfocused(&handle),
        WindowEvent::CloseRequested { api, .. } => {
            let _ = hide(&handle);
            api.prevent_close();
        }
        _ => {}
    });

    Ok(())
}

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
    let Ok(Some(monitor)) = window.current_monitor() else {
        return window.center().map_err(|e| e.to_string());
    };
    let work_area = monitor.work_area();

    let mut x = origin_x as f64 + width as f64 / 2.0 - size.width as f64 / 2.0;
    let mut y = origin_y as f64 + height as f64 + 8.0;

    let min_x = work_area.position.x as f64;
    let max_x = work_area.position.x as f64 + work_area.size.width as f64 - size.width as f64;
    let min_y = work_area.position.y as f64;
    let max_y = work_area.position.y as f64 + work_area.size.height as f64 - size.height as f64;

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
