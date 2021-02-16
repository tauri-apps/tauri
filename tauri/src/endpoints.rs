mod cli;
mod dialog;
#[cfg(event)]
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

use crate::{app::Context, ApplicationDispatcherExt};

use serde::{Deserialize, Serialize};
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
  async fn run<D: ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
    context: &Context,
  ) -> crate::Result<JsonValue> {
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

pub(crate) async fn handle<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  module: String,
  mut arg: JsonValue,
  context: &Context,
) -> crate::Result<JsonValue> {
  if let JsonValue::Object(ref mut obj) = arg {
    obj.insert("module".to_string(), JsonValue::String(module));
  }
  let module: Module = serde_json::from_value(arg)?;
  let response = module.run(webview_manager, context).await?;
  Ok(serde_json::to_value(response)?)
}

pub(crate) fn to_value(t: impl Serialize) -> crate::Result<JsonValue> {
  serde_json::to_value(t).map_err(Into::into)
}
