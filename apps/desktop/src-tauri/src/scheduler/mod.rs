/// Port for OS notification delivery. The domain and application layers
/// depend on this trait; the concrete Tauri implementation lives here.
pub trait NotificationSender: Send + Sync {
    /// Deliver a notification. Returns `Err` if permission is denied or the
    /// OS rejects the send.
    fn send(&self, title: &str, body: &str) -> Result<(), SchedulerError>;
    /// Whether notifications are currently permitted.
    fn permission_granted(&self) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("notification permission denied")]
    PermissionDenied,
    #[error("notification send failed: {0}")]
    SendFailed(String),
}

/// Production sender backed by `tauri-plugin-notification`.
pub struct TauriNotificationSender {
    app_handle: tauri::AppHandle,
}

impl TauriNotificationSender {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl NotificationSender for TauriNotificationSender {
    fn send(&self, title: &str, body: &str) -> Result<(), SchedulerError> {
        use tauri_plugin_notification::NotificationExt;
        self.app_handle
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| SchedulerError::SendFailed(e.to_string()))
    }

    fn permission_granted(&self) -> bool {
        use tauri::plugin::PermissionState;
        use tauri_plugin_notification::NotificationExt;
        self.app_handle
            .notification()
            .permission_state()
            .map(|s| s == PermissionState::Granted)
            .unwrap_or(false)
    }
}

/// In-memory sender for tests: records sent notifications.
#[derive(Debug, Default)]
pub struct FakeNotificationSender {
    pub sent: std::sync::Mutex<Vec<(String, String)>>,
    pub fail: std::sync::atomic::AtomicBool,
}

impl FakeNotificationSender {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_fail(&self, fail: bool) {
        self.fail.store(fail, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn sent_count(&self) -> usize {
        self.sent.lock().unwrap().len()
    }
}

impl NotificationSender for FakeNotificationSender {
    fn send(&self, title: &str, body: &str) -> Result<(), SchedulerError> {
        if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(SchedulerError::SendFailed("simulated failure".into()));
        }
        self.sent
            .lock()
            .unwrap()
            .push((title.to_string(), body.to_string()));
        Ok(())
    }

    fn permission_granted(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_sender_records_notifications() {
        let sender = FakeNotificationSender::new();
        sender.send("Title", "Body").unwrap();
        assert_eq!(sender.sent_count(), 1);
        let sent = sender.sent.lock().unwrap();
        assert_eq!(sent[0], ("Title".to_string(), "Body".to_string()));
    }

    #[test]
    fn fake_sender_can_simulate_failure() {
        let sender = FakeNotificationSender::new();
        sender.set_fail(true);
        assert!(sender.send("T", "B").is_err());
        assert_eq!(sender.sent_count(), 0);
    }
}
