use crate::constants::timeouts::NOTIFICATION_AUTO_DISMISS_TIME;

/// Types of notifications that can be displayed to the user.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    /// Success notification (green)
    Success,
    /// Error notification (red)
    Error,
    /// Warning notification (yellow)
    Warning,
    /// Info notification (blue)
    Info,
}

/// Represents a notification message displayed to the user.
/// Notifications can auto-dismiss after a duration or be persistent.
#[derive(Debug, Clone)]
pub struct Notification {
    /// The notification message text
    pub message: String,
    /// The type/severity of the notification
    pub notification_type: NotificationType,
    /// When the notification was created
    pub timestamp: chrono::DateTime<chrono::Local>,
    /// Optional auto-dismiss duration. None means persistent.
    pub auto_dismiss_after: Option<std::time::Duration>,
}

impl Notification {
    /// Creates a new notification with the specified message and type.
    /// Sets auto-dismiss to 5 seconds by default.
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: Some(NOTIFICATION_AUTO_DISMISS_TIME),
        }
    }

    /// Creates a success notification (green) with 5-second auto-dismiss.
    pub fn success(message: String) -> Self {
        Self::new(message, NotificationType::Success)
    }

    /// Creates an error notification (red) with 5-second auto-dismiss.
    pub fn error(message: String) -> Self {
        Self::new(message, NotificationType::Error)
    }

    /// Creates a warning notification (yellow) with 5-second auto-dismiss.
    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    /// Creates an info notification (blue) with 5-second auto-dismiss.
    pub fn info(message: String) -> Self {
        Self::new(message, NotificationType::Info)
    }

    /// Creates a persistent notification that won't auto-dismiss.
    /// User must manually dismiss or clear all notifications.
    pub fn persistent(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: None,
        }
    }

    /// Checks if this notification should be automatically dismissed.
    /// Returns true if the auto-dismiss duration has elapsed.
    pub fn should_dismiss(&self) -> bool {
        if let Some(duration) = self.auto_dismiss_after {
            chrono::Local::now().signed_duration_since(self.timestamp)
                > chrono::Duration::from_std(duration).unwrap_or_default()
        } else {
            false
        }
    }
}
