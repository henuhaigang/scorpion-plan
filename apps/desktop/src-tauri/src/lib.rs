use std::sync::Arc;

use chrono::{NaiveDate, NaiveTime};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

pub mod app;
pub mod domain;
pub mod panel;
pub mod repo;
pub mod scheduler;
pub mod time;

use crate::app::backup_service::BackupService;
use crate::app::reminder_service::ReminderService;
use crate::app::{CreateTaskCommand, DesktopTaskService, TaskService, UpdateTaskCommand};
use crate::domain::{GlobalReminderDefault, Task, TaskPriority};
use crate::panel::PanelController;
use crate::repo::sqlite::SqliteConnection;
use crate::repo::{
    migrate, SqliteReminderJobRepository, SqliteSettingsRepository, SqliteTaskRepository,
};
use crate::scheduler::TauriNotificationSender;
use crate::time::SystemClock;

type ConcreteReminderService = ReminderService<
    SqliteTaskRepository,
    SqliteSettingsRepository,
    SqliteReminderJobRepository,
    TauriNotificationSender,
    SystemClock,
>;

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

fn parse_uuid(raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|e| e.to_string())
}

fn parse_date(raw: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|e| e.to_string())
}

fn parse_time(raw: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(raw, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(raw, "%H:%M:%S"))
        .map_err(|e| e.to_string())
}

fn parse_priority(raw: &str) -> Result<TaskPriority, String> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "LOW" => Ok(TaskPriority::Low),
        "NORMAL" => Ok(TaskPriority::Normal),
        "HIGH" => Ok(TaskPriority::High),
        other => Err(format!("invalid priority: {other}")),
    }
}

#[tauri::command]
fn list_board(state: State<DesktopTaskService>) -> Result<crate::app::TaskBoard, String> {
    state.list_board().map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn create_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    title: String,
    date: Option<String>,
    time: Option<String>,
    all_day: bool,
    priority: String,
    category_id: Option<String>,
) -> Result<Task, String> {
    let mut cmd = CreateTaskCommand::quick_add(parse_uuid(&id)?, title);
    if let Some(d) = date {
        cmd.date = Some(parse_date(&d)?);
    }
    if let Some(t) = time {
        cmd.time = Some(parse_time(&t)?);
    }
    cmd.all_day = all_day;
    cmd.priority = parse_priority(&priority)?;
    if let Some(c) = category_id {
        cmd.category_id = Some(parse_uuid(&c)?);
    }
    let task = state.create_task(cmd).map_err(|e| e.to_string())?;
    let _ = reminders.reconcile_task(task.id);
    Ok(task)
}

#[tauri::command]
fn update_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
    title: Option<String>,
    priority: Option<String>,
    category_id: Option<String>,
) -> Result<Task, String> {
    let cmd = UpdateTaskCommand {
        title,
        priority: priority.map(|p| parse_priority(&p)).transpose()?,
        category_id: category_id.map(|c| parse_uuid(&c)).transpose()?.map(Some),
        reminder_preference: None,
    };
    let task = state
        .update_task(parse_uuid(&id)?, expected_version, cmd)
        .map_err(|e| e.to_string())?;
    let _ = reminders.reconcile_task(task.id);
    Ok(task)
}

#[tauri::command]
fn reschedule_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
    date: String,
    time: Option<String>,
) -> Result<Task, String> {
    // `time` semantics: None = keep current time; Some("") = clear time;
    // Some("HH:MM") = set a specific time.
    let new_time = match time.as_deref() {
        None => None,
        Some("") => Some(None),
        Some(t) => Some(Some(parse_time(t)?)),
    };
    let task = state
        .reschedule_task(
            parse_uuid(&id)?,
            expected_version,
            parse_date(&date)?,
            new_time,
        )
        .map_err(|e| e.to_string())?;
    let _ = reminders.reconcile_task(task.id);
    Ok(task)
}

#[tauri::command]
fn complete_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
) -> Result<Task, String> {
    let task = state
        .complete_task(parse_uuid(&id)?, expected_version)
        .map_err(|e| e.to_string())?;
    let _ = reminders.cancel_for_task(task.id);
    Ok(task)
}

#[tauri::command]
fn undo_completion(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
) -> Result<Task, String> {
    let task = state
        .undo_completion(parse_uuid(&id)?, expected_version)
        .map_err(|e| e.to_string())?;
    let _ = reminders.reconcile_task(task.id);
    Ok(task)
}

#[tauri::command]
fn cancel_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
) -> Result<Task, String> {
    let task = state
        .cancel_task(parse_uuid(&id)?, expected_version)
        .map_err(|e| e.to_string())?;
    let _ = reminders.cancel_for_task(task.id);
    Ok(task)
}

#[tauri::command]
fn restore_canceled_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
) -> Result<Task, String> {
    let task = state
        .restore_canceled_task(parse_uuid(&id)?, expected_version)
        .map_err(|e| e.to_string())?;
    let _ = reminders.reconcile_task(task.id);
    Ok(task)
}

#[tauri::command]
fn soft_delete_task(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    id: String,
    expected_version: i64,
) -> Result<Task, String> {
    let task = state
        .soft_delete_task(parse_uuid(&id)?, expected_version)
        .map_err(|e| e.to_string())?;
    let _ = reminders.cancel_for_task(task.id);
    Ok(task)
}

#[tauri::command]
fn export_backup(backup: State<BackupService>, app: AppHandle) -> Result<String, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let backup_dir = BackupService::default_backup_dir(&data_dir);
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let dest = backup_dir.join(format!("dayflow_backup_{}.sqlite", timestamp));
    let path = backup.export(&dest).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_global_reminder_default(
    state: State<DesktopTaskService>,
) -> Result<GlobalReminderDefault, String> {
    state.global_reminder_default().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_global_reminder_default(
    state: State<DesktopTaskService>,
    reminders: State<Arc<ConcreteReminderService>>,
    enabled: bool,
    minutes_before: u32,
) -> Result<(), String> {
    let value = GlobalReminderDefault {
        enabled,
        minutes_before,
    };
    state
        .set_global_reminder_default(&value)
        .map_err(|e| e.to_string())?;
    let _ = reminders.reconcile_all();
    Ok(())
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
            let jobs = Arc::new(SqliteReminderJobRepository::new(conn.clone()));
            let clock = Arc::new(SystemClock);
            let notifier = Arc::new(TauriNotificationSender::new(app.handle().clone()));

            let service = TaskService::new(tasks.clone(), settings.clone(), clock.clone());
            let reminder_service = Arc::new(ReminderService::new(
                tasks.clone(),
                settings.clone(),
                jobs.clone(),
                notifier.clone(),
                clock.clone(),
            ));
            let backup_service = BackupService::new(conn.clone());

            app.manage(service);
            app.manage(reminder_service.clone());
            app.manage(backup_service);
            app.manage(PanelController::default());

            panel::setup_tray(app.handle())?;
            panel::setup_window_events(app.handle())?;

            // Startup reconciliation: sync reminder jobs with task state.
            let rs = reminder_service.clone();
            std::thread::spawn(move || {
                let _ = rs.reconcile_all();
            });

            // Background polling loop: check for due reminders every 30 seconds.
            let poll_service = reminder_service.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(30));
                let _ = poll_service.poll_due();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            panel_show,
            panel_hide,
            panel_toggle,
            panel_quit,
            panel_state,
            save_draft,
            load_draft,
            list_board,
            create_task,
            update_task,
            reschedule_task,
            complete_task,
            undo_completion,
            cancel_task,
            restore_canceled_task,
            soft_delete_task,
            get_global_reminder_default,
            set_global_reminder_default,
            export_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
