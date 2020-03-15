#[allow(unused_imports)]
use std::{fs::read_to_string, path::Path, process::Stdio, thread::spawn};

use web_view::{builder, Content, WebView};

use super::App;
use crate::config::{get, Config};
#[cfg(feature = "embedded-server")]
use crate::api::tcp::{get_available_port, port_is_available};

// JavaScript string literal
const JS_STRING: &str = r#"
if (window.onTauriInit !== void 0) {
  window.onTauriInit()
  window.onTauriInit = void 0
}
if (window.__TAURI_INIT_HOOKS !== void 0) {
  for (var hook in window.__TAURI_INIT_HOOKS) {
    window.__TAURI_INIT_HOOKS[hook]()
  }
  window.__TAURI_INIT_HOOKS = void 0
}
Object.defineProperty(window, 'onTauriInit', {
  set: function(val) {
    if (typeof(val) === 'function') {
      val()
    }
  }
})
"#;

// Main entry point function for running the Webview
pub(crate) fn run(application: &mut App) -> crate::Result<()> {
  // get the tauri config struct
  let config = get()?;

  // setup the content using the config struct depending on the compile target
  let main_content = setup_content(config.clone())?;

  // setup the server url for the embedded-server
  #[cfg(feature = "embedded-server")]
  let server_url = {
    if let Content::Url(ref url) = &main_content {
      String::from(url)
    } else {
      String::from("")
    }
  };

  // build the webview
  let mut webview = build_webview(
    application,
    config,
    main_content,
    if application.splashscreen_html().is_some() {
      Some(Content::Html(application.splashscreen_html().expect("failed to get splashscreen_html").to_string()))
    } else {
      None
    },
  )?;

  crate::plugin::created(&mut webview);

  // on dev-server grab a handler and execute the tauri.js API entry point.
  #[cfg(feature = "dev-server")]
  webview
    .handle()
    .dispatch(|_webview| _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"))))?;

  // spawn the embedded server on our server url
  #[cfg(feature = "embedded-server")]
  spawn_server(server_url.to_string())?;

  // spin up the updater process
  #[cfg(feature = "updater")]
  spawn_updater()?;

  // run the webview
  webview.run()?;

  Ok(())
}

// setup content for dev-server
#[cfg(not(any(feature = "embedded-server", feature = "no-server")))]
fn setup_content(config: Config) -> crate::Result<Content<String>> {
  if config.build.dev_path.starts_with("http") {
    Ok(Content::Url(config.build.dev_path))
  } else {
    let dev_path = Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html");
    Ok(Content::Html(read_to_string(dev_path)?))
  }
}

// setup content for embedded server
#[cfg(feature = "embedded-server")]
fn setup_content(config: Config) -> crate::Result<Content<String>> {
  let (port, valid) = setup_port(config.clone()).expect("Unable to setup Port");
  let url = setup_server_url(config.clone(), valid, port).expect("Unable to setup URL");

  Ok(Content::Url(url.to_string()))
}

// setup content for no-server
#[cfg(feature = "no-server")]
fn setup_content(_: Config) -> crate::Result<Content<String>> {
  let index_path = Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html");
  Ok(Content::Html(read_to_string(index_path)?))
}

// get the port for the embedded server
#[cfg(feature = "embedded-server")]
fn setup_port(config: Config) -> Option<(String, bool)> {
  if config.tauri.embedded_server.port == "random" {
    match get_available_port() {
      Some(available_port) => Some((available_port.to_string(), true)),
      None => Some(("0".to_string(), false)),
    }
  } else {
    let port = config.tauri.embedded_server.port;
    let port_valid = port_is_available(
      port
        .parse::<u16>()
        .expect(&format!("Invalid port {}", port)),
    );
    Some((port, port_valid))
  }
}

// setup the server url for embedded server
#[cfg(feature = "embedded-server")]
fn setup_server_url(config: Config, valid: bool, port: String) -> Option<String> {
  if valid {
    let mut url = format!("{}:{}", config.tauri.embedded_server.host, port);
    if !url.starts_with("http") {
      url = format!("http://{}", url);
    }
    Some(url)
  } else {
    None
  }
}

// spawn the embedded server
#[cfg(feature = "embedded-server")]
fn spawn_server(server_url: String) -> crate::Result<()> {
  spawn(move || {
    let server = tiny_http::Server::http(
      server_url
        .clone()
        .replace("http://", "")
        .replace("https://", ""),
    )
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

  Ok(())
}

// spawn an updater process.
#[cfg(feature = "updater")]
fn spawn_updater() -> crate::Result<()> {
  spawn(|| {
    tauri_api::command::spawn_relative_command("updater".to_string(), Vec::new(), Stdio::inherit())
      .expect("Unable to spawn relative command");
  });
  Ok(())
}

// build the webview struct
fn build_webview(
  application: &mut App,
  config: Config,
  content: Content<String>,
  splashscreen_content: Option<Content<String>>
) -> crate::Result<WebView<'_, ()>> {
  let content_clone = match content {
    Content::Html(ref html) => Content::Html(html.clone()),
    Content::Url(ref url) => Content::Url(url.clone()),
  };
  let debug = cfg!(debug_assertions);
  // get properties from config struct
  let width = config.tauri.window.width;
  let height = config.tauri.window.height;
  let resizable = config.tauri.window.resizable;
  let fullscreen = config.tauri.window.fullscreen;
  let title = config.tauri.window.title.into_boxed_str();

  let has_splashscreen = splashscreen_content.is_some();
  let mut initialized_splashscreen = false;

  let mut webview = builder()
    .title(Box::leak(title))
    .size(width, height)
    .resizable(resizable)
    .debug(debug)
    .user_data(())
    .invoke_handler(move |webview, arg| {
      if arg == r#"{"cmd":"__initialized"}"# {
        let source = if has_splashscreen && !initialized_splashscreen {
          initialized_splashscreen = true;
          "splashscreen"
        } else {
          "window-1"
        };
        application.run_setup(webview, source.to_string());
        webview.eval(JS_STRING)?;
        if source == "window-1" {
          let handle = webview.handle();
          handle.dispatch(|webview| {
            crate::plugin::ready(webview);
            Ok(())
          }).expect("failed to invoke ready hook");
        }
      } else if arg == r#"{"cmd":"closeSplashscreen"}"# {
        let content_href = match content_clone {
          Content::Html(ref html) => html,
          Content::Url(ref url) => url,
        };
        webview.eval(&format!(r#"window.location.href = "{}""#, content_href))?;
      } else {
        let endpoint_handle = crate::endpoints::handle(webview, arg)
          .map_err(|tauri_handle_error| {
            let tauri_handle_error_str = tauri_handle_error.to_string();
            if tauri_handle_error_str.contains("unknown variant") {
              match application.run_invoke_handler(webview, arg) {
                Ok(handled) => if handled { String::from("") } else { tauri_handle_error_str }
                Err(e) => e
              }
            } else {
              tauri_handle_error_str
            }
          })
          .map_err(|app_handle_error| {
            let app_handle_error_str = app_handle_error.to_string();
            if app_handle_error_str.contains("unknown variant") {
              match crate::plugin::extend_api(webview, arg) {
                Ok(handled) => {
                  if handled {
                    String::from("")
                  } else {
                    app_handle_error_str
                  }
                },
                Err(e) => e
              }
            } else {
              app_handle_error_str
            }
          })
          .map_err(|e| e.replace("'", "\\'"));
        if let Err(handler_error_message) = endpoint_handle {
          if handler_error_message != "" {
            webview.eval(
              &get_api_error_message(arg, handler_error_message)
            )?;
          }
        }
      }

      Ok(())
    })
    .content(if splashscreen_content.is_some() {
      splashscreen_content.expect("failed to get splashscreen content")
    } else {
      content
    })
    .build()?;

  webview.set_fullscreen(fullscreen);

  if has_splashscreen {
    // inject the tauri.js entry point
    webview
    .handle()
    .dispatch(|_webview| _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"))))?;
  }
  
  Ok(webview)
}

fn get_api_error_message(arg: &str, handler_error_message: String) -> String {
  format!(
    r#"console.error('failed to match a command for {}, {}')"#, 
    arg.replace("'", "\\'"),
    handler_error_message
  )
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;
  use web_view::Content;

  #[cfg(not(feature = "embedded-server"))]
  use std::{fs::read_to_string, path::Path};

  fn init_config() -> crate::config::Config {
    crate::config::get().expect("unable to setup default config")
  }

  #[test]
  fn check_setup_content() {
    let config = init_config();
    let _c = config.clone();

    let res = super::setup_content(config);

    #[cfg(feature = "embedded-server")]
    match res {
      Ok(Content::Url(u)) => assert!(u.contains("http://")),
      _ => assert!(false),
    }

    #[cfg(feature = "no-server")]
    match res {
      Ok(Content::Html(s)) => assert_eq!(
        s,
        read_to_string(Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html")).unwrap()
      ),
      _ => assert!(false),
    }

    #[cfg(not(any(feature = "embedded-server", feature = "no-server")))]
    match res {
      Ok(Content::Url(dp)) => assert_eq!(dp, _c.build.dev_path),
      Ok(Content::Html(s)) => assert_eq!(
        s,
        read_to_string(Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html")).unwrap()
      ),
      _ => assert!(false),
    }
  }

  #[cfg(feature = "embedded-server")]
  #[test]
  fn check_setup_port() {
    let config = init_config();

    let res = super::setup_port(config);
    match res {
      Some((_s, _b)) => assert!(true),
      _ => assert!(false),
    }
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[cfg(feature = "embedded-server")]
    #[test]
    fn check_server_url(port in (any::<u32>().prop_map(|v| v.to_string()))) {
      let config = init_config();
      let valid = true;

      let p = port.clone();

      let res = super::setup_server_url(config, valid, port);

      match res {
        Some(url) => assert!(url.contains(&p)),
        None => assert!(false)
      }
    }
  }
}
