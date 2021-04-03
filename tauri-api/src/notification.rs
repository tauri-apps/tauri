#[cfg(windows)]
use std::path::MAIN_SEPARATOR;

/// The Notification definition.
/// Allows you to construct a Notification data and send it.
///
/// # Example
/// ```
/// use tauri_api::notification::Notification;
/// // shows a notification with the given title and body
/// Notification::new("studio.tauri.example")
///   .title("New message")
///   .body("You've got a new message.")
///   .show();
/// ```
#[allow(dead_code)]
#[derive(Default)]
pub struct Notification {
  /// The notification body.
  body: Option<String>,
  /// The notification title.
  title: Option<String>,
  /// The notification icon.
  icon: Option<String>,
  /// The notification identifier
  identifier: String,
}

impl Notification {
  /// Initializes a instance of a Notification.
  pub fn new(identifier: impl Into<String>) -> Self {
    Self {
      identifier: identifier.into(),
      ..Default::default()
    }
  }

  /// Sets the notification body.
  pub fn body(mut self, body: impl Into<String>) -> Self {
    self.body = Some(body.into());
    self
  }

  /// Sets the notification title.
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }

  /// Sets the notification icon.
  pub fn icon(mut self, icon: impl Into<String>) -> Self {
    self.icon = Some(icon.into());
    self
  }

  /// Shows the notification.
  pub fn show(self) -> crate::Result<()> {
    let mut notification = notify_rust::Notification::new();
    if let Some(body) = self.body {
      notification.body(&body);
    }
    if let Some(title) = self.title {
      notification.summary(&title);
    }
    if let Some(icon) = self.icon {
      notification.icon(&icon);
    }
    #[cfg(windows)]
    {
      let exe = std::env::current_exe()?;
      let exe_dir = exe.parent().expect("failed to get exe directory");
      let curr_dir = exe_dir.display().to_string();
      // set the notification's System.AppUserModel.ID only when running the installed runtime
      if !(curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
        || curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str()))
      {
        notification.app_id(&self.identifier);
      }
    }
    notification.show()?;
    Ok(())
  }
}
