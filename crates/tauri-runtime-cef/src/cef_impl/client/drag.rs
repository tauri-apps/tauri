// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use cef::*;

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
  pub(super) struct TauriCefDragHandler {
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
