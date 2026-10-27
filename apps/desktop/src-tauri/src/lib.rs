use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

pub mod app;
pub mod domain;
pub mod panel;
pub mod repo;
pub mod time;

use crate::app::{DesktopTaskService, TaskService};
use crate::panel::PanelController;
use crate::repo::sqlite::SqliteConnection;
use crate::repo::{migrate, SqliteSettingsRepository, SqliteTaskRepository};
use crate::time::SystemClock;

#[tauri::command]
fn panel_show(app: AppHandle) -> Result<(), String> {
    panel::show(&app)
}

#[tauri::command]
fn panel_hide(app: AppHandle) -> Result<(), String> {
    panel::hide(&app)
}

#[tauri::command]
fn panel_toggle(app: AppHandle) -> Result<(), String> {
    panel::toggle(&app)
}

#[tauri::command]
fn panel_quit(app: AppHandle) {
    panel::quit(&app)
}

#[tauri::command]
fn panel_state(state: State<PanelController>) -> Result<String, String> {
    Ok(if state.is_visible() {
        "visible".to_string()
    } else {
        "hidden".to_string()
    })
}

#[tauri::command]
fn save_draft(state: State<DesktopTaskService>, draft: String) -> Result<(), String> {
    state.set_draft(&draft).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_draft(state: State<DesktopTaskService>) -> Result<String, String> {
    state.get_draft().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("dayflow.sqlite");

            let conn = SqliteConnection::open(&db_path)?;
            migrate(&conn)?;

            let tasks = Arc::new(SqliteTaskRepository::new(conn.clone()));
            let settings = Arc::new(SqliteSettingsRepository::new(conn.clone()));
            let clock = Arc::new(SystemClock);
            let service = TaskService::new(tasks, settings, clock);

            app.manage(service);
            app.manage(PanelController::default());

            panel::setup_tray(app.handle())?;
            panel::setup_window_events(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            panel_show,
            panel_hide,
            panel_toggle,
            panel_quit,
            panel_state,
            save_draft,
            load_draft
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
