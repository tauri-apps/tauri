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

#[cfg(all(
  test,
  any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  )
))]
mod tests {
  use super::*;
  use std::sync::{LazyLock, mpsc};

  use crate::external_message_pump::CefExternalPump;
  use crate::runtime::RuntimeContext;
  use winit::event_loop::EventLoopProxy as WinitEventLoopProxy;

  /// The context the handler tests need: a live winit proxy to wake, a channel the
  /// deep-link path can send to, and the rest of the fields default-initialized the
  /// way `CefRuntime::init` initializes them.
  ///
  /// Building a winit event loop on a test thread needs `with_any_thread`, which is
  /// what `Runtime::new_any_thread` uses for the same reason. The `GDK_BACKEND`
  /// override matches the runtime's own: the CEF integration works in X11 window
  /// handles, so an inherited Wayland-only backend must not reach the loop.
  fn context() -> RuntimeContext<()> {
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::gtk4::EventLoopBuilderExtGtk4;

    // One event loop per process: winit-gtk4 refuses a second construction, and
    // the libtest default harness runs every test on its own thread. The loop
    // itself is never run here, only used as the proxy's owner.
    static EVENT_LOOP_PROXY: LazyLock<WinitEventLoopProxy> = LazyLock::new(|| {
      // SAFETY: test-only; nothing has been spawned yet that could be reading
      // the environment.
      unsafe { std::env::set_var("GDK_BACKEND", "x11") };
      let mut event_loop_builder = EventLoopBuilder::default();
      event_loop_builder.with_any_thread(true);
      let event_loop = event_loop_builder.build().expect("event loop");
      event_loop.create_proxy()
    });
    let proxy = EVENT_LOOP_PROXY.clone();

    RuntimeContext {
      sender: mpsc::channel().0,
      proxy,
      main_thread_id: std::thread::current().id(),
      next_window_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_event_id: Default::default(),
      next_webview_event_id: Default::default(),
      current_dispatch: Default::default(),
      app_wide_theme: Default::default(),
      cef_pump: CefExternalPump::new(),
      cache_path: Arc::new(std::env::temp_dir()),
      profile_preferences: Default::default(),
      content_settings: Default::default(),
      certificate_errors: Default::default(),
      devtools_allowed: false,
    }
  }

  /// CEF refuses every entry point but `initialize` (and `api_hash`) before the
  /// library is initialized, crashing on the first one otherwise — including
  /// `command_line_create`. The settings and application are all `None`, which CEF
  /// documents as a valid minimal initialization and is what the `cef` crate's own
  /// tests do.
  fn ensure_cef_initialized() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
      let _ = initialize(None, None, None, std::ptr::null_mut());
      let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    });
  }

  /// A `CommandLine` as CEF hands one to `on_before_child_process_launch`: created
  /// by the same constructor CEF uses, with the program set so the line looks like a
  /// launched process's.
  fn child_command_line() -> CommandLine {
    command_line_create().expect("a new CefCommandLine")
  }

  fn switch_value(command_line: &CommandLine, name: &str) -> String {
    CefString::from(&command_line.switch_value(Some(&CefString::from(name)))).to_string()
  }

  /// The whole point of the feature: switches configured through
  /// [`Cef::child_process_command_line_arg`] land on the command line CEF is about
  /// to launch a child process with. Invoked the way CEF invokes it, through the
  /// trait method the generated vtable forwards to.
  #[test]
  fn child_process_switches_reach_the_child_command_line() {
    ensure_cef_initialized();

    let handler = TauriCefBrowserProcessHandler::new(
      context(),
      Arc::new(AtomicBool::new(false)),
      Vec::new(),
      // What an application enabling WebGPU on the ANGLE Vulkan backend asks for,
      // as in the issue this fixes.
      vec![
        ("use-angle".to_string(), Some("vulkan".to_string())),
        (
          "--enable-features".to_string(),
          Some("Vulkan,VulkanFromANGLE".to_string()),
        ),
      ],
    );

    let mut command_line = child_command_line();
    ImplBrowserProcessHandler::on_before_child_process_launch(&handler, Some(&mut command_line));

    assert_eq!(
      command_line.has_switch(Some(&CefString::from("use-angle"))),
      1,
      "the GPU process reads its ANGLE backend from this switch"
    );
    assert_eq!(
      switch_value(&command_line, "use-angle"),
      "vulkan",
      "a switch with a value must keep the value"
    );
    assert_eq!(
      command_line.has_switch(Some(&CefString::from("enable-features"))),
      1,
      "Chromium strips the `--` prefix when storing the switch"
    );
    assert_eq!(
      switch_value(&command_line, "enable-features"),
      "Vulkan,VulkanFromANGLE",
      "the `--`-prefixed spelling reaches the same switch"
    );
  }

  /// A bare name with no value and no `-` prefix is a positional argument, not a
  /// switch — the invariant `append_command_line_args` documents.
  #[test]
  fn a_name_without_a_prefix_is_an_argument_not_a_switch() {
    ensure_cef_initialized();

    let handler = TauriCefBrowserProcessHandler::new(
      context(),
      Arc::new(AtomicBool::new(false)),
      Vec::new(),
      vec![("https://example.com".to_string(), None)],
    );

    let mut command_line = child_command_line();
    ImplBrowserProcessHandler::on_before_child_process_launch(&handler, Some(&mut command_line));

    let mut arguments = CefStringList::new();
    command_line.arguments(Some(&mut arguments));
    assert_eq!(
      arguments.into_iter().collect::<Vec<_>>(),
      ["https://example.com"],
      "a name without the `--` prefix is appended as a positional argument"
    );
    assert_eq!(
      command_line.has_switches(),
      0,
      "an argument is not a switch, so the line carries none"
    );
  }

  /// With nothing configured the callback must leave the child's command line
  /// exactly as Chromium built it: the hook exists to add switches, not to rewrite
  /// the ones Chromium already forwarded.
  #[test]
  fn an_empty_configuration_leaves_the_command_line_alone() {
    ensure_cef_initialized();

    let handler = TauriCefBrowserProcessHandler::new(
      context(),
      Arc::new(AtomicBool::new(false)),
      Vec::new(),
      Vec::new(),
    );

    let mut command_line = child_command_line();
    command_line.append_switch_with_value(
      Some(&CefString::from("type")),
      Some(&CefString::from("gpu-process")),
    );
    let before = command_line.command_line_string();
    let before = CefString::from(&before).to_string();

    ImplBrowserProcessHandler::on_before_child_process_launch(&handler, Some(&mut command_line));

    let after = command_line.command_line_string();
    let after = CefString::from(&after).to_string();
    assert_eq!(
      before, after,
      "nothing configured means nothing appended, nothing dropped"
    );
  }
}
