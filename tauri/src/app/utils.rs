use crate::{
  api::{assets::Assets, config::WindowUrl},
  app::Icon,
  ApplicationExt, WebviewBuilderExt,
};

use super::{
  webview::{CustomProtocol, WebviewBuilderExtPrivate, WebviewRpcHandler},
  App, Context, InvokeMessage, InvokePayload, PageLoadPayload, RpcRequest, Webview, WebviewManager,
};

use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  sync::{Arc, Mutex},
};

// setup content for dev-server
#[cfg(dev)]
pub(super) fn get_url(context: &Context) -> String {
  let config = &context.config;
  if config.build.dev_path.starts_with("http") {
    config.build.dev_path.clone()
  } else {
    let path = "index.html";
    format!(
      "data:text/html;base64,{}",
      base64::encode(
        context
          .assets
          .get(&path)
          .ok_or_else(|| crate::Error::AssetNotFound(path.to_string()))
          .map(Cow::into_owned)
          .expect("Unable to find `index.html` under your devPath folder")
      )
    )
  }
}

#[cfg(custom_protocol)]
pub(super) fn get_url(context: &Context) -> String {
  // Custom protocol doesn't require any setup, so just return URL
  format!("tauri://{}", context.config.tauri.bundle.identifier)
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
  with_global_tauri: bool,
) -> String {
  format!(
    r#"
      {bundle_script}
      {core_script}
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
    core_script = include_str!("../../scripts/core.js"),
    bundle_script = if with_global_tauri {
      include_str!("../../scripts/bundle.js")
    } else {
      ""
    },
    event_initialization_script = event_initialization_script(),
    plugin_initialization_script = plugin_initialization_script
  )
}

fn event_initialization_script() -> String {
  return format!(
    "
      window['{queue}'] = [];
      window['{fn}'] = function (eventData, salt, ignoreQueue) {{
      const listeners = (window['{listeners}'] && window['{listeners}'][eventData.event]) || []
      if (!ignoreQueue && listeners.length === 0) {{
        window['{queue}'].push({{
          eventData: eventData,
          salt: salt
        }})
      }}

      if (listeners.length > 0) {{
        window.__TAURI__.invoke('tauri', {{
          __tauriModule: 'Internal',
          message: {{
            cmd: 'validateSalt',
            salt: salt
          }}
        }}).then(function (flag) {{
          if (flag) {{
            for (let i = listeners.length - 1; i >= 0; i--) {{
              const listener = listeners[i]
              eventData.id = listener.id
              listener.handler(eventData)
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
  application: Arc<Mutex<App<A>>>,
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
        .initialization_script(&initialization_script(plugin_initialization_script, context.config.build.with_global_tauri))
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
        match serde_json::from_value::<InvokePayload>(arg) {
          Ok(message) => {
            let _ = on_message(application.clone(), webview_manager, command, message);
          }
          Err(e) => {
            if let Ok(dispatcher) = webview_manager.current_webview() {
              let error: crate::Error = e.into();
              let _ = dispatcher.eval(&format!(
                r#"console.error({})"#,
                JsonValue::String(error.to_string())
              ));
            }
          }
        }
      });
    let bundle_identifier = context.config.tauri.bundle.identifier.clone();
    #[cfg(debug_assertions)]
    let dist_dir = std::path::PathBuf::from(context.config.build.dist_dir.clone());
    #[cfg(not(debug_assertions))]
    let assets = context.assets;
    let custom_protocol = CustomProtocol {
      name: "tauri".into(),
      handler: Box::new(move |path| {
        let mut path = path
          .to_string()
          .replace(&format!("tauri://{}", bundle_identifier), "");
        if path.ends_with('/') {
          path.pop();
        }
        let path = if path.is_empty() {
          // if the url is `tauri://${appId}`, we should load `index.html`
          "index.html".to_string()
        } else {
          // skip leading `/`
          path.chars().skip(1).collect::<String>()
        };

        // In development builds, resolve, read and directly serve assets in the configured dist folder.
        #[cfg(debug_assertions)]
        {
          dist_dir
            .canonicalize()
            .or_else(|_| Err(crate::Error::AssetNotFound(path.clone())))
            .and_then(|pathbuf| {
              pathbuf
                .join(path.clone())
                .canonicalize()
                .or_else(|_| Err(crate::Error::AssetNotFound(path.clone())))
                .and_then(|pathbuf| {

                  if pathbuf.is_file() && pathbuf.starts_with(&dist_dir) {
                    match std::fs::read(pathbuf) {
                      Ok(asset) => return Ok(asset),
                      Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("Error reading asset from dist: {:?}", e); // TODO log::error!
                      }
                    }
                  }

                  Err(crate::Error::AssetNotFound(path))

                })
            })
        }

        // In release builds, fetch + serve decompressed embedded assets.
        #[cfg(not(debug_assertions))]
        {
          assets
            .get(&path)
            .ok_or(crate::Error::AssetNotFound(path))
            .map(Cow::into_owned)
        }
      }),
    };
    (webview_builder, Some(rpc_handler), Some(custom_protocol))
  } else {
    (webview.builder.url(webview_url), None, None)
  };

  Ok((webview_builder, rpc_handler, custom_protocol))
}

fn on_message<A: ApplicationExt + 'static>(
  application: Arc<Mutex<App<A>>>,
  webview_manager: WebviewManager<A>,
  command: String,
  payload: InvokePayload,
) -> crate::Result<()> {
  let message = InvokeMessage::new(webview_manager.clone(), command.to_string(), payload);
  if &command == "__initialized" {
    let payload: PageLoadPayload = serde_json::from_value(message.payload())?;
    application
      .lock()
      .unwrap()
      .run_on_page_load(&webview_manager, payload.clone());
    crate::plugin::on_page_load(A::plugin_store(), &webview_manager, payload);
  } else if let Some(module) = &message.payload.tauri_module {
    let module = module.to_string();
    crate::endpoints::handle(
      &webview_manager,
      module,
      message,
      &application.lock().unwrap().context,
    );
  } else if command.starts_with("plugin:") {
    crate::plugin::extend_api(A::plugin_store(), &webview_manager, command, message);
  } else {
    application
      .lock()
      .unwrap()
      .run_invoke_handler(&webview_manager, message);
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use crate::{generate_context, Context};

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json");
    let context = Context::new(context);
    let res = super::get_url(&context);
    #[cfg(custom_protocol)]
    assert!(res == "tauri://studio.tauri.example");

    #[cfg(dev)]
    {
      let config = &context.config;
      assert_eq!(res, config.build.dev_path);
    }
  }
}
