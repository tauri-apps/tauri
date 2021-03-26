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

use crate::{app::Context, ApplicationExt, InvokeMessage};

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
  fn run<A: ApplicationExt + 'static>(
    self,
    webview_manager: crate::WebviewManager<A>,
    message: InvokeMessage<A>,
    context: &Context,
  ) {
    match self {
      Self::Fs(cmd) => message.respond_async(async move { cmd.run() }),
      Self::Window(cmd) => message.respond_async(async move { cmd.run(&webview_manager).await }),
      Self::Shell(cmd) => message.respond_async(async move { cmd.run() }),
      Self::Event(cmd) => message.respond_async(async move { cmd.run(&webview_manager) }),
      Self::Internal(cmd) => message.respond_async(async move { cmd.run() }),
      Self::Dialog(cmd) => message.respond_async(async move { cmd.run() }),
      Self::Cli(cmd) => {
        if let Some(cli_config) = context.config.tauri.cli.clone() {
          message.respond_async(async move { cmd.run(&cli_config) })
        }
      }
      Self::Notification(cmd) => {
        let identifier = context.config.tauri.bundle.identifier.clone();
        message.respond_async(async move { cmd.run(identifier) })
      }
      Self::Http(cmd) => message.respond_async(async move { cmd.run().await }),
      Self::GlobalShortcut(cmd) => message.respond_async(async move { cmd.run(&webview_manager) }),
    }
  }
}

pub(crate) fn handle<A: ApplicationExt + 'static>(
  webview_manager: &crate::WebviewManager<A>,
  module: String,
  message: InvokeMessage<A>,
  context: &Context,
) {
  let mut payload = message.payload();
  if let JsonValue::Object(ref mut obj) = payload {
    obj.insert("module".to_string(), JsonValue::String(module));
  }
  match serde_json::from_value::<Module>(payload) {
    Ok(module) => module.run(webview_manager.clone(), message, context),
    Err(e) => message.respond_async(async move { Err(e.into()) }),
  }
}
