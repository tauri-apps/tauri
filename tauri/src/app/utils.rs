#[cfg(dev)]
use std::io::Read;
use std::sync::Arc;

#[cfg(dev)]
use crate::api::assets::{AssetFetch, Assets};

use crate::{
  api::{
    config::WindowUrl,
    rpc::{format_callback, format_callback_result},
  },
  app::{Icon, InvokeResponse},
  ApplicationExt, WebviewBuilderExt,
};

use super::{
  webview::{Callback, WebviewBuilderExtPrivate},
  App, Context, Webview, WebviewManager,
};
#[cfg(embedded_server)]
use crate::api::tcp::{get_available_port, port_is_available};

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
#[allow(clippy::unnecessary_wraps)]
pub(super) fn get_url(context: &Context) -> crate::Result<String> {
  let config = &context.config;
  if config.build.dev_path.starts_with("http") {
    #[cfg(windows)]
    {
      let exempt_output = std::process::Command::new("CheckNetIsolation")
        .args(&vec!["LoopbackExempt", "-s"])
        .output()
        .expect("failed to read LoopbackExempt -s");

      if !exempt_output.status.success() {
        panic!("Failed to execute CheckNetIsolation LoopbackExempt -s");
      }

      let output_str = String::from_utf8_lossy(&exempt_output.stdout).to_lowercase();
      if !output_str.contains("win32webviewhost_cw5n1h2txyewy") {
        println!("Running Loopback command");
        runas::Command::new("powershell")
          .args(&[
            "CheckNetIsolation LoopbackExempt -a -n=\"Microsoft.Win32WebViewHost_cw5n1h2txyewy\"",
          ])
          .force_prompt(true)
          .status()
          .expect("failed to run Loopback command");
      }
    }
    Ok(config.build.dev_path.clone())
  } else {
    Ok(format!(
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
    ))
  }
}

// setup content for embedded server
#[cfg(embedded_server)]
pub(super) fn get_url(context: &Context) -> crate::Result<String> {
  let (port, valid) = setup_port(&context);
  if valid {
    Ok(setup_server_url(port, &context))
  } else {
    Err(crate::Error::PortNotAvailable(port))
  }
}

// get the port for the embedded server
#[cfg(embedded_server)]
#[allow(dead_code)]
fn setup_port(context: &Context) -> (String, bool) {
  let config = &context.config;
  match config.tauri.embedded_server.port {
    tauri_api::config::Port::Random => match get_available_port() {
      Some(available_port) => (available_port.to_string(), true),
      None => ("0".to_string(), false),
    },
    tauri_api::config::Port::Value(port) => {
      let port_valid = port_is_available(port);
      (port.to_string(), port_valid)
    }
  }
}

// setup the server url for embedded server
#[cfg(embedded_server)]
#[allow(dead_code)]
fn setup_server_url(port: String, context: &Context) -> String {
  let config = &context.config;
  let mut url = format!("{}:{}", config.tauri.embedded_server.host, port);
  if !url.starts_with("http") {
    url = format!("http://{}", url);
  }
  url
}

// spawn the embedded server
#[cfg(embedded_server)]
pub(super) fn spawn_server(server_url: String, context: &Context) {
  let assets = context.assets;
  let public_path = context.config.tauri.embedded_server.public_path.clone();
  std::thread::spawn(move || {
    let server = tiny_http::Server::http(server_url.replace("http://", "").replace("https://", ""))
      .expect("Unable to spawn server");
    for request in server.incoming_requests() {
      let url = request.url().replace(&server_url, "");
      let url = match url.as_str() {
        "/" => "/index.html",
        url => {
          if url.starts_with(&public_path) {
            &url[public_path.len() - 1..]
          } else {
            eprintln!(
              "found url not matching public path.\nurl: {}\npublic path: {}",
              url, public_path
            );
            url
          }
        }
      }
      .to_string();
      request
        .respond(crate::server::asset_response(&url, assets))
        .expect("unable to setup response");
    }
  });
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
      if (window.__TAURI_INVOKE_HANDLER__) {{
        window.__TAURI__.invoke({{ cmd: "__initialized" }})
      }} else {{
        window.addEventListener('DOMContentLoaded', function () {{
          window.__TAURI__.invoke({{ cmd: "__initialized" }})
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
  Vec<Callback<<A as ApplicationExt>::Dispatcher>>,
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
  // TODO let debug = cfg!(debug_assertions);
  let webview_url = match &webview.url {
    WindowUrl::App => content_url.to_string(),
    WindowUrl::Custom(url) => url.to_string(),
  };

  let (webview_builder, callbacks) = if webview.url == WindowUrl::App {
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
    let tauri_invoke_handler = crate::Callback::<A::Dispatcher> {
      name: "__TAURI_INVOKE_HANDLER__".to_string(),
      function: Box::new(move |_, arg| {
        let arg = arg.into_iter().next().unwrap_or(JsonValue::Null);
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
                  on_message(application, webview_manager.clone(), message),
                  callback,
                  error,
                )
                .await;
              });
            } else {
              crate::async_runtime::spawn(async move {
                execute_promise(
                  &webview_manager,
                  on_message(application, webview_manager.clone(), message),
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
      }),
    };
    (webview_builder, vec![tauri_invoke_handler])
  } else {
    (webview.builder.url(webview_url), Vec::new())
  };

  Ok((webview_builder, callbacks))
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
  message: Message,
) -> crate::Result<InvokeResponse> {
  if message.inner == serde_json::json!({ "cmd":"__initialized" }) {
    application.run_setup(&webview_manager).await;
    crate::plugin::ready(A::plugin_store(), &webview_manager).await;
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
        .run_invoke_handler(&webview_manager, &message.inner)
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
        response = crate::plugin::extend_api(A::plugin_store(), &webview_manager, &message.inner)
          .await
          .map(|value| value.into());
      }
      response
    };
    response
  }
}

#[cfg(test)]
mod test {
  use crate::{Context, FromTauriContext};
  use proptest::prelude::*;

  #[derive(FromTauriContext)]
  #[config_path = "test/fixture/src-tauri/tauri.conf.json"]
  struct TauriContext;

  #[test]
  fn check_get_url() {
    let context = Context::new::<TauriContext>().unwrap();
    let res = super::get_url(&context);

    #[cfg(embedded_server)]
    match res {
      Ok(u) => assert!(u.contains("http://")),
      _ => panic!("setup content failed"),
    }

    #[cfg(dev)]
    {
      let config = &context.config;
      match res {
        Ok(u) => assert_eq!(u, config.build.dev_path),
        _ => panic!("setup content failed"),
      }
    }
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[cfg(embedded_server)]
    #[test]
    fn check_server_url(port in (any::<u32>().prop_map(|v| v.to_string()))) {
      let p = port.clone();
      let context = Context::new::<TauriContext>().unwrap();

      let url = super::setup_server_url(port, &context);
      assert!(url.contains(&p));
    }
  }
}
