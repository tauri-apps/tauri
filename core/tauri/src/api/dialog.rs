// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to display dialog.

#[cfg(any(dialog_open, dialog_save))]
use std::path::{Path, PathBuf};
use std::sync::mpsc::sync_channel;

use crate::{Runtime, Window};

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

macro_rules! run_dialog_sync {
  ($e:expr) => {{
    let (tx, rx) = sync_channel(0);
    let cb = move |response| {
      tx.send(response).unwrap();
    };
    run_dialog!($e, cb);
    rx.recv().unwrap()
  }};
}

/// The file dialog builder.
///
/// Constructs file picker dialogs that can select single/multiple files or directories.
#[cfg(any(dialog_open, dialog_save))]
#[derive(Debug, Default)]
pub struct FileDialogBuilder(rfd::FileDialog);

#[cfg(any(dialog_open, dialog_save))]
impl FileDialogBuilder {
  /// Gets the default file dialog builder.
  pub fn new() -> Self {
    Default::default()
  }

  /// Add file extension filter. Takes in the name of the filter, and list of extensions
  pub fn add_filter(mut self, name: impl AsRef<str>, extensions: &[&str]) -> Self {
    self.0 = self.0.add_filter(name.as_ref(), extensions);
    self
  }

  /// Set starting directory of the dialog.
  pub fn set_directory<P: AsRef<Path>>(mut self, directory: P) -> Self {
    self.0 = self.0.set_directory(directory);
    self
  }

  /// Set starting file name of the dialog.
  pub fn set_file_name(mut self, file_name: &str) -> Self {
    self.0 = self.0.set_file_name(file_name);
    self
  }

  /// Sets the parent window of the dialog.
  pub fn set_parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
    self.0 = self.0.set_parent(parent);
    self
  }

  /// Set the title of the dialog.
  pub fn set_title(mut self, title: &str) -> Self {
    self.0 = self.0.set_title(title);
    self
  }

  /// Shows the dialog to select a single file.
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// For usage in other contexts such as commands, prefer [`Self::pick_file`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// tauri::Builder::default()
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("failed to build tauri app")
  ///   .run(|_app, _event| {
  ///     FileDialogBuilder::new().pick_file_nonblocking(|file_path| {
  ///       // do something with the optional file path here
  ///       // the file path is `None` if the user closed the dialog
  ///     })
  ///   })
  /// ```
  pub fn pick_file_nonblocking<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.pick_file(), f)
  }

  /// Shows the dialog to select multiple files.
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// For usage in other contexts such as commands, prefer [`Self::pick_files`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// tauri::Builder::default()
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("failed to build tauri app")
  ///   .run(|_app, _event| {
  ///     FileDialogBuilder::new().pick_files_nonblocking(|file_paths| {
  ///       // do something with the optional file paths here
  ///       // the file paths value is `None` if the user closed the dialog
  ///     })
  ///   })
  /// ```
  pub fn pick_files_nonblocking<F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.pick_files(), f)
  }

  /// Shows the dialog to select a single folder.
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// For usage in other contexts such as commands, prefer [`Self::pick_folder`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// tauri::Builder::default()
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("failed to build tauri app")
  ///   .run(|_app, _event| {
  ///     FileDialogBuilder::new().pick_folder_nonblocking(|folder_path| {
  ///       // do something with the optional folder path here
  ///       // the folder path is `None` if the user closed the dialog
  ///     })
  ///   })
  /// ```
  pub fn pick_folder_nonblocking<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.pick_folder(), f)
  }

  /// Shows the dialog to save a file.
  /// This is not a blocking operation,
  /// and should be used when running on the main thread to avoid deadlocks with the event loop.
  ///
  /// For usage in other contexts such as commands, prefer [`Self::save_file`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// tauri::Builder::default()
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("failed to build tauri app")
  ///   .run(|_app, _event| {
  ///     FileDialogBuilder::new().save_file_nonblocking(|file_path| {
  ///       // do something with the optional file path here
  ///       // the file path is `None` if the user closed the dialog
  ///     })
  ///   })
  /// ```
  pub fn save_file_nonblocking<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.save_file(), f)
  }

  /// Shows the dialog to select a single file.
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// For usage on the main thread, see [`Self::pick_file_nonblocking`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// #[tauri::command]
  /// fn my_command() {
  ///   let file_path = FileDialogBuilder::new().pick_file();
  ///   // do something with the optional file path here
  ///   // the file path is `None` if the user closed the dialog
  /// }
  /// ```
  pub fn pick_file(self) -> Option<PathBuf> {
    run_dialog_sync!(self.0.pick_file())
  }

  /// Shows the dialog to select multiple files.
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// For usage on the main thread, see [`Self::pick_files_nonblocking`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// #[tauri::command]
  /// fn my_command() {
  ///   let file_path = FileDialogBuilder::new().pick_files();
  ///   // do something with the optional file paths here
  ///   // the file paths value is `None` if the user closed the dialog
  /// }
  /// ```
  pub fn pick_files(self) -> Option<Vec<PathBuf>> {
    run_dialog_sync!(self.0.pick_files())
  }

  /// Shows the dialog to select a single folder.
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// For usage on the main thread, see [`Self::pick_folder_nonblocking()`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// #[tauri::command]
  /// fn my_command() {
  ///   let folder_path = FileDialogBuilder::new().pick_folder();
  ///   // do something with the optional folder path here
  ///   // the folder path is `None` if the user closed the dialog
  /// }
  /// ```
  pub fn pick_folder(self) -> Option<PathBuf> {
    run_dialog_sync!(self.0.pick_folder())
  }

  /// Shows the dialog to save a file.
  /// This is a blocking operation,
  /// and should *NOT* be used when running on the main thread context.
  ///
  /// For usage on the main thread, see [`Self::save_file_nonblocking()`].
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use tauri::api::dialog::FileDialogBuilder;
  /// #[tauri::command]
  /// fn my_command() {
  ///   let file_path = FileDialogBuilder::new().save_file();
  ///   // do something with the optional file path here
  ///   // the file path is `None` if the user closed the dialog
  /// }
  /// ```
  pub fn save_file(self) -> Option<PathBuf> {
    run_dialog_sync!(self.0.save_file())
  }
}

/// Displays a non-blocking dialog with a message and an optional title with a "yes" and a "no" button.
///
/// This is not a blocking operation,
/// and should be used when running on the main thread to avoid deadlocks with the event loop.
///
/// For usage in other contexts such as commands, prefer [`ask()`].
///
/// # Example
///
/// ```rust,no_run
/// use tauri::api::dialog::ask_nonblocking;
/// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
/// # let window = tauri::Manager::get_window(&app, "main").unwrap();
/// ask_nonblocking(Some(&window), "Tauri", "Is Tauri awesome?", |answer| {
///   // do something with `answer`
/// });
/// ```
#[allow(unused_variables)]
pub fn ask_nonblocking<R: Runtime, F: FnOnce(bool) + Send + 'static>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
  f: F,
) {
  run_message_dialog_nonblocking(parent_window, title, message, rfd::MessageButtons::YesNo, f)
}

/// Displays a non-blocking dialog with a message and an optional title with an "ok" and a "cancel" button.
///
/// This is not a blocking operation,
/// and should be used when running on the main thread to avoid deadlocks with the event loop.
///
/// For usage in other contexts such as commands, prefer [`confirm()`].
///
/// # Example
///
/// ```rust,no_run
/// use tauri::api::dialog::confirm_nonblocking;
/// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
/// # let window = tauri::Manager::get_window(&app, "main").unwrap();
/// confirm_nonblocking(Some(&window), "Tauri", "Are you sure?", |answer| {
///   // do something with `answer`
/// });
/// ```
#[allow(unused_variables)]
pub fn confirm_nonblocking<R: Runtime, F: FnOnce(bool) + Send + 'static>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
  f: F,
) {
  run_message_dialog_nonblocking(
    parent_window,
    title,
    message,
    rfd::MessageButtons::OkCancel,
    f,
  )
}

/// Displays a non-blocking message dialog.
///
/// This is not a blocking operation,
/// and should be used when running on the main thread to avoid deadlocks with the event loop.
///
/// For usage in other contexts such as commands, prefer [`message()`].
///
/// # Example
///
/// ```rust,no_run
/// use tauri::api::dialog::message_nonblocking;
/// # let app = tauri::Builder::default().build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json")).unwrap();
/// # let window = tauri::Manager::get_window(&app, "main").unwrap();
/// message_nonblocking(Some(&window), "Tauri", "Tauri is awesome!");
/// ```
#[allow(unused_variables)]
pub fn message_nonblocking<R: Runtime>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
) {
  run_message_dialog_nonblocking(
    parent_window,
    title,
    message,
    rfd::MessageButtons::Ok,
    |_| {},
  )
}

/// Displays a dialog with a message and an optional title with a "yes" and a "no" button and wait for it to be closed.
///
/// This is a blocking operation,
/// and should *NOT* be used when running on the main thread context.
///
/// For usage on the main thread, see [`ask_nonblocking()`].
///
/// # Example
///
/// ```rust,no_run
/// use tauri::api::dialog::ask;
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
/// For usage on the main thread, see [`confirm_nonblocking()`].
///
/// # Example
///
/// ```rust,no_run
/// use tauri::api::dialog::confirm;
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
/// For usage on the main thread, see [`message_nonblocking()`].
///
/// # Example
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
  run_message_dialog_nonblocking(parent_window, title, message, buttons, move |response| {
    tx.send(response).unwrap();
  });
  rx.recv().unwrap()
}

#[allow(unused_variables)]
fn run_message_dialog_nonblocking<R: Runtime, F: FnOnce(bool) + Send + 'static>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
  buttons: rfd::MessageButtons,
  f: F,
) {
  let title = title.as_ref().to_string();
  let message = message.as_ref().to_string();
  #[allow(unused_mut)]
  let mut builder = rfd::MessageDialog::new()
    .set_title(&title)
    .set_description(&message)
    .set_buttons(buttons)
    .set_level(rfd::MessageLevel::Info);

  #[cfg(any(windows, target_os = "macos"))]
  {
    if let Some(window) = parent_window {
      builder = builder.set_parent(window);
    }
  }

  run_dialog!(builder.show(), f)
}
