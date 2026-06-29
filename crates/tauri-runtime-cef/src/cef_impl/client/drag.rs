// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use cef::*;
use tauri_runtime::{
  UserEvent,
  dpi::PhysicalPosition,
  webview::InitializationScript,
  window::{DragDropEvent, WindowId},
};
use url::Url;

use crate::runtime::{Message, RuntimeContext};

const DRAG_DROP_BRIDGE_PATH: &str = "/__tauri_cef_drag_drop__";

const DRAG_DROP_INIT_SCRIPT: &str = r#"
(() => {
  if (window.__TAURI_CEF_DRAG_DROP__) {
    return;
  }

  Object.defineProperty(window, "__TAURI_CEF_DRAG_DROP__", {
    value: true,
    configurable: false,
  });

  const PATH = "/__tauri_cef_drag_drop__";
  let entered = false;

  const position = (event) => ({
    x: event.clientX * window.devicePixelRatio,
    y: event.clientY * window.devicePixelRatio,
  });

  const send = (type, event) => {
    const pos = position(event);
    const url = new URL(PATH, window.location.href);
    url.searchParams.set("payload", JSON.stringify({ type, x: pos.x, y: pos.y }));
    fetch(url.href, {
      method: "GET",
      cache: "no-store",
      credentials: "omit",
    }).catch(() => {});
  };

  const listen = (eventName, handler) => {
    window.addEventListener(eventName, handler, { capture: true });
  };

  listen("dragenter", (event) => {
    if (!entered) {
      entered = true;
      send("enter", event);
    }
  });

  listen("dragover", (event) => {
    if (!entered) {
      entered = true;
      send("enter", event);
    }
    send("over", event);
  });

  listen("drop", (event) => {
    if (!entered) {
      send("enter", event);
    }
    entered = false;
    send("drop", event);
  });

  listen("dragleave", (event) => {
    const x = event.clientX;
    const y = event.clientY;
    if (entered && (x <= 0 || y <= 0 || x >= window.innerWidth || y >= window.innerHeight)) {
      entered = false;
      send("leave", event);
    }
  });
})();
"#;

pub(crate) fn drag_drop_initialization_script() -> InitializationScript {
  InitializationScript {
    script: DRAG_DROP_INIT_SCRIPT.to_string(),
    for_main_frame_only: false,
  }
}

#[derive(Default)]
pub(crate) struct DragDropState {
  pub(crate) paths: Option<Vec<PathBuf>>,
  pub(crate) native_entered: bool,
  pub(crate) entered: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragDropEventTarget {
  Window,
  Webview,
}

#[derive(Clone, serde::Deserialize)]
pub(crate) struct DragDropScriptEvent {
  #[serde(rename = "type")]
  pub(crate) kind: String,
  pub(crate) x: f64,
  pub(crate) y: f64,
}

fn collect_drag_data_paths(drag_data: &mut DragData) -> Vec<PathBuf> {
  let mut paths = CefStringList::new();
  if drag_data.file_paths(Some(&mut paths)) != 0 {
    let paths = paths
      .into_iter()
      .filter(|path| !path.is_empty())
      .map(PathBuf::from)
      .collect::<Vec<_>>();

    if !paths.is_empty() {
      return paths;
    }
  }

  let file_name = CefStringUtf16::from(&drag_data.file_name()).to_string();
  if file_name.is_empty() {
    Vec::new()
  } else {
    vec![PathBuf::from(file_name)]
  }
}

wrap_drag_handler! {
  pub struct TauriCefDragHandler {
    drag_drop_state: Arc<Mutex<DragDropState>>,
  }

  impl DragHandler {
    fn on_drag_enter(
      &self,
      _browser: Option<&mut Browser>,
      drag_data: Option<&mut DragData>,
      _mask: DragOperationsMask,
    ) -> ::std::os::raw::c_int {
      let mut state = self.drag_drop_state.lock().unwrap();
      state.entered = false;
      state.paths = drag_data
        .map(collect_drag_data_paths)
        .filter(|paths| !paths.is_empty());
      state.native_entered = state.paths.is_some();

      // Let Chromium continue with the drag operation so the injected script can
      // report over/drop/leave with accurate viewport positions.
      0
    }
  }
}

pub(crate) fn event_from_script_event(
  drag_drop_state: &Arc<Mutex<DragDropState>>,
  script_event: DragDropScriptEvent,
) -> Option<DragDropEvent> {
  let position = PhysicalPosition::new(script_event.x, script_event.y);
  let mut state = drag_drop_state.lock().unwrap();
  if !state.native_entered {
    return None;
  }

  match script_event.kind.as_str() {
    "enter" => {
      if state.entered {
        return None;
      }

      let paths = state.paths.clone()?;
      state.entered = true;
      Some(DragDropEvent::Enter { paths, position })
    }
    "over" => state.entered.then_some(DragDropEvent::Over { position }),
    "drop" => {
      let paths = state.entered.then(|| state.paths.take()).flatten();
      state.entered = false;
      state.native_entered = false;
      paths.map(|paths| DragDropEvent::Drop { paths, position })
    }
    "leave" => {
      state.native_entered = false;
      state.paths = None;

      if state.entered {
        state.entered = false;
        Some(DragDropEvent::Leave)
      } else {
        None
      }
    }
    _ => None,
  }
}

wrap_resource_request_handler! {
  pub(crate) struct WebDragDropResourceRequestHandler<T: UserEvent> {
    context: RuntimeContext<T>,
    window_id: WindowId,
    webview_id: u32,
    drag_drop_event_target: DragDropEventTarget,
    drag_drop_handler_enabled: bool,
    drag_drop_state: Arc<Mutex<DragDropState>>,
  }

  impl ResourceRequestHandler {
    fn on_before_resource_load(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      request: Option<&mut Request>,
      _callback: Option<&mut Callback>,
    ) -> ReturnValue {
      if self.drag_drop_handler_enabled
        && let Some(request) = request
      {
        let url = CefString::from(&request.url()).to_string();
        if let Ok(url) = Url::parse(&url)
          && url.path() == DRAG_DROP_BRIDGE_PATH
        {
          if let Some(payload) = url
            .query_pairs()
            .find_map(|(key, value)| (key == "payload").then(|| value.into_owned()))
            && let Ok(event) = serde_json::from_str::<DragDropScriptEvent>(&payload)
          {
            let _ = self.context.send_message(Message::DragDropScriptEvent {
              window_id: self.window_id,
              webview_id: self.webview_id,
              target: self.drag_drop_event_target,
              drag_drop_state: self.drag_drop_state.clone(),
              event,
            });
          }

          return sys::cef_return_value_t::RV_CANCEL.into();
        }
      }

      sys::cef_return_value_t::RV_CONTINUE.into()
    }
  }
}
