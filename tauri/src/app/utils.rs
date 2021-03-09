use std::{io::Read, sync::Arc};

use crate::{
  api::{
    assets::{AssetFetch, Assets},
    config::WindowUrl,
    rpc::{format_callback, format_callback_result},
  },
  app::{Icon, InvokeResponse},
  ApplicationExt, WebviewBuilderExt,
};

use super::{
  webview::{CustomProtocol, WebviewBuilderExtPrivate, WebviewRpcHandler},
  App, Context, PageLoadPayload, RpcRequest, Webview, WebviewManager,
};

use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize)]
struct Message {
  #[serde(rename = "__tauriModule")]
  tauri_module: Option<String>,
  callback: String,
  error: String,
  #[serde(rename = "mainThread", default)]
  main_thread: bool,
  #[serde(flatten)]
  inner: JsonValue,
}

// setup content for dev-server
#[cfg(dev)]
pub(super) fn get_url(context: &Context) -> String {
  let config = &context.config;
  if config.build.dev_path.starts_with("http") {
    config.build.dev_path.clone()
  } else {
    format!(
      "data:text/html;base64,{}",
      base64::encode(
        context
          .assets
          .get(&Assets::format_key("index.html"), AssetFetch::Decompress)
          .ok_or_else(|| crate::Error::AssetNotFound("index.html".to_string()))
          .and_then(|(read, _)| {
            read
              .bytes()
              .collect::<Result<Vec<u8>, _>>()
              .map_err(Into::into)
          })
          .expect("Unable to find `index.html` under your devPath folder")
      )
    )
  }
}

#[cfg(custom_protocol)]
pub(super) fn get_url(_: &Context) -> String {
  // Custom protocol doesn't require any setup, so just return URL
  "tauri://index.html".into()
}

// spawn an updater process.
#[cfg(feature = "updater")]
#[allow(dead_code)]
pub(super) fn spawn_updater() {
  std::thread::spawn(|| {
    tauri_api::command::spawn_relative_command(
      "updater".to_string(),
      Vec::new(),
      std::process::Stdio::inherit(),
    )
    .expect("Unable to spawn relative command");
  });
}

pub(super) fn initialization_script(
  plugin_initialization_script: &str,
  tauri_script: &str,
) -> String {
  format!(
    r#"
      {tauri_initialization_script}
      {event_initialization_script}
      if (window.rpc) {{
        window.__TAURI__.invoke("__initialized", {{ url: window.location.href }})
      }} else {{
        window.addEventListener('DOMContentLoaded', function () {{
          window.__TAURI__.invoke("__initialized", {{ url: window.location.href }})
        }})
      }}
      {plugin_initialization_script}
    "#,
    tauri_initialization_script = tauri_script,
    event_initialization_script = event_initialization_script(),
    plugin_initialization_script = plugin_initialization_script
  )
}

fn event_initialization_script() -> String {
  return format!(
    "
      window['{queue}'] = [];
      window['{fn}'] = function (payload, salt, ignoreQueue) {{
      const listeners = (window['{listeners}'] && window['{listeners}'][payload.type]) || []
      if (!ignoreQueue && listeners.length === 0) {{
        window['{queue}'].push({{
          payload: payload,
          salt: salt
        }})
      }}

      if (listeners.length > 0) {{
        window.__TAURI__.invoke({{
          __tauriModule: 'Internal',
          message: {{
            cmd: 'validateSalt',
            salt: salt
          }}
        }}).then(function (flag) {{
          if (flag) {{
            for (let i = listeners.length - 1; i >= 0; i--) {{
              const listener = listeners[i]
              if (listener.once)
                listeners.splice(i, 1)
              listener.handler(payload)
            }}
          }}
        }})
      }}
    }}
    ",
    fn = crate::event::emit_function_name(),
    queue = crate::event::event_queue_object_name(),
    listeners = crate::event::event_listeners_object_name()
  );
}

pub(super) type BuiltWebview<A> = (
  <A as ApplicationExt>::WebviewBuilder,
  Option<WebviewRpcHandler<<A as ApplicationExt>::Dispatcher>>,
  Option<CustomProtocol>,
);

// build the webview.
pub(super) fn build_webview<A: ApplicationExt + 'static>(
  application: Arc<App<A>>,
  webview: Webview<A>,
  webview_manager: &WebviewManager<A>,
  content_url: &str,
  window_labels: &[String],
  plugin_initialization_script: &str,
  context: &Context,
) -> crate::Result<BuiltWebview<A>> {
  let webview_url = match &webview.url {
    WindowUrl::App => content_url.to_string(),
    WindowUrl::Custom(url) => url.to_string(),
  };

  let is_local = match webview.url {
    WindowUrl::App => true,
    WindowUrl::Custom(url) => &url[0..8] == "tauri://",
  };
  let (webview_builder, rpc_handler, custom_protocol) = if is_local {
    let mut webview_builder = webview.builder.url(webview_url)
        .initialization_script(&initialization_script(plugin_initialization_script, &context.tauri_script))
        .initialization_script(&format!(
          r#"
              window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
              window.__TAURI__.__currentWindow = {{ label: "{current_window_label}" }}
            "#,
          window_labels_array =
            serde_json::to_string(&window_labels).unwrap(),
          current_window_label = webview.label,
        ));

    if !webview_builder.has_icon() {
      if let Some(default_window_icon) = &context.default_window_icon {
        webview_builder = webview_builder.icon(Icon::Raw(default_window_icon.to_vec()))?;
      }
    }

    let webview_manager_ = webview_manager.clone();
    let rpc_handler: Box<dyn Fn(<A as ApplicationExt>::Dispatcher, RpcRequest) + Send> =
      Box::new(move |_, request: RpcRequest| {
        let command = request.command.clone();
        let arg = request
          .params
          .unwrap()
          .as_array_mut()
          .unwrap()
          .first_mut()
          .unwrap_or(&mut JsonValue::Null)
          .take();
        let webview_manager = webview_manager_.clone();
        match serde_json::from_value::<Message>(arg) {
          Ok(message) => {
            let application = application.clone();
            let callback = message.callback.to_string();
            let error = message.error.to_string();

            if message.main_thread {
              crate::async_runtime::block_on(async move {
                execute_promise(
                  &webview_manager,
                  on_message(
                    application,
                    webview_manager.clone(),
                    command.clone(),
                    message,
                  ),
                  callback,
                  error,
                )
                .await;
              });
            } else {
              crate::async_runtime::spawn(async move {
                execute_promise(
                  &webview_manager,
                  on_message(application, webview_manager.clone(), command, message),
                  callback,
                  error,
                )
                .await;
              });
            }
          }
          Err(e) => {
            if let Ok(dispatcher) =
              crate::async_runtime::block_on(webview_manager.current_webview())
            {
              let error: crate::Error = e.into();
              let _ = dispatcher.eval(&format!(
                r#"console.error({})"#,
                JsonValue::String(error.to_string())
              ));
            }
          }
        }
      });
    let assets = context.assets;
    let custom_protocol = CustomProtocol {
      name: "tauri".into(),
      handler: Box::new(move |path| {
        let mut path = path.to_string().replace("tauri://", "");
        if path.ends_with('/') {
          path.pop();
        }
        let path =
          if let Some((first, components)) = path.split('/').collect::<Vec<&str>>().split_first() {
            match components.len() {
              0 => first.to_string(),
              _ => components.join("/"),
            }
          } else {
            path
          };

        let asset_response = assets
          .get(&Assets::format_key(&path), AssetFetch::Decompress)
          .ok_or(crate::Error::AssetNotFound(path))
          .and_then(|(read, _)| {
            read
              .bytes()
              .collect::<Result<Vec<u8>, _>>()
              .map_err(Into::into)
          });
        match asset_response {
          Ok(asset) => Ok(asset),
          Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("{:?}", e); // TODO log::error!
            Err(e)
          }
        }
      }),
    };
    (webview_builder, Some(rpc_handler), Some(custom_protocol))
  } else {
    (webview.builder.url(webview_url), None, None)
  };

  Ok((webview_builder, rpc_handler, custom_protocol))
}

/// Asynchronously executes the given task
/// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
///
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
async fn execute_promise<
  A: ApplicationExt + 'static,
  F: futures::Future<Output = crate::Result<InvokeResponse>> + Send + 'static,
>(
  webview_manager: &crate::WebviewManager<A>,
  task: F,
  success_callback: String,
  error_callback: String,
) {
  let callback_string = match format_callback_result(
    task
      .await
      .and_then(|response| response.json)
      .map_err(|err| err.to_string()),
    success_callback,
    error_callback.clone(),
  ) {
    Ok(callback_string) => callback_string,
    Err(e) => format_callback(error_callback, e.to_string()),
  };
  if let Ok(dispatcher) = webview_manager.current_webview().await {
    let _ = dispatcher.eval(callback_string.as_str());
  }
}

async fn on_message<A: ApplicationExt + 'static>(
  application: Arc<App<A>>,
  webview_manager: WebviewManager<A>,
  command: String,
  message: Message,
) -> crate::Result<InvokeResponse> {
  if &command == "__initialized" {
    let payload: PageLoadPayload = serde_json::from_value(message.inner)?;
    application
      .run_on_page_load(&webview_manager, payload.clone())
      .await;
    crate::plugin::on_page_load(A::plugin_store(), &webview_manager, payload).await;
    Ok(().into())
  } else {
    let response = if let Some(module) = &message.tauri_module {
      crate::endpoints::handle(
        &webview_manager,
        module.to_string(),
        message.inner,
        &application.context,
      )
      .await
    } else {
      let mut response = match application
        .run_invoke_handler(&webview_manager, command.clone(), &message.inner)
        .await
      {
        Ok(value) => {
          if let Some(value) = value {
            Ok(value)
          } else {
            Err(crate::Error::UnknownApi(None))
          }
        }
        Err(e) => Err(e),
      };
      if let Err(crate::Error::UnknownApi(_)) = response {
        match crate::plugin::extend_api(
          A::plugin_store(),
          &webview_manager,
          command,
          &message.inner,
        )
        .await
        {
          Ok(value) => {
            // If value is None, that means that no plugin matched the command
            // and the UnknownApi error should be sent to the webview
            // Otherwise, send the result of plugin handler
            if value.is_some() {
              response = Ok(value.into());
            }
          }
          Err(e) => {
            // A plugin handler was found but it failed
            response = Err(e);
          }
        }
      }
      response
    };
    response
  }
}

#[cfg(test)]
mod test {
  use crate::{Context, FromTauriContext};

  #[derive(FromTauriContext)]
  #[config_path = "test/fixture/src-tauri/tauri.conf.json"]
  struct TauriContext;

  #[test]
  fn check_get_url() {
    let context = Context::new::<TauriContext>().unwrap();
    let res = super::get_url(&context);
    #[cfg(custom_protocol)]
    assert!(res == "tauri://index.html");

    #[cfg(dev)]
    {
      let config = &context.config;
      assert_eq!(res, config.build.dev_path);
    }
  }
}
