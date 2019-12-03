#[cfg(feature = "embedded-server")]
use includedir_codegen::Compression;
use std::env;

#[cfg(not(feature = "embedded-server"))]
use lol_html::html_content::ContentType;
#[cfg(not(feature = "embedded-server"))]
use lol_html::{rewrite_str, element, RewriteStrSettings};
#[cfg(not(feature = "embedded-server"))]
use std::io::Write;

fn main() {
  #[cfg(feature = "embedded-server")]
  {
    match env::var("TAURI_DIST_DIR") {
      Ok(dist_path) => includedir_codegen::start("ASSETS")
        .dir(dist_path, Compression::None)
        .build("data.rs")
        .unwrap(),
      Err(_e) => panic!("Build error: Couldn't find ENV: {}", _e),
    }
  }
  #[cfg(not(feature = "embedded-server"))]
  {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("index.html");
    let mut file = std::fs::File::create(&dest_path).unwrap();

    let html = include_str!(concat!(env!("TAURI_DIST_DIR"), "/index.html"));
    let tauri_script = include_str!(concat!(env!("TAURI_DIR"), "/tauri.js"));

    let parsed_html = rewrite_str(
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
    )
    .unwrap();

    let template = include_str!("./template.html").replace("__TAURI_IFRAME_BASE64", &base64::encode(&parsed_html));

    file.write_all(template.as_bytes()).unwrap();
  }
}
