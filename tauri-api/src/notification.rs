#[cfg(windows)]
use crate::config::get as get_config;
#[cfg(windows)]
use std::path::MAIN_SEPARATOR;

/// The Notification definition.
/// Allows you to construct a Notification data and send it.
///
/// # Example
/// ```
/// use tauri_api::notification::Notification;
/// // shows a notification with the given title and body
/// Notification::new()
///   .title("New message")
///   .body("You've got a new message.")
///   .show();
/// ```
#[allow(dead_code)]
pub struct Notification {
  /// The notification body.
  body: Option<String>,
  /// The notification title.
  title: Option<String>,
  /// The notification icon.
  icon: Option<String>,
}

impl Notification {
  /// Initializes a instance of a Notification.
  pub fn new() -> Self {
    Self {
      body: None,
      title: None,
      icon: None,
    }
  }

  /// Sets the notification body.
  pub fn body(&mut self, body: String) -> &mut Self {
    self.body = Some(body);
    self
  }

  /// Sets the notification title.
  pub fn title(&mut self, title: String) -> &mut Self {
    self.title = Some(title);
    self
  }

  /// Sets the notification icon.
  pub fn icon(&mut self, icon: String) -> &mut Self {
    self.icon = Some(icon);
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
      // set the notification's System.AppUserModel.ID only when running the installed app
      if !(curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
        || curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str()))
      {
        let config = get_config()?;
        let identifier = config.tauri.bundle.identifier.clone();
        notification.app_id(&identifier);
      }
    }
    notification
      .show()
      .map(|_| ())
      .map_err(|e| anyhow::anyhow!(e.to_string()))
  }
}
