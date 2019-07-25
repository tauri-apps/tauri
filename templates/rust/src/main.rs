#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate proton;
extern crate proton_ui;
extern crate serde_json;

#[cfg(not(feature = "dev"))]
extern crate includedir;
#[cfg(not(feature = "dev"))]
extern crate phf;

#[cfg(not(feature = "dev"))]
extern crate tiny_http;

#[cfg(feature = "dev")]
use clap::{App, Arg};

#[cfg(not(feature = "dev"))]
#[cfg(not(feature = "serverless"))]
use std::thread;

mod cmd;

#[cfg(not(feature = "dev"))]
#[cfg(not(feature = "serverless"))]
mod server;

fn main() {
  let debug;
  let content;
  let _server_url: String;

  #[cfg(feature = "updater")]
  {
    thread::spawn(|| {
      proton::command::spawn_relative_command(
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
    content = proton_ui::Content::Url(matches.value_of("url").unwrap().to_owned());
    debug = true;
  }

  #[cfg(not(feature = "dev"))]
  {
    debug = cfg!(debug_assertions);
    #[cfg(feature = "serverless")]
    {
      fn inline_style(s: &str) -> String {
        format!(r#"<style type="text/css">{}</style>"#, s)
      }
      fn inline_script(s: &str) -> String {
        format!(r#"<script type="text/javascript">{}</script>"#, s)
      }
      let html = format!(r#"<!DOCTYPE html><html><head><meta http-equiv="Content-Security-Policy" content="default-src data: filesystem: ws: http: https: 'unsafe-eval' 'unsafe-inline'">{styles}</head><body><div id="q-app"></div>{scripts}</body></html>"#,
        styles = inline_style(include_str!("../target/compiled-web/css/app.css")),
        scripts = inline_script(include_str!("../target/compiled-web/js/app.js")),
      );
      content = proton_ui::Content::Html(html);
    }
    #[cfg(not(feature = "serverless"))]
    {
      if let Some(available_port) = proton::tcp::get_available_port() {
        _server_url = format!("{}:{}", "127.0.0.1", available_port);
        content = proton_ui::Content::Url(format!("http://{}", _server_url));
      } else {
        panic!("Could not find an open port");
      }
    }
  }

  let config = proton::config::get();

  let webview = proton_ui::builder()
    .title(&config.title)
    .content(content)
    .size(config.width, config.height)
    .resizable(config.resizable)
    .debug(debug)
    .user_data(())
    .invoke_handler(|webview, arg| {
      // leave this as is to use the proton API from your JS code
      if !proton::api::handler(webview, arg) {
        use cmd::Cmd::*;
        match serde_json::from_str(arg) {
          Err(_) => {}
          Ok(command) => {
            match command {
              // definitions for your custom commands from Cmd here
              MyCustomCommand { argument } => {
                //  your command code
                println!("{}", argument);
              }
            }
          }
        }
      }

      Ok(())
    })
    .content(content)
    .build()
    .unwrap();

  #[cfg(not(feature = "dev"))]
  {

    #[cfg(not(feature = "serverless"))]
    {
      thread::spawn(move || {
        let server = tiny_http::Server::http(_server_url).unwrap();
        for request in server.incoming_requests() {
          let mut url = request.url().to_string();
          if url == "/" {
            url = "/index.html".to_string();
          }
          request.respond(server::asset_response(&url)).unwrap();
        }
      });
    }
  }

  webview.run().unwrap();
}
