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
    Uuid::parse_str(raw).map_err(|e| format!("invalid uuid `{raw}`: {e}"))
}

fn parse_date(raw: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|e| format!("invalid date `{raw}`: {e}"))
}

fn parse_time(raw: &str) -> Result<NaiveTime, String> {
    let value = raw.trim();
    NaiveTime::parse_from_str(value, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M:%S"))
        .map_err(|e| format!("invalid time `{raw}`: {e}"))
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
fn list_history(state: State<DesktopTaskService>) -> Result<crate::app::HistoryView, String> {
    let items = state.list_history().map_err(|e| e.to_string())?;
    Ok(crate::app::HistoryView { items })
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
    all_day: Option<bool>,
    priority: Option<String>,
    category_id: Option<String>,
) -> Result<Task, String> {
    let mut cmd = CreateTaskCommand::quick_add(parse_uuid(&id)?, title);
    if let Some(d) = date.filter(|value| !value.trim().is_empty()) {
        cmd.date = Some(parse_date(&d)?);
    }
    if let Some(t) = time.filter(|value| !value.trim().is_empty()) {
        cmd.time = Some(parse_time(&t)?);
    }
    cmd.all_day = all_day.unwrap_or(false);
    cmd.priority = match priority {
        Some(value) if !value.trim().is_empty() => parse_priority(&value)?,
        _ => TaskPriority::Normal,
    };
    if let Some(c) = category_id.filter(|value| !value.trim().is_empty()) {
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
        Some(value) if value.trim().is_empty() => Some(None),
        Some(t) => Some(Some(parse_time(t)?)),
    };
    if date.trim().is_empty() {
        return Err("date is required".to_string());
    }
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
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = panel::toggle(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                use objc2::MainThreadMarker;
                use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
                use tauri_plugin_global_shortcut::GlobalShortcutExt;

                let main_thread = MainThreadMarker::new().ok_or_else(|| {
                    std::io::Error::other("macOS application setup must run on the main thread")
                })?;
                let native_app = NSApplication::sharedApplication(main_thread);
                if !native_app.setActivationPolicy(NSApplicationActivationPolicy::Accessory) {
                    return Err(std::io::Error::other(
                        "could not set macOS accessory activation policy",
                    )
                    .into());
                }

                app.global_shortcut()
                    .register("CommandOrControl+Shift+Space")?;
            }

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

           if std::env::var_os("DAYFLOW_SELF_TEST").is_some() {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let hold_ms = std::env::var("DAYFLOW_SELF_TEST_HOLD_MS")
                        .ok()
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or_default()
                        .to_string();
                    let report = if let Some(window) = handle.get_webview_window("main") {
                        let script = r#"
                           window.__dayflowSelfTestResult = '';
                           (async () => {
                             const result = { ok: false, steps: [] };
                             const invoke = window.__TAURI_INTERNALS__.invoke;
                             const step = async (name, fn) => {
                               try {
                                 const value = await fn();
                                 result.steps.push({ name, ok: true, value });
                                 return value;
                               } catch (error) {
                                 result.steps.push({
                                   name,
                                   ok: false,
                                   error: {
                                     name: error && error.name,
                                     message: error && error.message,
                                     stack: error && error.stack
                                   }
                                 });
                                 throw error;
                               }
                             };
                             const probeWindow = async () => {
                               try {
                                 const label = window.__TAURI_INTERNALS__.metadata.currentWindow.label;
                                 return {
                                   label,
                                   visible: await invoke('plugin:window|is_visible', { label }),
                                   focused: await invoke('plugin:window|is_focused', { label }),
                                   alwaysOnTop: await invoke('plugin:window|is_always_on_top', { label }),
                                   position: await invoke('plugin:window|outer_position', { label }),
                                   size: await invoke('plugin:window|outer_size', { label })
                                 };
                               } catch (error) {
                                 return { error: String(error) };
                               }
                             };
                             try {
                               result.panelBefore = await step('panel_state_before', () => invoke('panel_state'));
                               await step('panel_show', () => invoke('panel_show'));
                               await new Promise((resolve) => setTimeout(resolve, __HOLD_MS__));
                               await new Promise((resolve) => setTimeout(resolve, 750));
                               result.panelAfterShow = await step('panel_state_after_show', () => invoke('panel_state'));
                               result.windowAfterShow = await probeWindow();
                               const id = crypto.randomUUID();
                               const created = await step('create_task_with_time', () => invoke('create_task', {
                                 id,
                                 title: 'self-test',
                                 date: '2026-10-29',
                                 time: '09:00',
                                 allDay: false,
                                 priority: 'NORMAL'
                               }));
                               const createdNoTime = await step('create_task_without_time', () => invoke('create_task', {
                                 id: crypto.randomUUID(),
                                 title: 'self-test-no-time',
                                 date: '2026-10-29',
                                 allDay: false,
                                 priority: 'NORMAL'
                               }));
                               const createdAllDay = await step('create_task_all_day', () => invoke('create_task', {
                                 id: crypto.randomUUID(),
                                 title: 'self-test-allday',
                                 date: '2026-10-29',
                                 allDay: true
                               }));
                               const updated = await step('update_task', () => invoke('update_task', {
                                 id: created.id,
                                 expectedVersion: created.version,
                                 title: 'self-test-updated'
                               }));
                               const cleared = await step('reschedule_clear_time', () => invoke('reschedule_task', {
                                 id: updated.id,
                                 expectedVersion: updated.version,
                                 date: '2026-10-30',
                                 time: ''
                               }));
                               const timed = await step('reschedule_set_time', () => invoke('reschedule_task', {
                                 id: cleared.id,
                                 expectedVersion: cleared.version,
                                 date: '2026-10-30',
                                 time: '10:00'
                               }));
                               const board = await step('list_board', () => invoke('list_board'));
                               result.boardTimedCount = board.today.timed.length;
                               await step('soft_delete_created', () => invoke('soft_delete_task', {
                                 id: timed.id,
                                 expectedVersion: timed.version
                               }));
                               await step('soft_delete_created_no_time', () => invoke('soft_delete_task', {
                                 id: createdNoTime.id,
                                 expectedVersion: createdNoTime.version
                               }));
                               await step('soft_delete_created_all_day', () => invoke('soft_delete_task', {
                                 id: createdAllDay.id,
                                 expectedVersion: createdAllDay.version
                               }));
                               await step('panel_hide', () => invoke('panel_hide'));
                               await new Promise((resolve) => setTimeout(resolve, 250));
                               result.panelAfterHide = await step('panel_state_after_hide', () => invoke('panel_state'));
                               result.windowAfterHide = await probeWindow();
                               result.ok = true;
                             } catch (error) {
                               result.ok = false;
                               result.fatal = {
                                 name: error && error.name,
                                 message: error && error.message,
                                 stack: error && error.stack
                               };
                             }
                             window.__dayflowSelfTestResult = JSON.stringify(result);
                           })();
                         "#
                        .replace("__HOLD_MS__", &hold_ms);
                        let eval_result = window.eval(script);
                        let mut result_value = String::new();
                        for _ in 0..40 {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            let (tx, rx) = std::sync::mpsc::channel();
                            let poll_result = window.eval_with_callback(
                                "window.__dayflowSelfTestResult || ''",
                                move |value| {
                                    let _ = tx.send(value);
                                },
                            );
                            if poll_result.is_err() {
                                break;
                            }
                            if let Ok(value) = rx.recv_timeout(std::time::Duration::from_secs(1))
                            {
                                if !value.is_empty() && value != "null" && value != "\"\"" {
                                    result_value = value;
                                    break;
                                }
                            }
                        }
                        format!("eval={eval_result:?} result={result_value}")
                    } else {
                        "main window missing".to_string()
                    };
                    println!("DAYFLOW_SELF_TEST {report}");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let _ = panel::hide(&handle);
                    handle.exit(0);
                });
            }

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
            list_history,
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

#[cfg(test)]
mod tests {
    use super::parse_time;
    use chrono::NaiveTime;

    #[test]
    fn parses_time_input_values_with_or_without_seconds() {
        assert_eq!(
            parse_time("09:30").unwrap(),
            NaiveTime::from_hms_opt(9, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time(" 09:30:15 ").unwrap(),
            NaiveTime::from_hms_opt(9, 30, 15).unwrap()
        );
    }

    #[test]
    fn rejects_invalid_time_input_values() {
        assert!(parse_time("25:30").is_err());
        assert!(parse_time("not a time").is_err());
    }
}
