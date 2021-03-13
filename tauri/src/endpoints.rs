mod cli;
mod dialog;
mod event;
#[allow(unused_imports)]
mod file_system;
mod global_shortcut;
mod http;
mod internal;
mod notification;
mod shell;
mod window;

use crate::{
  app::{Context, InvokeResponse},
  ApplicationExt,
};

use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Deserialize)]
#[serde(tag = "module", content = "message")]
enum Module {
  Fs(file_system::Cmd),
  Window(window::Cmd),
  Shell(shell::Cmd),
  Event(event::Cmd),
  Internal(internal::Cmd),
  Dialog(dialog::Cmd),
  Cli(cli::Cmd),
  Notification(notification::Cmd),
  Http(http::Cmd),
  GlobalShortcut(global_shortcut::Cmd),
}

impl Module {
  async fn run<A: ApplicationExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<A>,
    context: &Context,
  ) -> crate::Result<InvokeResponse> {
    match self {
      Self::Fs(cmd) => cmd.run(),
      Self::Window(cmd) => cmd.run(webview_manager).await,
      Self::Shell(cmd) => cmd.run(),
      Self::Event(cmd) => cmd.run(webview_manager),
      Self::Internal(cmd) => cmd.run(),
      Self::Dialog(cmd) => cmd.run(),
      Self::Cli(cmd) => cmd.run(context),
      Self::Notification(cmd) => cmd.run(context),
      Self::Http(cmd) => crate::async_runtime::block_on(cmd.run()),
      Self::GlobalShortcut(cmd) => cmd.run(webview_manager),
    }
  }
}

pub(crate) async fn handle<A: ApplicationExt + 'static>(
  webview_manager: &crate::WebviewManager<A>,
  module: String,
  mut arg: JsonValue,
  context: &Context,
) -> crate::Result<InvokeResponse> {
  if let JsonValue::Object(ref mut obj) = arg {
    obj.insert("module".to_string(), JsonValue::String(module));
  }
  let module: Module = serde_json::from_value(arg)?;
  module.run(webview_manager, context).await
}
