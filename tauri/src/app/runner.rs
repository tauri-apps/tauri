use crate::config::Config;
#[cfg(feature = "embedded-server")]
use crate::tcp::{get_available_port, is_port_avaliable};

pub(crate) fn run(application: &mut crate::App) {
  let config = crate::config::get();

  let content = setup_content(config.clone()).expect("Unable to get content type");
  #[cfg(feature = "embedded-server")]
  let server_url = setup_server_url(config.clone()).expect("Unable to get server URL");

  #[cfg(feature = "updater")]
  match spawn_updater() {
    Some(_) => (),
    None => panic!("Failed to spawn updater"),
  };

  let webview = build_webview(application, config, content).expect("Unable to build Webview");

  #[cfg(feature = "dev-server")]
  webview
    .handle()
    .dispatch(|_webview| _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"))))
    .expect("Failed to grab webview handle");

  #[cfg(feature = "embedded-server")]
  spawn_server(server_url);

  webview.run().expect("Failed to run webview");
}

#[cfg(not(any(feature = "embedded-server", feature = "no-server")))]
fn setup_content(config: Config) -> Result<web_view::Content<String>, ()> {
  if config.build.dev_path.starts_with("http") {
    Ok(web_view::Content::Url(config.build.dev_path))
  } else {
    let dev_path = std::path::Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html");
    Ok(web_view::Content::Html(
      std::fs::read_to_string(dev_path).expect("failed to build index.tauri.html"),
    ))
  }
}

#[cfg(feature = "embedded-server")]
fn setup_content(config: Config) -> Result<web_view::Content<String>, ()> {
  if let Some(url) = setup_server_url(config) {
    Ok(web_view::Content::Url(url.to_string()))
  }
}

#[cfg(feature = "no-server")]
fn setup_content(config: Config) -> Result<web_view::Content<String>, ()> {
  let index_path = std::path::Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html");
  Ok(web_view::Content::Html(
    std::fs::read_to_string(index_path).expect("failed to read string"),
  ))
}

#[cfg(feature = "embedded-server")]
fn setup_port(config: &Config) -> Option<(String, bool)> {
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

#[cfg(feature = "embedded-server")]
fn setup_server_url(config: Config) -> Option<String> {
  if let Some((port, valid)) = setup_port(config) {
    if valid {
      let mut url = format!(
        "{}:{}",
        config.tauri.embedded_server.host, config.tauri.embedded_server.port
      );
      if !url.starts_with("http") {
        Some(format!("http://{}", url))
      }
      Some(url)
    }
  }
}

#[cfg(feature = "updater")]
fn spawn_updater() -> Result<(), ()> {
  std::thread::spawn(|| {
    tauri_api::command::spawn_relative_command(
      "updater".to_string(),
      Vec::new(),
      std::process::Stdio::inherit(),
    )
    .expect("Failed to spawn updater thread");
  });
  Ok(())
}

fn build_webview(
  application: &mut crate::App,
  config: Config,
  content: web_view::Content<String>,
) -> Result<web_view::WebView<'_, ()>, ()> {
  let debug = cfg!(debug_assertions);
  let width = config.tauri.window.width;
  let height = config.tauri.window.height;
  let resizable = config.tauri.window.resizable;
  let title = config.tauri.window.title.into_boxed_str();

  Ok(
    web_view::builder()
      .title(Box::leak(title))
      .size(width, height)
      .resizable(resizable)
      .debug(debug)
      .user_data(())
      .invoke_handler(move |webview, arg| {
        if arg == r#"{"cmd":"__initialized"}"# {
          application.run_setup(webview);
          webview
            .eval(
              "
          if (window.onTauriInit !== void 0) {
            window.onTauriInit()
          }
        ",
            )
            .expect("failed to evaluate window.onTauriInit");
        } else if !crate::endpoints::handle(webview, arg) {
          application.run_invoke_handler(webview, arg);
        }

        Ok(())
      })
      .content(content)
      .build()
      .expect("Failed to build webview builder"),
  )
}

#[cfg(feature = "embedded-server")]
fn spawn_server(server_url: String) {
  std::thread::spawn(move || {
    let server = tiny_http::Server::http(
      server_url
        .clone()
        .replace("http://", "")
        .replace("https://", ""),
    )
    .expect(&format!(
      "Could not start embedded server with the specified url: {}",
      server_url
    ));
    for request in server.incoming_requests() {
      let url = match request.url() {
        "/" => "/index.tauri.html",
        url => url,
      }
      .to_string();
      request
        .respond(crate::server::asset_response(&url))
        .expect("Failed to read asset type");
    }
  });
}
