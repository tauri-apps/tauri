use crate::event;
use std::thread::{sleep, spawn};
use tauri_api::config::Config;
use web_view::WebView;

use tauri_updater;

pub fn spawn_update_process(webview: &WebView<'_, ()>, config: Config) -> crate::Result<()> {
  // todo(lemarier): wait the `update-available` event to be registred before checking our update
  let fivesec = std::time::Duration::from_millis(5000);
  sleep(fivesec);

  // do nothing if our updater is not active
  if !config.tauri.updater.active {
    return Ok(());
  }

  // prepare our endpoints
  let endpoints = config
    .tauri
    .updater
    .endpoints
    .expect("Unable to extract endpoints")
    .clone();

  // did we have a pubkey?
  let pubkey = config.tauri.updater.pubkey.clone();

  // check update
  check_update(&webview, endpoints, pubkey);

  Ok(())
}

fn check_update(webview: &WebView<'_, ()>, endpoints: Vec<String>, pubkey: Option<String>) {
  // clone our handler
  let handler = webview.handle();

  spawn(move || {
    // Check if we have a new version announced
    let updater = tauri_updater::builder()
      .urls(&endpoints[..])
      // we force the version 0.0.1 for our test
      // should be removed
      .current_version("0.0.1")
      .build()
      .expect("Unable to check updates")
      .run();

    if updater.should_update {
      println!("NEW VERSION AVAILABLE");
      // unwrap our body or return an empty string
      let body = updater.body.clone().unwrap_or("".into());

      // tell the world about our new update
      event::emit(
        &handler,
        "update-available".into(),
        Some(format!(
          r#"{{"version":"{:}", "date":"{:}", "body":"{:}"}}"#,
          updater.version, updater.date, body,
        )),
      );

      // we listen to our event to trigger the download
      event::listen(String::from("updater-install"), move |_msg| {
        // set status to downloading
        event::emit(
          &handler,
          "update-install-status".into(),
          Some(format!(r#"{{"status":"PENDING"}}"#)),
        );

        // init download
        // @todo:(lemarier) maybe emit download progress
        // but its a bit more complexe
        &updater
          .download_and_install(pubkey.clone())
          .expect("unable to download");

        event::emit(
          &handler,
          "update-install-status".into(),
          Some(format!(r#"{{"status":"DONE"}}"#)),
        );
      });
    }
  });
}
