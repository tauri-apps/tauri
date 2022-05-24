// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Use native message and file open/save dialogs.
//!
//! This module exposes non-blocking APIs on its root, relying on callback closures
//! to give results back. This is particularly useful when running dialogs from the main thread.
//! When using on asynchronous contexts such as async commands, the [`blocking`] APIs are recommended.

pub use nonblocking::*;

#[cfg(not(target_os = "linux"))]
macro_rules! run_dialog {
  ($e:expr, $h: ident) => {{
    std::thread::spawn(move || {
      let response = $e;
      $h(response);
    });
  }};
}

#[cfg(target_os = "linux")]
macro_rules! run_dialog {
  ($e:expr, $h: ident) => {{
    std::thread::spawn(move || {
      let context = glib::MainContext::default();
      context.invoke_with_priority(glib::PRIORITY_HIGH, move || {
        let response = $e;
        $h(response);
      });
    });
  }};
}

#[cfg(not(target_os = "linux"))]
macro_rules! run_file_dialog {
  ($e:expr, $h: ident) => {{
    std::thread::spawn(move || {
      let response = crate::async_runtime::block_on($e);
      $h(response);
    });
  }};
}

#[cfg(target_os = "linux")]
macro_rules! run_file_dialog {
  ($e:expr, $h: ident) => {{
    std::thread::spawn(move || {
      let context = glib::MainContext::default();
      context.invoke_with_priority(glib::PRIORITY_HIGH, move || {
        let response = $e;
        $h(response);
      });
    });
  }};
}

macro_rules! run_dialog_sync {
  ($e:expr) => {{
    let (tx, rx) = sync_channel(0);
    let cb = move |response| {
      tx.send(response).unwrap();
    };
    run_file_dialog!($e, cb);
    rx.recv().unwrap()
  }};
}

macro_rules! file_dialog_builder {
  () => {
    #[cfg(target_os = "linux")]
    type FileDialog = rfd::FileDialog;
    #[cfg(not(target_os = "linux"))]
    type FileDialog = rfd::AsyncFileDialog;

    /// The file dialog builder.
    ///
    /// Constructs file picker dialogs that can select single/multiple files or directories.
    #[derive(Debug, Default)]
    pub struct FileDialogBuilder(FileDialog);

    impl FileDialogBuilder {
      /// Gets the default file dialog builder.
      pub fn new() -> Self {
        Default::default()
      }

      /// Add file extension filter. Takes in the name of the filter, and list of extensions
      #[must_use]
      pub fn add_filter(mut self, name: impl AsRef<str>, extensions: &[&str]) -> Self {
        self.0 = self.0.add_filter(name.as_ref(), extensions);
        self
      }

      /// Set starting directory of the dialog.
      #[must_use]
      pub fn set_directory<P: AsRef<Path>>(mut self, directory: P) -> Self {
        self.0 = self.0.set_directory(directory);
        self
      }

      /// Set starting file name of the dialog.
      #[must_use]
      pub fn set_file_name(mut self, file_name: &str) -> Self {
        self.0 = self.0.set_file_name(file_name);
        self
      }

      /// Sets the parent window of the dialog.
      #[must_use]
      pub fn set_parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.0 = self.0.set_parent(parent);
        self
      }

      /// Set the title of the dialog.
      #[must_use]
      pub fn set_title(mut self, title: &str) -> Self {
        self.0 = self.0.set_title(title);
        self
      }
    }
  };
}

macro_rules! message_dialog_builder {
  () => {
    /// A builder for message dialogs.
    pub struct MessageDialogBuilder(rfd::MessageDialog);

    impl MessageDialogBuilder {
      /// Creates a new message dialog builder.
      pub fn new(title: impl AsRef<str>, message: impl AsRef<str>) -> Self {
        let title = title.as_ref().to_string();
        let message = message.as_ref().to_string();
        Self(
          rfd::MessageDialog::new()
            .set_title(&title)
            .set_description(&message),
        )
      }

      /// Set parent windows explicitly (optional)
      ///
      /// ## Platform-specific
      ///
      /// - **Linux:** Unsupported.
      pub fn parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.0 = self.0.set_parent(parent);
        self
      }

      /// Set the set of button that will be displayed on the dialog.
      pub fn buttons(mut self, buttons: MessageDialogButtons) -> Self {
        self.0 = self.0.set_buttons(buttons.into());
        self
      }

      /// Set type of a dialog.
      ///
      /// Depending on the system it can result in type specific icon to show up,
      /// the will inform user it message is a error, warning or just information.
      pub fn kind(mut self, kind: MessageDialogKind) -> Self {
        self.0 = self.0.set_level(kind.into());
        self
      }
    }
  };
}

/// Options for action buttons on message dialogs.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MessageDialogButtons {
  /// Ok button.
  Ok,
  /// Ok and Cancel buttons.
  OkCancel,
  /// Yes and No buttons.
  YesNo,
}

impl From<MessageDialogButtons> for rfd::MessageButtons {
  fn from(kind: MessageDialogButtons) -> Self {
    match kind {
      MessageDialogButtons::Ok => Self::Ok,
      MessageDialogButtons::OkCancel => Self::OkCancel,
      MessageDialogButtons::YesNo => Self::YesNo,
    }
  }
}

/// Types of message, ask and confirm dialogs.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MessageDialogKind {
  /// Information dialog.
  Info,
  /// Warning dialog.
  Warning,
  /// Error dialog.
  Error,
}

impl From<MessageDialogKind> for rfd::MessageLevel {
  fn from(kind: MessageDialogKind) -> Self {
    match kind {
      MessageDialogKind::Info => Self::Info,
      MessageDialogKind::Warning => Self::Warning,
      MessageDialogKind::Error => Self::Error,
    }
  }
}

/// Blocking interfaces for the dialog APIs.
///
/// The blocking APIs will block the current thread to execute instead of relying on callback closures,
/// which makes them easier to use.
///
/// **NOTE:** You cannot block the main thread when executing the dialog APIs, so you must use the [`crate::api::dialog`] methods instead.
/// Examples of main thread context are the [`crate::App::run`] closure and non-async commmands.
pub mod blocking {
  use super::{MessageDialogButtons, MessageDialogKind};
  use crate::{Runtime, Window};
  use std::path::{Path, PathBuf};
  use std::sync::mpsc::sync_channel;

  file_dialog_builder!();
  message_dialog_builder!();

  impl FileDialogBuilder {
    /// Shows the dialog to select a single file.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::blocking::FileDialogBuilder;
    /// #[tauri::command]
    /// fn my_command() {
    ///   let file_path = FileDialogBuilder::new().pick_file();
    ///   // do something with the optional file path here
    ///   // the file path is `None` if the user closed the dialog
    /// }
    /// ```
    pub fn pick_file(self) -> Option<PathBuf> {
      #[allow(clippy::let_and_return)]
      let response = run_dialog_sync!(self.0.pick_file());
      #[cfg(not(target_os = "linux"))]
      let response = response.map(|p| p.path().to_path_buf());
      response
    }

    /// Shows the dialog to select multiple files.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::blocking::FileDialogBuilder;
    /// #[tauri::command]
    /// fn my_command() {
    ///   let file_path = FileDialogBuilder::new().pick_files();
    ///   // do something with the optional file paths here
    ///   // the file paths value is `None` if the user closed the dialog
    /// }
    /// ```
    pub fn pick_files(self) -> Option<Vec<PathBuf>> {
      #[allow(clippy::let_and_return)]
      let response = run_dialog_sync!(self.0.pick_files());
      #[cfg(not(target_os = "linux"))]
      let response =
        response.map(|paths| paths.into_iter().map(|p| p.path().to_path_buf()).collect());
      response
    }

    /// Shows the dialog to select a single folder.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::blocking::FileDialogBuilder;
    /// #[tauri::command]
    /// fn my_command() {
    ///   let folder_path = FileDialogBuilder::new().pick_folder();
    ///   // do something with the optional folder path here
    ///   // the folder path is `None` if the user closed the dialog
    /// }
    /// ```
    pub fn pick_folder(self) -> Option<PathBuf> {
      #[allow(clippy::let_and_return)]
      let response = run_dialog_sync!(self.0.pick_folder());
      #[cfg(not(target_os = "linux"))]
      let response = response.map(|p| p.path().to_path_buf());
      response
    }

    /// Shows the dialog to save a file.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::blocking::FileDialogBuilder;
    /// #[tauri::command]
    /// fn my_command() {
    ///   let file_path = FileDialogBuilder::new().save_file();
    ///   // do something with the optional file path here
    ///   // the file path is `None` if the user closed the dialog
    /// }
    /// ```
    pub fn save_file(self) -> Option<PathBuf> {
      #[allow(clippy::let_and_return)]
      let response = run_dialog_sync!(self.0.save_file());
      #[cfg(not(target_os = "linux"))]
      let response = response.map(|p| p.path().to_path_buf());
      response
    }
  }

  impl MessageDialogBuilder {
    //// Shows a message dialog.
    ///
    /// - In `Ok` dialog, it will return `true` when `OK` was pressed.
    /// - In `OkCancel` dialog, it will return `true` when `OK` was pressed.
    /// - In `YesNo` dialog, it will return `true` when `Yes` was pressed.
    pub fn show(self) -> bool {
      let (tx, rx) = sync_channel(1);
      let f = move |response| {
        tx.send(response).unwrap();
      };
      run_dialog!(self.0.show(), f);
      rx.recv().unwrap()
    }
  }

  /// Displays a dialog with a message and an optional title with a "yes" and a "no" button and wait for it to be closed.
  ///
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::blocking::ask;
  /// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
  /// # let window = tauri::Manager::get_window(&app, "main").unwrap();
  /// let answer = ask(Some(&window), "Tauri", "Is Tauri awesome?");
  /// // do something with `answer`
  /// ```
  #[allow(unused_variables)]
  pub fn ask<R: Runtime>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
  ) -> bool {
    run_message_dialog(parent_window, title, message, rfd::MessageButtons::YesNo)
  }

  /// Displays a dialog with a message and an optional title with an "ok" and a "cancel" button and wait for it to be closed.
  ///
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::blocking::confirm;
  /// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
  /// # let window = tauri::Manager::get_window(&app, "main").unwrap();
  /// let answer = confirm(Some(&window), "Tauri", "Are you sure?");
  /// // do something with `answer`
  /// ```
  #[allow(unused_variables)]
  pub fn confirm<R: Runtime>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
  ) -> bool {
    run_message_dialog(parent_window, title, message, rfd::MessageButtons::OkCancel)
  }

  /// Displays a message dialog and wait for it to be closed.
  ///
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::blocking::message;
  /// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
  /// # let window = tauri::Manager::get_window(&app, "main").unwrap();
  /// message(Some(&window), "Tauri", "Tauri is awesome!");
  /// ```
  #[allow(unused_variables)]
  pub fn message<R: Runtime>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
  ) {
    let _ = run_message_dialog(parent_window, title, message, rfd::MessageButtons::Ok);
  }

  #[allow(unused_variables)]
  fn run_message_dialog<R: Runtime>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
    buttons: rfd::MessageButtons,
  ) -> bool {
    let (tx, rx) = sync_channel(1);
    super::nonblocking::run_message_dialog(
      parent_window,
      title,
      message,
      buttons,
      MessageDialogKind::Info,
      move |response| {
        tx.send(response).unwrap();
      },
    );
    rx.recv().unwrap()
  }
}

mod nonblocking {
  use super::{MessageDialogButtons, MessageDialogKind};
  use crate::{Runtime, Window};
  use std::path::{Path, PathBuf};

  file_dialog_builder!();
  message_dialog_builder!();

  impl FileDialogBuilder {
    /// Shows the dialog to select a single file.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// For usage in other contexts such as commands, prefer [`Self::pick_file`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::FileDialogBuilder;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|_app, _event| {
    ///     FileDialogBuilder::new().pick_file(|file_path| {
    ///       // do something with the optional file path here
    ///       // the file path is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    pub fn pick_file<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
      #[cfg(not(target_os = "linux"))]
      let f = |path: Option<rfd::FileHandle>| f(path.map(|p| p.path().to_path_buf()));
      run_file_dialog!(self.0.pick_file(), f)
    }

    /// Shows the dialog to select multiple files.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::FileDialogBuilder;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|_app, _event| {
    ///     FileDialogBuilder::new().pick_files(|file_paths| {
    ///       // do something with the optional file paths here
    ///       // the file paths value is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    pub fn pick_files<F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(self, f: F) {
      #[cfg(not(target_os = "linux"))]
      let f = |paths: Option<Vec<rfd::FileHandle>>| {
        f(paths.map(|list| list.into_iter().map(|p| p.path().to_path_buf()).collect()))
      };
      run_file_dialog!(self.0.pick_files(), f)
    }

    /// Shows the dialog to select a single folder.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::FileDialogBuilder;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|_app, _event| {
    ///     FileDialogBuilder::new().pick_folder(|folder_path| {
    ///       // do something with the optional folder path here
    ///       // the folder path is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    pub fn pick_folder<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
      #[cfg(not(target_os = "linux"))]
      let f = |path: Option<rfd::FileHandle>| f(path.map(|p| p.path().to_path_buf()));
      run_file_dialog!(self.0.pick_folder(), f)
    }

    /// Shows the dialog to save a file.
    ///
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri::api::dialog::FileDialogBuilder;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|_app, _event| {
    ///     FileDialogBuilder::new().save_file(|file_path| {
    ///       // do something with the optional file path here
    ///       // the file path is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    pub fn save_file<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
      #[cfg(not(target_os = "linux"))]
      let f = |path: Option<rfd::FileHandle>| f(path.map(|p| p.path().to_path_buf()));
      run_file_dialog!(self.0.save_file(), f)
    }
  }

  impl MessageDialogBuilder {
    /// Shows a message dialog:
    ///
    /// - In `Ok` dialog, it will call the closure with `true` when `OK` was pressed
    /// - In `OkCancel` dialog, it will call the closure with `true` when `OK` was pressed
    /// - In `YesNo` dialog, it will call the closure with `true` when `Yes` was pressed
    pub fn show<F: FnOnce(bool) + Send + 'static>(self, f: F) {
      run_dialog!(self.0.show(), f);
    }
  }

  /// Displays a non-blocking dialog with a message and an optional title with a "yes" and a "no" button.
  ///
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::ask;
  /// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
  /// # let window = tauri::Manager::get_window(&app, "main").unwrap();
  /// ask(Some(&window), "Tauri", "Is Tauri awesome?", |answer| {
  ///   // do something with `answer`
  /// });
  /// ```
  #[allow(unused_variables)]
  pub fn ask<R: Runtime, F: FnOnce(bool) + Send + 'static>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
    f: F,
  ) {
    run_message_dialog(
      parent_window,
      title,
      message,
      rfd::MessageButtons::YesNo,
      MessageDialogKind::Info,
      f,
    )
  }

  /// Displays a non-blocking dialog with a message and an optional title with an "ok" and a "cancel" button.
  ///
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::confirm;
  /// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
  /// # let window = tauri::Manager::get_window(&app, "main").unwrap();
  /// confirm(Some(&window), "Tauri", "Are you sure?", |answer| {
  ///   // do something with `answer`
  /// });
  /// ```
  #[allow(unused_variables)]
  pub fn confirm<R: Runtime, F: FnOnce(bool) + Send + 'static>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
    f: F,
  ) {
    run_message_dialog(
      parent_window,
      title,
      message,
      rfd::MessageButtons::OkCancel,
      MessageDialogKind::Info,
      f,
    )
  }

  /// Displays a non-blocking message dialog.
  ///
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::message;
  /// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
  /// # let window = tauri::Manager::get_window(&app, "main").unwrap();
  /// message(Some(&window), "Tauri", "Tauri is awesome!");
  /// ```
  #[allow(unused_variables)]
  pub fn message<R: Runtime>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
  ) {
    run_message_dialog(
      parent_window,
      title,
      message,
      rfd::MessageButtons::Ok,
      MessageDialogKind::Info,
      |_| {},
    )
  }

  #[allow(unused_variables)]
  pub(crate) fn run_message_dialog<R: Runtime, F: FnOnce(bool) + Send + 'static>(
    parent_window: Option<&Window<R>>,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
    buttons: rfd::MessageButtons,
    level: MessageDialogKind,
    f: F,
  ) {
    let title = title.as_ref().to_string();
    let message = message.as_ref().to_string();
    #[allow(unused_mut)]
    let mut builder = rfd::MessageDialog::new()
      .set_title(&title)
      .set_description(&message)
      .set_buttons(buttons)
      .set_level(level.into());

    #[cfg(any(windows, target_os = "macos"))]
    {
      if let Some(window) = parent_window {
        builder = builder.set_parent(window);
      }
    }

    run_dialog!(builder.show(), f)
  }
}
