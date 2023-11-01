// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to desktop notifications.

#[cfg(windows)]
use std::path::MAIN_SEPARATOR as SEP;

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
  /// The notification sound
  sound: Option<Sound>,
}

/// Notification sound.
#[derive(Debug)]
pub enum Sound {
  /// The default notification sound.
  Default,
  /// A custom notification sound.
  ///
  /// ## Platform-specific
  ///
  /// Each OS has a different sound name so you will need to conditionally specify an appropriate sound
  /// based on the OS in use, for a list of sounds see:
  /// - **Linux**: can be one of the sounds listed in <https://0pointer.de/public/sound-naming-spec.html>
  /// - **Windows**: can be one of the sounds listed in <https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-audio>
  ///   but without the prefix, for example, if `ms-winsoundevent:Notification.Default` you would use `Default` and
  ///   if `ms-winsoundevent:Notification.Looping.Alarm2`, you would use `Alarm2`.
  ///   Windows 7 is not supported, if a sound is provided, it will play the default sound, otherwise it will be silent.
  /// - **macOS**: you can specify the name of the sound you'd like to play when the notification is shown.
  /// Any of the default sounds (under System Preferences > Sound) can be used, in addition to custom sound files.
  /// Be sure that the sound file is under one of the following locations:
  ///   - `~/Library/Sounds`
  ///   - `/Library/Sounds`
  ///   - `/Network/Library/Sounds`
  ///   - `/System/Library/Sounds`
  ///
  ///   See the [`NSSound`] docs for more information.
  ///
  /// [`NSSound`]: https://developer.apple.com/documentation/appkit/nssound
  Custom(String),
}

impl From<String> for Sound {
  fn from(value: String) -> Self {
    Self::Custom(value)
  }
}

impl From<&str> for Sound {
  fn from(value: &str) -> Self {
    Self::Custom(value.into())
  }
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
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: The app must be installed for this to have any effect.
  #[must_use]
  pub fn icon(mut self, icon: impl Into<String>) -> Self {
    self.icon = Some(icon.into());
    self
  }

  /// Sets the notification sound. By default the notification has no sound.
  ///
  /// See [`Sound`] for more information.
  #[must_use]
  pub fn sound(mut self, sound: impl Into<Sound>) -> Self {
    self.sound.replace(sound.into());
    self
  }

  /// Shows the notification.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use tauri::api::notification::Notification;
  ///
  /// // on an actual app, remove the string argument
  /// let context = tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json");
  /// Notification::new(&context.config().tauri.bundle.identifier)
  ///   .title("Tauri")
  ///   .body("Tauri is awesome!")
  ///   .show()
  ///   .unwrap();
  /// ```
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Not supported on Windows 7. If your app targets it, enable the `windows7-compat` feature and use [`Self::notify`].
  #[cfg_attr(
    all(not(doc_cfg), feature = "windows7-compat"),
    deprecated = "This function does not work on Windows 7. Use `Self::notify` instead."
  )]
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
    if let Some(sound) = self.sound {
      notification.sound_name(&match sound {
        #[cfg(target_os = "macos")]
        Sound::Default => "NSUserNotificationDefaultSoundName".to_string(),
        #[cfg(windows)]
        Sound::Default => "Default".to_string(),
        #[cfg(all(unix, not(target_os = "macos")))]
        Sound::Default => "message-new-instant".to_string(),
        Sound::Custom(c) => c,
      });
    }
    #[cfg(windows)]
    {
      let exe = tauri_utils::platform::current_exe()?;
      let exe_dir = exe.parent().expect("failed to get exe directory");
      let curr_dir = exe_dir.display().to_string();
      // set the notification's System.AppUserModel.ID only when running the installed app
      if !(curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
        || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str()))
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

  /// Shows the notification. This API is similar to [`Self::show`], but it also works on Windows 7.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use tauri::api::notification::Notification;
  ///
  /// // on an actual app, remove the string argument
  /// let context = tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json");
  /// let identifier = context.config().tauri.bundle.identifier.clone();
  ///
  /// tauri::Builder::default()
  ///   .setup(move |app| {
  ///     Notification::new(&identifier)
  ///       .title("Tauri")
  ///       .body("Tauri is awesome!")
  ///       .notify(&app.handle())
  ///       .unwrap();
  ///     Ok(())
  ///   })
  ///   .run(context)
  ///   .expect("error while running tauri application");
  /// ```
  #[cfg(feature = "windows7-compat")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "windows7-compat")))]
  #[allow(unused_variables)]
  pub fn notify<R: crate::Runtime>(self, app: &crate::AppHandle<R>) -> crate::api::Result<()> {
    #[cfg(windows)]
    {
      if crate::utils::platform::is_windows_7() {
        self.notify_win7(app)
      } else {
        #[allow(deprecated)]
        self.show()
      }
    }
    #[cfg(not(windows))]
    {
      #[allow(deprecated)]
      self.show()
    }
  }

  #[cfg(all(windows, feature = "windows7-compat"))]
  fn notify_win7<R: crate::Runtime>(self, app: &crate::AppHandle<R>) -> crate::api::Result<()> {
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
      notification.silent(self.sound.is_none());
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
}
