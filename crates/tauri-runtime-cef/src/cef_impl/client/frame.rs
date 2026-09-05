// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use cef::*;

use crate::{FrameEventHandler, FrameEventKind, frame::emit_frame_event};

wrap_frame_handler! {
  pub struct TauriCefFrameHandler {
    handler: Option<Arc<FrameEventHandler>>,
  }

  impl FrameHandler {
    fn on_frame_created(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>) {
      emit_frame_event(&self.handler, browser, frame, FrameEventKind::Created);
    }

    fn on_frame_attached(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>, _reattached: ::std::os::raw::c_int) {
      emit_frame_event(&self.handler, browser, frame, FrameEventKind::Attached);
    }

    fn on_frame_detached(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>) {
      emit_frame_event(&self.handler, browser, frame, FrameEventKind::Detached);
    }

    fn on_frame_destroyed(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>) {
      emit_frame_event(&self.handler, browser, frame, FrameEventKind::Destroyed);
    }

    fn on_main_frame_changed(&self, browser: Option<&mut Browser>, _old_frame: Option<&mut Frame>, new_frame: Option<&mut Frame>) {
      emit_frame_event(&self.handler, browser, new_frame, FrameEventKind::MainFrameChanged);
    }
  }
}
