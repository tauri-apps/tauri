pub(crate) fn run(application: &mut crate::App) {
  let debug = cfg!(debug_assertions);
  let config = crate::config::get();
  let tauri_src = include_str!(concat!(env!("OUT_DIR"), "/tauri_src"));
  let content = if tauri_src.starts_with("http://") || tauri_src.starts_with("https://") {
    web_view::Content::Url(tauri_src)
  } else {
    web_view::Content::Html(tauri_src)
  };

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
  webview.handle()
    .dispatch(|_webview| {
      _webview.eval(include_str!(concat!(env!("TAURI_DIR"), "/tauri.js")))
    })
    .unwrap();

  #[cfg(feature = "embedded-server")]
  {
    std::thread::spawn(move || {
      let server = tiny_http::Server::http(
        tauri_src
          .clone()
          .replace("http://", "")
          .replace("https://", ""),
      )
      .expect(&format!(
        "Could not start embedded server with the specified url: {}",
        tauri_src
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

  webview.run().unwrap();
}
