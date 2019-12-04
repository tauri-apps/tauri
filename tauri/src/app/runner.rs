pub(crate) fn run(application: &mut crate::App) {
  let debug = cfg!(feature = "dev") || cfg!(debug_assertions);
  let config = crate::config::get();
  let content;
  #[cfg(feature = "dev")]
  {
    content = if config.dev_path.starts_with("http") {
      web_view::Content::Url(config.dev_path.as_str())
    } else {
      web_view::Content::Html(include_str!(concat!(env!("OUT_DIR"), "/index.html")))
    };
  }
  #[cfg(not(feature = "dev"))]
  {
    content = web_view::Content::Html(include_str!(concat!(env!("OUT_DIR"), "/index.html")));
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

  #[cfg(feature = "dev")]
  {
    let handle = webview.handle();
    handle.dispatch(|_webview| {
      _webview.eval("window.teste = 5").unwrap();
      _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js")))
    }).unwrap();
  }

  #[cfg(not(feature = "dev"))]
  {
    #[cfg(feature = "embedded-server")]
    {
      let server_url = include_str!(concat!(env!("TAURI_DIST_DIR"), "/tauri.server"));
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
          let mut url = request.url().to_string();
          if url == "/" {
            url = "/index.tauri.html".to_string();
          }
          request
            .respond(crate::server::asset_response(&url))
            .unwrap();
        }
      });
    }
  }

  webview.run().unwrap();
}
