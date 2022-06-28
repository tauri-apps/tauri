// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to desktop notifications.

use crate::{AppHandle, Runtime};
#[cfg(windows)]
use std::path::MAIN_SEPARATOR;

/// The desktop notification definition.
///
/// Allows you to construct a Notification data and send it.
///
/// # Examples
/// ```rust,no_run
/// use tauri::api::notification::Notification;
/// // first we build the application to access the Tauri configuration
/// let app = tauri::Builder::default()
///   // on an actual app, remove the string argument
///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///   .expect("error while building tauri application");
///
/// // shows a notification with the given title and body
/// Notification::new(&app.config().tauri.bundle.identifier)
///   .title("New message")
///   .body("You've got a new message.")
///   .show();
///
/// // run the app
/// app.run(|_app_handle, _event| {});
/// ```
#[allow(dead_code)]
#[derive(Debug, Default)]
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
  #[must_use]
  pub fn body(mut self, body: impl Into<String>) -> Self {
    self.body = Some(body.into());
    self
  }

  /// Sets the notification title.
  #[must_use]
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }

  /// Sets the notification icon.
  #[must_use]
  pub fn icon(mut self, icon: impl Into<String>) -> Self {
    self.icon = Some(icon.into());
    self
  }

  /// Shows the notification. This API is only available when the app because it offers compatibility with Windows 7.
  ///
  /// If your app does not run on Windows 7, you can use [`Self::show`] which can be executed in other contexts.
  #[allow(unused_variables)]
  pub fn notify<R: Runtime>(self, app: &AppHandle<R>) -> crate::api::Result<()> {
    #[cfg(windows)]
    {
      let is_windows7 = sys_info::os_release()
        .map(|release| release.starts_with("6.1"))
        .unwrap_or_default();
      if is_windows7 {
        self.notify_win7(app)
      } else {
        self.show()
      }
    }
    #[cfg(not(windows))]
    {
      self.show()
    }
  }

  #[cfg(windows)]
  fn notify_win7<R: Runtime>(self, app: &AppHandle<R>) -> crate::api::Result<()> {
    let app = app.clone();
    let default_window_icon = app.manager.inner.default_window_icon.clone();
    let _ = app.run_on_main_thread(move || {
      let mut notification = win7_notifications::Notification::new();
      if let Some(body) = self.body {
        notification.body(&body);
      }
      if let Some(title) = self.title {
        notification.summary(&title);
      }
      if let Some(crate::Icon::Rgba {
        rgba,
        width,
        height,
      }) = default_window_icon
      {
        notification.icon(rgba, width, height);
      }
      let _ = notification.show();
    });

    Ok(())
  }

  /// Shows the notification. If you need Windows 7 support, use [`Self::notify`] instead.
  pub fn show(self) -> crate::api::Result<()> {
    let mut notification = notify_rust::Notification::new();
    if let Some(body) = self.body {
      notification.body(&body);
    }
    if let Some(title) = self.title {
      notification.summary(&title);
    }
    if let Some(icon) = self.icon {
      notification.icon(&icon);
    } else {
      notification.auto_icon();
    }
    #[cfg(windows)]
    {
      let exe = tauri_utils::platform::current_exe()?;
      let exe_dir = exe.parent().expect("failed to get exe directory");
      let curr_dir = exe_dir.display().to_string();
      // set the notification's System.AppUserModel.ID only when running the installed app
      if !(curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
        || curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str()))
      {
        notification.app_id(&self.identifier);
      }
    }
    #[cfg(target_os = "macos")]
    {
      let _ = notify_rust::set_application(if cfg!(feature = "custom-protocol") {
        &self.identifier
      } else {
        "com.apple.Terminal"
      });
    }

    crate::async_runtime::spawn(async move {
      let _ = notification.show();
    });

    Ok(())
  }
}
