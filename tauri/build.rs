#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[cfg(feature = "embedded-server")]
use includedir_codegen::Compression;

use lol_html::html_content::ContentType;
use lol_html::{rewrite_str, element, RewriteStrSettings};

use std::env;
use std::io::Write;


#[path = "src/config.rs"] mod config;
#[path = "src/tcp.rs"] mod tcp;

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  let dest_path = std::path::Path::new(&out_dir).join("index.html");
  let mut file = std::fs::File::create(&dest_path).unwrap();

  let tauri_src: String;
  let config = config::get();

  #[cfg(feature = "dev")]
  {
    tauri_src = if config.dev_path.starts_with("http") {
      config.dev_path
    } else {
      format!("data:text/html;base64,{}", parse_html_file(&std::fs::read_to_string(config.dev_path).unwrap()))
    };
  }

  #[cfg(not(feature = "dev"))]
  {
    #[cfg(feature = "embedded-server")]
    {
      match env::var("TAURI_DIST_DIR") {
        Ok(dist_path) => {
          // rewrite HTML
          // TODO would be nice if we could remove the index.html from the binary, since we rewrite it
          let index_path = std::path::Path::new(&dist_path).join("index.tauri.html");
          let mut index_file = std::fs::File::create(&index_path).unwrap();
          index_file.write_all(parse_dist_html().as_bytes()).unwrap();

          // include assets
          includedir_codegen::start("ASSETS")
          .dir(dist_path, Compression::None)
          .build("data.rs")
          .unwrap()
        },
        Err(_e) => panic!("Build error: Couldn't find ENV: {}", _e),
      }

      // define URL
      let port;
      let port_valid;
      if config.embedded_server.port == "random" {
        match tcp::get_available_port() {
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
        tauri_src = url.to_string();
        let server_url_path = std::path::Path::new(env!("TAURI_DIST_DIR")).join("tauri.server");
        let mut server_url_file = std::fs::File::create(&server_url_path).unwrap();
        server_url_file.write_all(url.as_bytes()).unwrap();
      } else {
        panic!(format!("Port {} is not valid or not open", port));
      }
    }

    #[cfg(not(feature = "embedded-server"))]
    {
      tauri_src = format!("data:text/html;base64,{}", &base64::encode(&parse_dist_html()));
    }
  }

  let out_html = include_str!("./template.html")
    .replace("__TAURI_SRC", &tauri_src);
  file.write_all(out_html.as_bytes()).unwrap();
}

#[cfg(not(feature = "dev"))]
fn parse_dist_html() -> String {
  parse_html_file(include_str!(concat!(env!("TAURI_DIST_DIR"), "/index.html")))
}

fn parse_html_file(html: &str) -> String {
  let tauri_script = include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"));

  rewrite_str(
    html,
    RewriteStrSettings {
      element_content_handlers: vec![
        element!("body", |el| {
          el.before(format!(r#"<script type="text/javascript"=>{}</script>"#, tauri_script).as_str(), ContentType::Html);
          Ok(())
        })
      ],
      ..RewriteStrSettings::default()
    },
  ).unwrap()
}