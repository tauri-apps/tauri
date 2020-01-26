#[allow(unused_imports)]
use std::{fs::read_to_string, path::Path, process::Stdio, thread::spawn};

use web_view::{builder, Content, WebView};

use super::App;
use crate::config::{get, Config};
#[cfg(feature = "embedded-server")]
use crate::tcp::{get_available_port, port_is_available};
use crate::TauriResult;

// JavaScript string literal
const JS_STRING: &'static str = r#"
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
pub(crate) fn run(application: &mut App) -> TauriResult<()> {
  // get the tauri config struct
  let config = get()?;

  // setup the content using the config struct depending on the compile target
  let content = setup_content(config.clone())?;

  // setup the server url for the embedded-server
  #[cfg(feature = "embedded-server")]
  let server_url = {
    if let Content::Url(ref url) = &content {
      String::from(url)
    } else {
      String::from("")
    }
  };

  // build the webview
  let mut webview = build_webview(application, config, content)?;
  webview.set_color((255, 255, 255));

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
fn setup_content(config: Config) -> TauriResult<Content<String>> {
  if config.build.dev_path.starts_with("http") {
    Ok(Content::Url(config.build.dev_path))
  } else {
    let dev_path = Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html");
    Ok(Content::Html(read_to_string(dev_path)?))
  }
}

// setup content for embedded server
#[cfg(feature = "embedded-server")]
fn setup_content(config: Config) -> TauriResult<Content<String>> {
  let (port, valid) = setup_port(config.clone()).expect("Unable to setup Port");
  let url = setup_server_url(config.clone(), valid, port).expect("Unable to setup URL");

  Ok(Content::Url(url.to_string()))
}

// setup content for no-server
#[cfg(feature = "no-server")]
fn setup_content(_: Config) -> TauriResult<Content<String>> {
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
fn spawn_server(server_url: String) -> TauriResult<()> {
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
fn spawn_updater() -> TauriResult<()> {
  spawn(|| {
    tauri_api::command::spawn_relative_command(
      "updater".to_string(),
      Vec::new(),
      Stdio::inherit(),
    )?;
  });
  Ok(())
}

// build the webview struct
fn build_webview(
  application: &mut App,
  config: Config,
  content: Content<String>,
) -> TauriResult<WebView<'_, ()>> {
  let debug = cfg!(debug_assertions);
  // get properties from config struct
  let width = config.tauri.window.width;
  let height = config.tauri.window.height;
  let resizable = config.tauri.window.resizable;
  let title = config.tauri.window.title.into_boxed_str();

  Ok(
    builder()
      .title(Box::leak(title))
      .size(width, height)
      .resizable(resizable)
      .debug(debug)
      .user_data(())
      .invoke_handler(move |webview, arg| {
        if arg == r#"{"cmd":"__initialized"}"# {
          application.run_setup(webview);
          webview.eval(JS_STRING)?;
        } else if let Ok(b) = crate::endpoints::handle(webview, arg) {
          if !b {
            application.run_invoke_handler(webview, arg);
          }
        }

        Ok(())
      })
      .content(content)
      .build()?,
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
