use crate::event;
use std::{
  process::exit,
  thread::{sleep, spawn},
  time::Duration,
};
use tauri_api::{config::get as get_config, dialog::ask, dialog::DialogSelection};
use web_view::WebView;

use tauri_updater;

/// Spawn the update process
pub fn spawn_update_process(webview: &WebView<'_, ()>) -> crate::Result<()> {
  if let Err(e) = init_updater(webview) {
    // we got an error, lets emit it so we can catch it with our event system later
    let handler = webview.handle();
    println!("[UPDATE ERROR] {}", e);
    event::emit(
      &handler,
      "updater-error".into(),
      Some(format!(r#"{{"error":"{:}"}}"#, e,)),
    );
  }

  Ok(())
}

// updater entrypoint
fn init_updater(webview: &WebView<'_, ()>) -> crate::Result<()> {
  let config = get_config()?;

  let handler = webview.handle();

  // do nothing if our updater is not active or we can't find endpoints
  if !config.tauri.updater.active || config.tauri.updater.endpoints.is_none() {
    return Ok(());
  }

  // prepare our endpoints
  let endpoints = config
    .tauri
    .updater
    .endpoints
    .as_ref()
    // this expect can lead to a panic
    // we should have a better handling here
    .expect("Something wrong with endpoints")
    .clone();

  // did we have a pubkey?
  let pubkey = config.tauri.updater.pubkey.clone();

  // if dialog is enabled, events aren't needed so use
  // simple updater
  if config.tauri.updater.dialog {
    return simple_update_with_dialog(&endpoints, &pubkey);
  }

  // check update inside a new thread
  spawn(move || -> crate::Result<()> {
    // todo(lemarier): wait the `update-available` event to be registred before checking our update
    let fivesec = Duration::from_millis(5000);
    sleep(fivesec);

    // Check if we have a new version announced
    let updater = tauri_updater::builder()
      .urls(&endpoints[..])
      // we force the version 0.0.1 for our test
      // should be removed
      .current_version("0.0.1")
      .build()?;

    if updater.should_update {
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
        &updater.download_and_install(pubkey.clone()).unwrap_or(());

        event::emit(
          &handler,
          "update-install-status".into(),
          Some(format!(r#"{{"status":"DONE"}}"#)),
        );
      });
    }

    Ok(())
  });

  Ok(())
}

fn simple_update_with_dialog(
  endpoints: &Vec<String>,
  pubkey: &Option<String>,
) -> crate::Result<()> {
  let updater = tauri_updater::builder()
    .urls(&endpoints[..])
    // we force the version 0.0.1 for our test
    // should be removed
    .current_version("0.0.1")
    .build()?;

  // we have a new update
  if updater.should_update {
    let body = updater.body.clone().unwrap_or("".into());

    // Ask user if we need to install
    let should_install = ask(
      &format!(
        r#"{:} {:} is now available -- you have {:}.
Would you like to install it now?

Release Notes:
{:}"#,
        // todo(lemarier): Replace with app name from cargo maybe?
        "AppX",
        updater.version,
        updater.current_version,
        body
      ),
      // todo(lemarier): Replace with app name from cargo maybe?
      &format!(r#"A new version of {:} is available! "#, "AppX"),
    );
    if should_install == DialogSelection::Yes {
      &updater.download_and_install(pubkey.clone())?;

      // Ask user if we need to close the app
      let should_exit = ask(
        "The installation was successful, do you want to restart the application now?",
        "Ready to Restart",
      );
      if should_exit == DialogSelection::Yes {
        exit(1);
      }
    }
  }

  return Ok(());
}
