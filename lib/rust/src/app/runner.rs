#[cfg(feature = "dev")]
use clap::{App, Arg};

#[cfg(not(feature = "dev"))]
#[cfg(feature = "embedded-server")]
use std::thread;

pub(crate) fn run(application: &mut crate::App) {
  let debug;
  let content;
  let config = crate::config::get();
  #[cfg(feature = "embedded-server")]
  let mut server_url: String;

  #[cfg(feature = "updater")]
  {
    thread::spawn(|| {
      crate::command::spawn_relative_command(
        "updater".to_string(),
        Vec::new(),
        std::process::Stdio::inherit(),
      )
      .unwrap();
    });
  }

  #[cfg(feature = "dev")]
  {
    let app = App::new("app")
      .version("1.0.0")
      .author("Author")
      .about("About")
      .arg(
        Arg::with_name("url")
          .short("u")
          .long("url")
          .value_name("URL")
          .help("Loads the specified URL into webview")
          .required(true)
          .takes_value(true),
      );

    let matches = app.get_matches();
    content = web_view::Content::Url(matches.value_of("url").unwrap().to_owned());
    debug = true;
  }

  #[cfg(not(feature = "dev"))]
  {
    debug = cfg!(debug_assertions);
    #[cfg(not(feature = "embedded-server"))]
    {
      content =
        web_view::Content::Html(include_str!(concat!(env!("TAURI_DIST_DIR"), "/index.html")));
    }
    #[cfg(feature = "embedded-server")]
    {
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
        server_url = format!("{}:{}", config.embedded_server.host, port);
        if !server_url.starts_with("http") {
          server_url = format!("http://{}", server_url);
        }
        content = web_view::Content::Url(server_url.clone());
      } else {
        panic!(format!("Port {} is not valid or not open", port));
      }
    }
  }

  let webview = web_view::builder()
    .title(&config.window.title)
    .size(config.window.width, config.window.height)
    .resizable(config.window.resizable)
    .debug(debug)
    .user_data(())
    .invoke_handler(|webview, arg| {
      // leave this as is to use the tauri API from your JS code
      if !crate::api::handler(webview, arg) {
        application.run_invoke_handler(webview, arg);
      }

      Ok(())
    })
    .content(content)
    .build()
    .unwrap();

  #[cfg(not(feature = "dev"))]
  {
    #[cfg(feature = "embedded-server")]
    {
      thread::spawn(move || {
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
            url = "/index.html".to_string();
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
