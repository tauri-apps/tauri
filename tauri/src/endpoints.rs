mod cli;
mod dialog;
#[cfg(event)]
mod event;
#[allow(unused_imports)]
mod file_system;
#[cfg(http_request)]
mod http;
mod internal;
#[cfg(notification)]
mod notification;
mod shell;
mod window;

use crate::{app::Context, ApplicationDispatcherExt};

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct ModuleDto {
  module: String,
  callback: Option<String>,
  error: Option<String>,
  message: Value,
}

#[derive(Deserialize)]
#[serde(tag = "module")]
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
}

impl Module {
  async fn run<D: ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
    context: &Context,
  ) -> crate::Result<()> {
    match self {
      Self::Fs(cmd) => cmd.run(webview_manager).await,
      Self::Window(cmd) => cmd.run(webview_manager).await?,
      Self::Shell(cmd) => cmd.run(webview_manager).await,
      Self::Event(cmd) => cmd.run(webview_manager).await?,
      Self::Internal(cmd) => cmd.run(webview_manager).await?,
      Self::Dialog(cmd) => cmd.run(webview_manager).await?,
      Self::Cli(cmd) => cmd.run(webview_manager, context).await,
      Self::Notification(cmd) => cmd.run(webview_manager, context).await?,
      Self::Http(cmd) => cmd.run(webview_manager).await,
    }
    Ok(())
  }
}

pub(crate) async fn handle<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  arg: &str,
  context: &Context,
) -> crate::Result<()> {
  match serde_json::from_str::<ModuleDto>(arg) {
    Err(e) => {
      if e.to_string().contains("missing field `module`") {
        Err(crate::Error::UnknownApi(Some(e)))
      } else {
        Err(e.into())
      }
    }
    Ok(mut module_dto) => {
      if let Value::Object(ref mut obj) = module_dto.message {
        obj.insert("module".to_string(), Value::String(module_dto.module));
        if let Some(callback) = module_dto.callback {
          obj.insert("callback".to_string(), Value::String(callback));
        }
        if let Some(error) = module_dto.error {
          obj.insert("error".to_string(), Value::String(error));
        }
      }
      let module: Module = serde_json::from_str(&module_dto.message.to_string())?;
      module.run(webview_manager, context).await?;
      Ok(())
    }
  }
}

#[allow(dead_code)]
fn api_error<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  error_fn: String,
  message: &str,
) {
  let reject_code = tauri_api::rpc::format_callback(error_fn, message);
  if let Ok(dispatcher) = webview_manager.current_webview() {
    dispatcher.eval(&reject_code);
  }
}

#[allow(dead_code)]
fn allowlist_error<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  error_fn: String,
  allowlist_key: &str,
) {
  api_error(
    webview_manager,
    error_fn,
    &format!(
      "{}' not on the allowlist (https://tauri.studio/docs/api/config#tauri.allowlist)",
      allowlist_key
    ),
  )
}

#[allow(dead_code)]
fn throw_allowlist_error<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  allowlist_key: &str,
) {
  let reject_code = format!(
    r#"throw new Error("'{}' not on the allowlist")"#,
    allowlist_key
  );
  if let Ok(dispatcher) = webview_manager.current_webview() {
    dispatcher.eval(&reject_code);
  }
}
