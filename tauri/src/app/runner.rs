use std::{
  path::Path,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use crate::{SizeHint, Webview, WebviewBuilder, WebviewDispatcher};

use super::App;
#[cfg(embedded_server)]
use crate::api::tcp::{get_available_port, port_is_available};
use tauri_api::config::get;

#[allow(dead_code)]
enum Content<T> {
  Html(T),
  Url(T),
}

/// Main entry point for running the Webview
pub(crate) fn run<W: Webview + 'static>(application: App<W>) -> crate::Result<()> {
  // setup the content using the config struct depending on the compile target
  let main_content = setup_content()?;

  // setup the server url for the embedded-server
  #[cfg(embedded_server)]
  let server_url = {
    if let Content::Url(ref url) = &main_content {
      String::from(url)
    } else {
      String::from("")
    }
  };

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
  let mut webview = build_webview(application, main_content, splashscreen_content)?;

  let mut dispatcher = webview.dispatcher();
  crate::async_runtime::spawn(async move {
    crate::plugin::created(W::plugin_store(), &mut dispatcher).await
  });

  // spawn the embedded server on our server url
  #[cfg(embedded_server)]
  spawn_server(server_url);

  // spin up the updater process
  #[cfg(feature = "updater")]
  spawn_updater();

  // run the webview
  webview.run();

  Ok(())
}

#[cfg(all(embedded_server, no_server))]
fn setup_content() -> crate::Result<Content<String>> {
  panic!("only one of `embedded-server` and `no-server` is allowed")
}

// setup content for dev-server
#[cfg(dev)]
fn setup_content() -> crate::Result<Content<String>> {
  let config = get()?;
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
    let dev_dir = &config.build.dev_path;
    let dev_path = Path::new(dev_dir).join("index.tauri.html");
    if !dev_path.exists() {
      panic!(
        "Couldn't find 'index.tauri.html' inside {}; did you forget to run 'tauri dev'?",
        dev_dir
      );
    }
    Ok(Content::Html(format!(
      "data:text/html,{}",
      urlencoding::encode(&std::fs::read_to_string(dev_path)?)
    )))
  }
}

// setup content for embedded server
#[cfg(all(embedded_server, not(no_server)))]
fn setup_content() -> crate::Result<Content<String>> {
  let (port, valid) = setup_port()?;
  let url = (if valid {
    setup_server_url(port)
  } else {
    Err(anyhow::anyhow!("invalid port"))
  })
  .expect("Unable to setup URL");

  Ok(Content::Url(url))
}

// setup content for no-server
#[cfg(all(no_server, not(embedded_server)))]
fn setup_content() -> crate::Result<Content<String>> {
  let html = include_str!(concat!(env!("OUT_DIR"), "/index.tauri.html"));
  Ok(Content::Html(format!(
    "data:text/html,{}",
    urlencoding::encode(html)
  )))
}

// get the port for the embedded server
#[cfg(embedded_server)]
#[allow(dead_code)]
fn setup_port() -> crate::Result<(String, bool)> {
  let config = get()?;
  match config.tauri.embedded_server.port {
    tauri_api::config::Port::Random => match get_available_port() {
      Some(available_port) => Ok((available_port.to_string(), true)),
      None => Ok(("0".to_string(), false)),
    },
    tauri_api::config::Port::Value(port) => {
      let port_valid = port_is_available(port);
      Ok((port.to_string(), port_valid))
    }
  }
}

// setup the server url for embedded server
#[cfg(embedded_server)]
#[allow(dead_code)]
fn setup_server_url(port: String) -> crate::Result<String> {
  let config = get()?;
  let mut url = format!("{}:{}", config.tauri.embedded_server.host, port);
  if !url.starts_with("http") {
    url = format!("http://{}", url);
  }
  Ok(url)
}

// spawn the embedded server
#[cfg(embedded_server)]
fn spawn_server(server_url: String) {
  std::thread::spawn(move || {
    let server = tiny_http::Server::http(server_url.replace("http://", "").replace("https://", ""))
      .expect("Unable to spawn server");
    for request in server.incoming_requests() {
      let url = match request.url() {
        "/" => "/index.tauri.html",
        url => url,
      }
      .to_string();
      request
        .respond(crate::server::asset_response(&url))
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

pub fn init() -> String {
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
fn build_webview<W: Webview + 'static>(
  application: App<W>,
  content: Content<String>,
  splashscreen_content: Option<Content<String>>,
) -> crate::Result<W> {
  let config = get()?;
  let debug = cfg!(debug_assertions);
  // get properties from config struct
  let width = config.tauri.window.width;
  let height = config.tauri.window.height;
  let resizable = if config.tauri.window.resizable {
    SizeHint::NONE
  } else {
    SizeHint::FIXED
  };
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

  let init = format!(
    r#"
      {tauri_init}
      {event_init}
      if (window.__TAURI_INVOKE_HANDLER__) {{
        window.__TAURI_INVOKE_HANDLER__(JSON.stringify({{ cmd: "__initialized" }}))
      }} else {{
        window.addEventListener('DOMContentLoaded', function () {{
          window.__TAURI_INVOKE_HANDLER__(JSON.stringify({{ cmd: "__initialized" }}))
        }})
      }}
      {plugin_init}
    "#,
    tauri_init = include_str!(concat!(env!("OUT_DIR"), "/__tauri.js")),
    event_init = init(),
    plugin_init = crate::async_runtime::block_on(crate::plugin::init_script(W::plugin_store()))
  );

  let application = Arc::new(application);

  let mut webview = W::Builder::new()
    .init(&init)
    .title(&title)
    .width(width as usize)
    .height(height as usize)
    .resizable(resizable)
    .debug(debug)
    .url(&url)
    .bind("__TAURI_INVOKE_HANDLER__", move |dispatcher, _, arg| {
      let arg = arg.into_iter().next().unwrap_or_else(|| String::new());
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
            crate::plugin::ready(W::plugin_store(), &mut dispatcher).await;
          }
        } else if arg == r#"{"cmd":"closeSplashscreen"}"# {
          dispatcher.eval(&format!(r#"window.location.href = "{}""#, content_url));
        } else {
          let mut endpoint_handle = crate::endpoints::handle(&mut dispatcher, &arg)
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
                match crate::plugin::extend_api(W::plugin_store(), &mut dispatcher, &arg).await {
                  Ok(handled) => {
                    if handled {
                      String::from("")
                    } else {
                      app_handle_error.to_string()
                    }
                  }
                  Err(e) => e,
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
    })
    .finish()?;
  // TODO waiting for webview window API
  // webview.set_fullscreen(fullscreen);

  if has_splashscreen {
    let env_var = envmnt::get_or("TAURI_DIR", "../dist");
    let path = Path::new(&env_var);
    let contents = std::fs::read_to_string(path.join("/tauri.js"))?;
    // inject the tauri.js entry point
    webview.eval(&contents);
  }

  Ok(webview)
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
  use proptest::prelude::*;
  use std::env;

  #[test]
  fn check_setup_content() {
    let tauri_dir = match option_env!("TAURI_DIR") {
      Some(d) => d.to_string(),
      None => env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .expect("Unable to convert to normal String"),
    };
    env::set_current_dir(tauri_dir).expect("failed to change cwd");
    let res = super::setup_content();

    #[cfg(embedded_server)]
    match res {
      Ok(Content::Url(ref u)) => assert!(u.contains("http://")),
      _ => panic!("setup content failed"),
    }

    #[cfg(no_server)]
    match res {
      Ok(Content::Html(s)) => {
        let dist_dir = match option_env!("TAURI_DIST_DIR") {
          Some(d) => d.to_string(),
          None => env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .expect("Unable to convert to normal String"),
        };
        assert_eq!(
          s,
          format!(
            "data:text/html,{}",
            urlencoding::encode(
              &std::fs::read_to_string(std::path::Path::new(&dist_dir).join("index.tauri.html"))
                .unwrap()
            )
          )
        );
      }
      _ => panic!("setup content failed"),
    }

    #[cfg(dev)]
    {
      let config = tauri_api::config::get().expect("unable to setup default config");
      match res {
        Ok(Content::Url(dp)) => assert_eq!(dp, config.build.dev_path),
        Ok(Content::Html(s)) => {
          let dev_dir = &config.build.dev_path;
          let dev_path = std::path::Path::new(dev_dir).join("index.tauri.html");
          assert_eq!(
            s,
            format!(
              "data:text/html,{}",
              urlencoding::encode(
                &std::fs::read_to_string(dev_path).expect("failed to read dev path")
              )
            )
          );
        }
        _ => panic!("setup content failed"),
      }
    }
  }

  #[cfg(embedded_server)]
  #[test]
  fn check_setup_port() {
    let res = super::setup_port();
    match res {
      Ok((_s, _b)) => {}
      _ => panic!("setup port failed"),
    }
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[cfg(embedded_server)]
    #[test]
    fn check_server_url(port in (any::<u32>().prop_map(|v| v.to_string()))) {
      let p = port.clone();

      let res = super::setup_server_url(port);

      match res {
        Ok(url) => assert!(url.contains(&p)),
        Err(e) => panic!("setup_server_url Err {:?}", e.to_string())
      }
    }
  }
}
