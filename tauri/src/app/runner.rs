pub(crate) fn run(application: &mut crate::App) {
  let debug = cfg!(debug_assertions);
  let config = crate::config::get();

  let content;
  #[cfg(not(any(feature = "embedded-server", feature = "no-server")))]
  {
    content = if config.build.dev_path.starts_with("http") {
      web_view::Content::Url(config.build.dev_path)
    } else {
      let dev_path = std::path::Path::new(&config.build.dev_path).join("index.tauri.html");
      web_view::Content::Html(
        std::fs::read_to_string(dev_path).expect("failed to read index.tauri.html"),
      )
    };
  }

  #[cfg(feature = "embedded-server")]
  let server_url;

  #[cfg(feature = "embedded-server")]
  {
    // define URL
    let port;
    let port_valid;
    if config.tauri.embedded_server.port == "random" {
      match crate::tcp::get_available_port() {
        Some(available_port) => {
          port = available_port.to_string();
          port_valid = true;
        }
        None => {
          port = "0".to_string();
          port_valid = false;
        }
      }
    } else {
      port = config.tauri.embedded_server.port;
      port_valid = crate::tcp::port_is_available(
        port
          .parse::<u16>()
          .expect(&format!("Invalid port {}", port)),
      );
    }
    if port_valid {
      let mut url = format!("{}:{}", config.tauri.embedded_server.host, port);
      if !url.starts_with("http") {
        url = format!("http://{}", url);
      }
      server_url = url.clone();
      content = web_view::Content::Url(url.to_string());
    } else {
      panic!(format!("Port {} is not valid or not open", port));
    }
  }

  #[cfg(feature = "no-server")]
  {
    let index_path = std::path::Path::new(env!("TAURI_DIST_DIR")).join("index.tauri.html");
    content =
      web_view::Content::Html(std::fs::read_to_string(index_path).expect("failed to read string"));
  }

  #[cfg(feature = "updater")]
  {
    std::thread::spawn(|| {
      crate::command::spawn_relative_command(
        "updater".to_string(),
        Vec::new(),
        std::process::Stdio::inherit(),
      )
      .expect("Failed to spawn updater thread");
    });
  }

  let webview = web_view::builder()
    .title(&config.tauri.window.title)
    .size(config.tauri.window.width, config.tauri.window.height)
    .resizable(config.tauri.window.resizable)
    .debug(debug)
    .user_data(())
    .invoke_handler(|webview, arg| {
      if arg == r#"{"cmd":"__initialized"}"# {
        application.run_setup(webview);
        webview.eval("
          if (window.onTauriInit !== void 0) {
            window.onTauriInit()
          }
        ").expect("failed to evaluate window.onTauriInit");
      } else if !crate::endpoints::handle(webview, arg) {
        application.run_invoke_handler(webview, arg);
      }

      Ok(())
    })
    .content(content)
    .build()
    .expect("Failed to build webview builder");

  #[cfg(feature = "dev-server")]
  webview
    .handle()
    .dispatch(|_webview| _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"))))
    .expect("Failed to grab webview handle");

  #[cfg(feature = "embedded-server")]
  {
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

  webview.run().expect("Failed to run webview");
}
