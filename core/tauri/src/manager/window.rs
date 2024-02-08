// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{HashMap, HashSet},
  fmt,
  path::PathBuf,
  sync::{Arc, Mutex, MutexGuard},
};

use serde::Serialize;
use tauri_runtime::{
  window::WindowBuilder,
  window::{
    dpi::{PhysicalPosition, PhysicalSize},
    DetachedWindow, FileDropEvent, PendingWindow,
  },
};

use crate::{
  app::GlobalWindowEventListener, sealed::ManagerBase, AppHandle, EventLoopMessage, EventTarget,
  Icon, Manager, Runtime, Scopes, Window, WindowEvent,
};

use super::AppManager;

const WINDOW_RESIZED_EVENT: &str = "tauri://resize";
const WINDOW_MOVED_EVENT: &str = "tauri://move";
const WINDOW_CLOSE_REQUESTED_EVENT: &str = "tauri://close-requested";
const WINDOW_DESTROYED_EVENT: &str = "tauri://destroyed";
const WINDOW_FOCUS_EVENT: &str = "tauri://focus";
const WINDOW_BLUR_EVENT: &str = "tauri://blur";
const WINDOW_SCALE_FACTOR_CHANGED_EVENT: &str = "tauri://scale-change";
const WINDOW_THEME_CHANGED: &str = "tauri://theme-changed";
const WINDOW_FILE_DROP_EVENT: &str = "tauri://file-drop";
const WINDOW_FILE_DROP_HOVER_EVENT: &str = "tauri://file-drop-hover";
const WINDOW_FILE_DROP_CANCELLED_EVENT: &str = "tauri://file-drop-cancelled";

pub struct WindowManager<R: Runtime> {
  pub windows: Mutex<HashMap<String, Window<R>>>,
  pub default_icon: Option<Icon>,
  /// Window event listeners to all windows.
  pub event_listeners: Arc<Vec<GlobalWindowEventListener<R>>>,
}

impl<R: Runtime> fmt::Debug for WindowManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowManager")
      .field("default_window_icon", &self.default_icon)
      .finish()
  }
}

impl<R: Runtime> WindowManager<R> {
  /// Get a locked handle to the windows.
  pub(crate) fn windows_lock(&self) -> MutexGuard<'_, HashMap<String, Window<R>>> {
    self.windows.lock().expect("poisoned window manager")
  }

  pub fn prepare_window(
    &self,
    mut pending: PendingWindow<EventLoopMessage, R>,
  ) -> crate::Result<PendingWindow<EventLoopMessage, R>> {
    if self.windows_lock().contains_key(&pending.label) {
      return Err(crate::Error::WindowLabelAlreadyExists(pending.label));
    }

    if !pending.window_builder.has_icon() {
      if let Some(default_window_icon) = self.default_icon.clone() {
        pending.window_builder = pending
          .window_builder
          .icon(default_window_icon.try_into()?)?;
      }
    }

    Ok(pending)
  }

  pub(crate) fn attach_window(
    &self,
    app_handle: AppHandle<R>,
    window: DetachedWindow<EventLoopMessage, R>,
    multiwebview: bool,
    #[cfg(desktop)] menu: Option<crate::window::WindowMenu<R>>,
  ) -> Window<R> {
    let window = Window::new(
      app_handle.manager.clone(),
      window,
      app_handle,
      #[cfg(desktop)]
      menu,
      multiwebview,
    );

    let window_ = window.clone();
    let window_event_listeners = self.event_listeners.clone();
    let manager = window.manager.clone();
    window.on_window_event(move |event| {
      let _ = on_window_event(&window_, &manager, event);
      for handler in window_event_listeners.iter() {
        handler(&window_, event);
      }
    });

    // insert the window into our manager
    {
      self
        .windows_lock()
        .insert(window.label().to_string(), window.clone());
    }

    // let plugins know that a new window has been added to the manager
    let manager = window.manager.clone();
    let window_ = window.clone();
    // run on main thread so the plugin store doesn't dead lock with the event loop handler in App
    let _ = window.run_on_main_thread(move || {
      manager
        .plugins
        .lock()
        .expect("poisoned plugin store")
        .window_created(window_);
    });

    window
  }

  pub fn labels(&self) -> HashSet<String> {
    self.windows_lock().keys().cloned().collect()
  }
}

#[derive(Serialize, Clone)]
struct FileDropPayload<'a> {
  paths: &'a Vec<PathBuf>,
  position: &'a PhysicalPosition<f64>,
}

fn on_window_event<R: Runtime>(
  window: &Window<R>,
  manager: &AppManager<R>,
  event: &WindowEvent,
) -> crate::Result<()> {
  match event {
    WindowEvent::Resized(size) => window.emit(WINDOW_RESIZED_EVENT, size)?,
    WindowEvent::Moved(position) => window.emit(WINDOW_MOVED_EVENT, position)?,
    WindowEvent::CloseRequested { api } => {
      let listeners = window.manager().listeners();
      let has_js_listener =
        listeners.has_js_listener(WINDOW_CLOSE_REQUESTED_EVENT, |target| match target {
          EventTarget::Window { label } | EventTarget::WebviewWindow { label } => {
            label == window.label()
          }
          _ => false,
        });
      if has_js_listener {
        api.prevent_close();
      }
      window.emit(WINDOW_CLOSE_REQUESTED_EVENT, ())?;
    }
    WindowEvent::Destroyed => {
      window.emit(WINDOW_DESTROYED_EVENT, ())?;
      let label = window.label();
      let webviews_map = manager.webview.webviews_lock();
      let webviews = webviews_map.values();
      for webview in webviews {
        webview.eval(&format!(
          r#"(function () {{ const metadata = window.__TAURI_INTERNALS__.metadata; if (metadata != null) {{ metadata.windows = window.__TAURI_INTERNALS__.metadata.windows.filter(w => w.label !== "{label}"); }} }})()"#,
        ))?;
      }
    }
    WindowEvent::Focused(focused) => window.emit(
      if *focused {
        WINDOW_FOCUS_EVENT
      } else {
        WINDOW_BLUR_EVENT
      },
      (),
    )?,
    WindowEvent::ScaleFactorChanged {
      scale_factor,
      new_inner_size,
      ..
    } => window.emit(
      WINDOW_SCALE_FACTOR_CHANGED_EVENT,
      ScaleFactorChanged {
        scale_factor: *scale_factor,
        size: *new_inner_size,
      },
    )?,
    WindowEvent::FileDrop(event) => match event {
      FileDropEvent::Hovered { paths, position } => {
        let payload = FileDropPayload { paths, position };
        window.emit(WINDOW_FILE_DROP_HOVER_EVENT, payload)?
      }
      FileDropEvent::Dropped { paths, position } => {
        let scopes = window.state::<Scopes>();
        for path in paths {
          if path.is_file() {
            let _ = scopes.allow_file(path);
          } else {
            let _ = scopes.allow_directory(path, false);
          }
        }
        let payload = FileDropPayload { paths, position };
        window.emit(WINDOW_FILE_DROP_EVENT, payload)?
      }
      FileDropEvent::Cancelled => window.emit(WINDOW_FILE_DROP_CANCELLED_EVENT, ())?,
      _ => unimplemented!(),
    },
    WindowEvent::ThemeChanged(theme) => window.emit(WINDOW_THEME_CHANGED, theme.to_string())?,
  }
  Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScaleFactorChanged {
  scale_factor: f64,
  size: PhysicalSize<u32>,
}
