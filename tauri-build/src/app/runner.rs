pub(crate) fn run(application: &mut crate::App) {
  let debug = cfg!(debug_assertions);
  let config = crate::config::get();

  let content;
  #[cfg(not(any(feature = "embedded-server", feature = "no-server")))]
  {
    content = if config.dev_path.starts_with("http") {
      web_view::Content::Url(config.dev_path)
    } else {
      let dev_path = std::path::Path::new(&config.dev_path).join("index.tauri.html");
      web_view::Content::Html(
        std::fs::read_to_string(dev_path).expect("failed to build index.tauri.html"),
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
    if config.embedded_server.port == "random" {
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
      port = config.embedded_server.port;
      port_valid = crate::tcp::port_is_available(
        port
          .parse::<u16>()
          .expect(&format!("Invalid port {}", port)),
      );
    }
    if port_valid {
      let mut url = format!("{}:{}", config.embedded_server.host, port);
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
      .unwrap();
    });
  }

  let mut ran_setup = false;

  let webview = web_view::builder()
    .title(&config.window.title)
    .size(config.window.width, config.window.height)
    .resizable(config.window.resizable)
    .debug(debug)
    .user_data(())
    .invoke_handler(|webview, arg| {
      if !crate::api::handler(webview, arg) {
        application.run_invoke_handler(webview, arg);
      }
      // the first command is always the `init`, so we can safely run the setup hook here
      if !ran_setup {
        ran_setup = true;
        application.run_setup(webview);
      }

      Ok(())
    })
    .content(content)
    .build()
    .unwrap();

  #[cfg(feature = "dev-server")]
  webview
    .handle()
    .dispatch(|_webview| _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"))))
    .unwrap();

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
          .unwrap();
      }
    });
  }

  webview.run().unwrap();
}
