// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use cef::*;
use tauri_runtime::UserEvent;

use crate::runtime::{Message, RuntimeContext};

wrap_browser_process_handler! {
  pub(crate) struct TauriCefBrowserProcessHandler<T: UserEvent> {
    context: RuntimeContext<T>,
    context_initialized: Arc<AtomicBool>,
    deep_link_schemes: Vec<String>,
    // Switches appended to every child process's command line, from
    // `Cef::child_process_command_line_args`. See `on_before_child_process_launch`.
    child_process_command_line_args: Vec<(String, Option<String>)>,
  }

  impl BrowserProcessHandler {
    fn on_context_initialized(&self) {
      self.context_initialized.store(true, Ordering::SeqCst);
      self.context.proxy.wake_up();
    }

    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
      self.context.cef_pump.on_schedule_message_pump_work(delay_ms);
      self.context.proxy.wake_up();
    }

    fn on_before_child_process_launch(&self, command_line: Option<&mut CommandLine>) {
      // A child's command line already carries the switches Chromium chose to
      // propagate; the application's own are the ones that go missing — an
      // ANGLE backend or a feature flag the GPU process must see for WebGPU to
      // find an adapter, say. Appending them here, in the browser process, is
      // the supported way to get them there.
      if let Some(command_line) = command_line {
        crate::runtime::append_command_line_args(
          command_line,
          &self.child_process_command_line_args,
        );
      }
    }

    fn on_already_running_app_relaunch(
      &self,
      command_line: Option<&mut CommandLine>,
      _current_directory: Option<&CefString>,
    ) -> std::os::raw::c_int {
      let Some(command_line) = command_line else {
        return 0;
      };
      let mut list = CefStringList::new();
      command_line.arguments(Some(&mut list));
      let args: Vec<String> = list.into_iter().collect();
      if let Some(first_arg) = args.first()
        && let Ok(url) = url::Url::parse(first_arg)
      {
        let scheme = url.scheme().to_string();
        if self.deep_link_schemes.iter().any(|s| s == &scheme) {
          let _ = self.context.sender.send(Message::Opened(vec![url]));
          self.context.proxy.wake_up();
          return 1;
        }
      }
      // TODO: add event
      1
    }
  }
}
