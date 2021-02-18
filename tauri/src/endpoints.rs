mod cli;
mod dialog;
mod event;
#[allow(unused_imports)]
mod file_system;
#[cfg(global_shortcut)]
mod global_shortcut;
#[cfg(http_request)]
mod http;
mod internal;
#[cfg(notification)]
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
      Self::Fs(cmd) => cmd.run().await,
      Self::Window(cmd) => cmd.run(webview_manager).await,
      Self::Shell(cmd) => cmd.run().await,
      Self::Event(cmd) => cmd.run(webview_manager).await,
      Self::Internal(cmd) => cmd.run().await,
      Self::Dialog(cmd) => cmd.run().await,
      Self::Cli(cmd) => cmd.run(context).await,
      Self::Notification(cmd) => cmd.run(context).await,
      Self::Http(cmd) => cmd.run().await,
      Self::GlobalShortcut(cmd) => cmd.run(webview_manager).await,
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
