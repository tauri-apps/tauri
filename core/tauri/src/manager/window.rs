// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
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
  dpi::{PhysicalPosition, PhysicalSize},
  window::WindowBuilder,
  window::{DetachedWindow, DragDropEvent, PendingWindow},
};

use crate::{
  app::GlobalWindowEventListener, image::Image, sealed::ManagerBase, AppHandle, EventLoopMessage,
  EventTarget, Manager, Runtime, Scopes, Window, WindowEvent,
};

const WINDOW_RESIZED_EVENT: &str = "tauri://resize";
const WINDOW_MOVED_EVENT: &str = "tauri://move";
const WINDOW_CLOSE_REQUESTED_EVENT: &str = "tauri://close-requested";
const WINDOW_DESTROYED_EVENT: &str = "tauri://destroyed";
const WINDOW_FOCUS_EVENT: &str = "tauri://focus";
const WINDOW_BLUR_EVENT: &str = "tauri://blur";
const WINDOW_SCALE_FACTOR_CHANGED_EVENT: &str = "tauri://scale-change";
const WINDOW_THEME_CHANGED: &str = "tauri://theme-changed";
pub const DRAG_EVENT: &str = "tauri://drag";
pub const DROP_EVENT: &str = "tauri://drop";
pub const DROP_HOVER_EVENT: &str = "tauri://drop-over";
pub const DROP_CANCELLED_EVENT: &str = "tauri://drag-cancelled";

pub struct WindowManager<R: Runtime> {
  pub windows: Mutex<HashMap<String, Window<R>>>,
  pub default_icon: Option<Image<'static>>,
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
        pending.window_builder = pending.window_builder.icon(default_window_icon.into())?;
      }
    }

    Ok(pending)
  }

  pub(crate) fn attach_window(
    &self,
    app_handle: AppHandle<R>,
    window: DetachedWindow<EventLoopMessage, R>,
    #[cfg(desktop)] menu: Option<crate::window::WindowMenu<R>>,
  ) -> Window<R> {
    let window = Window::new(
      app_handle.manager.clone(),
      window,
      app_handle,
      #[cfg(desktop)]
      menu,
    );

    let window_ = window.clone();
    let window_event_listeners = self.event_listeners.clone();
    window.on_window_event(move |event| {
      let _ = on_window_event(&window_, event);
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

impl<R: Runtime> Window<R> {
  /// Emits event to [`EventTarget::Window`] and [`EventTarget::WebviewWindow`]
  fn emit_to_window<S: Serialize + Clone>(&self, event: &str, payload: S) -> crate::Result<()> {
    let window_label = self.label();
    self.emit_filter(event, payload, |target| match target {
      EventTarget::Window { label } | EventTarget::WebviewWindow { label } => label == window_label,
      _ => false,
    })
  }

  /// Checks whether has js listener for [`EventTarget::Window`] or [`EventTarget::WebviewWindow`]
  fn has_js_listener(&self, event: &str) -> bool {
    let window_label = self.label();
    let listeners = self.manager().listeners();
    listeners.has_js_listener(event, |target| match target {
      EventTarget::Window { label } | EventTarget::WebviewWindow { label } => label == window_label,
      _ => false,
    })
  }
}

#[derive(Serialize, Clone)]
pub struct DragDropPayload<'a> {
  pub paths: &'a Vec<PathBuf>,
  pub position: &'a PhysicalPosition<f64>,
}

#[derive(Serialize, Clone)]
pub struct DragOverPayload<'a> {
  pub position: &'a PhysicalPosition<f64>,
}

fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) -> crate::Result<()> {
  match event {
    WindowEvent::Resized(size) => window.emit_to_window(WINDOW_RESIZED_EVENT, size)?,
    WindowEvent::Moved(position) => window.emit_to_window(WINDOW_MOVED_EVENT, position)?,
    WindowEvent::CloseRequested { api } => {
      if window.has_js_listener(WINDOW_CLOSE_REQUESTED_EVENT) {
        api.prevent_close();
      }
      window.emit_to_window(WINDOW_CLOSE_REQUESTED_EVENT, ())?;
    }
    WindowEvent::Destroyed => {
      window.emit_to_window(WINDOW_DESTROYED_EVENT, ())?;
      let label = window.label();

      if let Ok(webview_labels_array) = serde_json::to_string(&window.manager().webview.labels()) {
        let _ = window.manager().webview.eval_script_all(format!(
          r#"(function () {{ const metadata = window.__TAURI_INTERNALS__.metadata; if (metadata != null) {{ metadata.windows = window.__TAURI_INTERNALS__.metadata.windows.filter(w => w.label !== "{label}"); metadata.webviews = {webview_labels_array}.map(function (label) {{ return {{ label: label }} }}) }} }})()"#,
        ));
      }
    }
    WindowEvent::Focused(focused) => window.emit_to_window(
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
    } => window.emit_to_window(
      WINDOW_SCALE_FACTOR_CHANGED_EVENT,
      ScaleFactorChanged {
        scale_factor: *scale_factor,
        size: *new_inner_size,
      },
    )?,
    WindowEvent::DragDrop(event) => match event {
      DragDropEvent::Dragged { paths, position } => {
        let payload = DragDropPayload { paths, position };

        if window.is_webview_window() {
          window.emit_to(EventTarget::labeled(window.label()), DRAG_EVENT, payload)?
        } else {
          window.emit_to_window(DRAG_EVENT, payload)?
        }
      }
      DragDropEvent::DragOver { position } => {
        let payload = DragOverPayload { position };
        if window.is_webview_window() {
          window.emit_to(
            EventTarget::labeled(window.label()),
            DROP_HOVER_EVENT,
            payload,
          )?
        } else {
          window.emit_to_window(DROP_HOVER_EVENT, payload)?
        }
      }
      DragDropEvent::Dropped { paths, position } => {
        let scopes = window.state::<Scopes>();
        for path in paths {
          if path.is_file() {
            let _ = scopes.allow_file(path);
          } else {
            let _ = scopes.allow_directory(path, true);
          }
        }
        let payload = DragDropPayload { paths, position };

        if window.is_webview_window() {
          window.emit_to(EventTarget::labeled(window.label()), DROP_EVENT, payload)?
        } else {
          window.emit_to_window(DROP_EVENT, payload)?
        }
      }
      DragDropEvent::Cancelled => {
        if window.is_webview_window() {
          window.emit_to(
            EventTarget::labeled(window.label()),
            DROP_CANCELLED_EVENT,
            (),
          )?
        } else {
          window.emit_to_window(DROP_CANCELLED_EVENT, ())?
        }
      }
      _ => unimplemented!(),
    },
    WindowEvent::ThemeChanged(theme) => {
      window.emit_to_window(WINDOW_THEME_CHANGED, theme.to_string())?
    }
  }
  Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScaleFactorChanged {
  scale_factor: f64,
  size: PhysicalSize<u32>,
}
