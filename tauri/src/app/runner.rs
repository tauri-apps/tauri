#[cfg(dev)]
use std::io::Read;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

#[cfg(dev)]
use crate::api::assets::{AssetFetch, Assets};

use crate::{ApplicationDispatcherExt, ApplicationExt, WebviewBuilderExt, WindowBuilderExt};

use super::App;
#[cfg(embedded_server)]
use crate::api::tcp::{get_available_port, port_is_available};
use crate::app::Context;

#[allow(dead_code)]
enum Content<T> {
  Html(T),
  Url(T),
}

/// Main entry point for running the Webview
pub(crate) fn run<A: ApplicationExt + 'static>(application: App<A>) -> crate::Result<()> {
  let plugin_config = application.context.config.plugins.clone();
  crate::async_runtime::block_on(async move {
    crate::plugin::initialize(A::plugin_store(), plugin_config).await
  })?;

  // setup the content using the config struct depending on the compile target
  let main_content = setup_content(&application.context)?;

  #[cfg(embedded_server)]
  {
    // setup the server url for the embedded-server
    let server_url = if let Content::Url(url) = &main_content {
      String::from(url)
    } else {
      String::from("")
    };

    // spawn the embedded server on our server url
    #[cfg(embedded_server)]
    spawn_server(server_url, &application.context);
  }

  let splashscreen_content = if application.splashscreen_html().is_some() {
    Some(Content::Html(
      application
        .splashscreen_html()
        .expect("failed to get splashscreen_html")
        .to_string(),
    ))
  } else {
    None
  };

  // build the webview
  let (webview_application, mut dispatcher) =
    build_webview(application, main_content, splashscreen_content)?;

  crate::async_runtime::spawn(async move {
    crate::plugin::created(A::plugin_store(), &mut dispatcher).await
  });

  // spin up the updater process
  #[cfg(feature = "updater")]
  spawn_updater();

  // run the webview
  webview_application.run();

  Ok(())
}

// setup content for dev-server
#[cfg(dev)]
fn setup_content(context: &Context) -> crate::Result<Content<String>> {
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
    Ok(Content::Url(config.build.dev_path.clone()))
  } else {
    Ok(Content::Html(format!(
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
    )))
  }
}

// setup content for embedded server
#[cfg(embedded_server)]
fn setup_content(context: &Context) -> crate::Result<Content<String>> {
  let (port, valid) = setup_port(&context);
  if valid {
    let url = setup_server_url(port, &context);
    Ok(Content::Url(url))
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
fn spawn_server(server_url: String, context: &Context) {
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
fn spawn_updater() {
  std::thread::spawn(|| {
    tauri_api::command::spawn_relative_command(
      "updater".to_string(),
      Vec::new(),
      std::process::Stdio::inherit(),
    )
    .expect("Unable to spawn relative command");
  });
}

pub fn event_initialization_script() -> String {
  #[cfg(not(event))]
  return String::from("");
  #[cfg(event)]
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
        window.__TAURI__.promisified({{
          cmd: 'validateSalt',
          salt: salt
        }}).then(function () {{
          for (let i = listeners.length - 1; i >= 0; i--) {{
            const listener = listeners[i]
            if (listener.once)
              listeners.splice(i, 1)
            listener.handler(payload)
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

// build the webview struct
fn build_webview<A: ApplicationExt + 'static>(
  application: App<A>,
  content: Content<String>,
  splashscreen_content: Option<Content<String>>,
) -> crate::Result<(A, A::Dispatcher)> {
  let config = &application.context.config;
  // TODO let debug = cfg!(debug_assertions);
  // get properties from config struct
  // TODO let width = config.tauri.window.width;
  // TODO let height = config.tauri.window.height;
  let resizable = config.tauri.window.resizable;
  // let fullscreen = config.tauri.window.fullscreen;
  let title = config.tauri.window.title.clone();

  let has_splashscreen = splashscreen_content.is_some();
  let initialized_splashscreen = Arc::new(AtomicBool::new(false));

  let content_url = match content {
    Content::Html(s) => s,
    Content::Url(s) => s,
  };
  let url = match splashscreen_content {
    Some(Content::Html(s)) => s,
    _ => content_url.to_string(),
  };

  let initialization_script = format!(
    r#"
      {tauri_initialization_script}
      {event_initialization_script}
      if (window.__TAURI_INVOKE_HANDLER__) {{
        window.__TAURI_INVOKE_HANDLER__(JSON.stringify({{ cmd: "__initialized" }}))
      }} else {{
        window.addEventListener('DOMContentLoaded', function () {{
          window.__TAURI_INVOKE_HANDLER__(JSON.stringify({{ cmd: "__initialized" }}))
        }})
      }}
      {plugin_initialization_script}
    "#,
    tauri_initialization_script = application.context.tauri_script,
    event_initialization_script = event_initialization_script(),
    plugin_initialization_script =
      crate::async_runtime::block_on(crate::plugin::initialization_script(A::plugin_store()))
  );

  let application = Arc::new(application);

  let mut webview_application = A::new()?;

  let main_window =
    webview_application.create_window(A::WindowBuilder::new().resizable(resizable).title(title))?;

  let dispatcher = webview_application.dispatcher(&main_window);

  let tauri_invoke_handler = crate::Callback::<A::Dispatcher> {
    name: "__TAURI_INVOKE_HANDLER__".to_string(),
    function: Box::new(move |dispatcher, _, arg| {
      let arg = arg.into_iter().next().unwrap_or_else(String::new);
      let application = application.clone();
      let mut dispatcher = dispatcher.clone();
      let content_url = content_url.to_string();
      let initialized_splashscreen = initialized_splashscreen.clone();

      crate::async_runtime::spawn(async move {
        if arg == r#"{"cmd":"__initialized"}"# {
          let source = if has_splashscreen && !initialized_splashscreen.load(Ordering::Relaxed) {
            initialized_splashscreen.swap(true, Ordering::Relaxed);
            "splashscreen"
          } else {
            "window-1"
          };
          application
            .run_setup(&mut dispatcher, source.to_string())
            .await;
          if source == "window-1" {
            crate::plugin::ready(A::plugin_store(), &mut dispatcher).await;
          }
        } else if arg == r#"{"cmd":"closeSplashscreen"}"# {
          dispatcher.eval(&format!(r#"window.location.href = "{}""#, content_url));
        } else {
          let mut endpoint_handle =
            crate::endpoints::handle(&mut dispatcher, &arg, &application.context)
              .await
              .map_err(|e| e.to_string());
          if let Err(ref tauri_handle_error) = endpoint_handle {
            if tauri_handle_error.contains("unknown variant") {
              let error = match application.run_invoke_handler(&mut dispatcher, &arg).await {
                Ok(handled) => {
                  if handled {
                    String::from("")
                  } else {
                    tauri_handle_error.to_string()
                  }
                }
                Err(e) => e,
              };
              endpoint_handle = Err(error);
            }
          }
          if let Err(ref app_handle_error) = endpoint_handle {
            if app_handle_error.contains("unknown variant") {
              let error =
                match crate::plugin::extend_api(A::plugin_store(), &mut dispatcher, &arg).await {
                  Ok(_) => String::from(""),
                  Err(e) => e.to_string(),
                };
              endpoint_handle = Err(error);
            }
          }
          endpoint_handle = endpoint_handle.map_err(|e| e.replace("'", "\\'"));
          if let Err(handler_error_message) = endpoint_handle {
            if !handler_error_message.is_empty() {
              dispatcher.eval(&get_api_error_message(&arg, handler_error_message));
            }
          }
        }
      });
      0
    }),
  };

  webview_application.create_webview(
    A::WebviewBuilder::new()
      .url(url)
      .initialization_script(&initialization_script),
    main_window,
    vec![tauri_invoke_handler],
  )?;

  // TODO waiting for webview window API
  // webview.set_fullscreen(fullscreen);

  Ok((webview_application, dispatcher))
}

// Formats an invoke handler error message to print to console.error
fn get_api_error_message(arg: &str, handler_error_message: String) -> String {
  format!(
    r#"console.error('failed to match a command for {}, {}')"#,
    arg.replace("'", "\\'"),
    handler_error_message
  )
}

#[cfg(test)]
mod test {
  use super::Content;
  use crate::{Context, FromTauriContext};
  use proptest::prelude::*;
  #[cfg(dev)]
  use std::io::Read;

  #[derive(FromTauriContext)]
  #[config_path = "test/fixture/src-tauri/tauri.conf.json"]
  struct TauriContext;

  #[test]
  fn check_setup_content() {
    let context = Context::new::<TauriContext>().unwrap();
    let res = super::setup_content(&context);

    #[cfg(embedded_server)]
    match res {
      Ok(Content::Url(ref u)) => assert!(u.contains("http://")),
      _ => panic!("setup content failed"),
    }

    #[cfg(dev)]
    {
      let config = &context.config;
      match res {
        Ok(Content::Url(dp)) => assert_eq!(dp, config.build.dev_path),
        Ok(Content::Html(s)) => {
          assert_eq!(
            s,
            format!(
              "data:text/html;base64,{}",
              base64::encode(
                context
                  .assets
                  .get(
                    &crate::api::assets::Assets::format_key("index.html"),
                    crate::api::assets::AssetFetch::Decompress
                  )
                  .ok_or_else(|| crate::Error::AssetNotFound("index.html".to_string()))
                  .and_then(|(read, _)| {
                    read
                      .bytes()
                      .collect::<Result<Vec<u8>, _>>()
                      .map_err(Into::into)
                  })
                  .expect("Unable to find `index.html` under your dist folder")
              )
            )
          );
        }
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
