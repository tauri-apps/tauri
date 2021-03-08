use crate::{ApplicationExt, WebviewManager, api::config::UpdaterConfig, api::dialog::ask, api::dialog::AskResponse};
use std::{
  process::exit,
  thread::{sleep},
  time::Duration,
};

const APP_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const APP_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");

// updater entrypoint
pub(crate) async fn spawn_update_process<A: ApplicationExt + 'static>(
    updater_config: UpdaterConfig,
    webview_manager: &WebviewManager<A>,
  ) -> crate::Result<()> {

  // do nothing if our updater is not active or we can't find endpoints
  if !updater_config.active || updater_config.endpoints.is_none() {
    return Ok(());
  }

  // prepare our endpoints
  let endpoints = updater_config
    .endpoints
    .as_ref()
    // this expect can lead to a panic
    // we should have a better handling here
    .expect("Something wrong with endpoints")
    .clone();

  // did we have a pubkey?
  let pubkey = updater_config.pubkey.clone();

  // if dialog is enabled, events aren't needed so use
  // simple updater
  if updater_config.dialog {
    let updater = tauri_updater::builder()
    .urls(&endpoints[..])
    .current_version(APP_VERSION.unwrap_or("0.0.0"))
    .build()?;

    // we have a new update
    if updater.should_update {
      let body = updater.body.clone().unwrap_or("".into());

      let app_name = APP_NAME.unwrap_or("Unknown");

      // Ask user if we need to install
      let should_install = ask(
        &format!(
          r#"{:} {:} is now available -- you have {:}.
  Would you like to install it now?

  Release Notes:
  {:}"#,
          // todo(lemarier): Replace with app name from cargo maybe?
          app_name,
          updater.version,
          updater.current_version,
          body
        ),
        // todo(lemarier): Replace with app name from cargo maybe?
        &format!(r#"A new version of {:} is available! "#, app_name),
      );
      match should_install {
        AskResponse::Yes => {
          &updater.download_and_install(pubkey.clone())?;

          // Ask user if we need to close the app
          let should_exit = ask(
            "The installation was successful, do you want to restart the application now?",
            "Ready to Restart",
          );
          match should_exit {
            AskResponse::Yes => {
              exit(1);
            },
            AskResponse::No => {
              // Do nothing -- maybe we can emit some event here
            }
          }
        },
        AskResponse::No => {
          // Do nothing -- maybe we can emit some event here
        }
      }
    }

    return Ok(());
  }

  // todo(lemarier): wait the `update-available` event to be registred before checking our update
  let fivesec = Duration::from_millis(5000);
  sleep(fivesec);

  // Check if we have a new version announced
  let updater = tauri_updater::builder()
    .urls(&endpoints[..])
    .current_version(APP_VERSION.unwrap_or("0.0.0"))
    .build()?;

  if updater.should_update {
    // unwrap our body or return an empty string
    let body = updater.body.clone().unwrap_or("".into());

    // tell the world about our new update
    webview_manager.emit(
      "update-available",
      Some(format!(
      r#"{{"version":"{:}", "date":"{:}", "body":"{:}"}}"#,
      updater.version, updater.date, body,
    ))).await?;

    let current_webview = webview_manager.current_webview().await?;
    let current_webview_clone = current_webview.clone();

    // we listen to our event to trigger the download
    current_webview.listen(String::from("updater-install"), move |_msg| {
      // set status to downloading
      // TODO handle error
      #[allow(unused_must_use)] {
        current_webview_clone.emit("update-install-status", Some(format!(r#"{{"status":"PENDING"}}"#)));
      }
  
      // init download
      // @todo:(lemarier) maybe emit download progress
      // but its a bit more complexe
      &updater.download_and_install(pubkey.clone()).unwrap_or(());

      // TODO handle error
      #[allow(unused_must_use)] {
        current_webview_clone.emit("update-install-status", Some(format!(r#"{{"status":"DONE"}}"#)));
      }
      
    })
  }


  Ok(())
}