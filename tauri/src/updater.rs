use crate::{
  api::{
    config::UpdaterConfig,
    dialog::{ask, AskResponse},
  },
  ApplicationExt, WebviewManager,
};
use std::process::exit;

// todo(lemarier): Attention CARGO_PKG_VERSION & CARGO_PKG_NAME values 
// are from tauri and not from the compiled application -- 
// we need to find a way to pass data from the compiled app

// Read app version from Cargo to compare with announced version
const APP_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
// Read app name from Cargo to show in dialog
const APP_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");
// Check for new updates
pub const EVENT_CHECK_UPDATE: &str = "tauri://update";
// New update available
pub const EVENT_UPDATE_AVAILABLE: &str = "tauri://update-available";
// Used to intialize an update *should run check-update first (once you received the update available event)*
pub const EVENT_INSTALL_UPDATE: &str = "tauri://update-install";
// Send updater status or error even if dialog is enabled, you should
// always listen for this event. It'll send you the install progress
// and any error triggered during update check and install
pub const EVENT_STATUS_UPDATE: &str = "tauri://update-status";
// this is the status emitted when the download start
pub const EVENT_STATUS_PENDING: &str = "PENDING";
// When you got this status, something went wrong
// you can find the error message inside the `error` field.
pub const EVENT_STATUS_ERROR: &str = "ERROR";
// When you receive this status, you should ask the user to restart
pub const EVENT_STATUS_SUCCESS: &str = "DONE";

#[derive(Clone, serde::Serialize)]
struct StatusEvent {
  status: String,
  error: Option<String>,
}

#[derive(Clone, serde::Serialize)]
struct UpdateAvailableEvent {
  version: String,
  date: String,
  body: String,
}

/// Spawn the update process for dialog
pub(crate) async fn spawn_update_process_dialog<A: ApplicationExt + 'static>(
  updater_config: UpdaterConfig,
  webview_manager: &WebviewManager<A>,
) {
  if !updater_config.active || updater_config.endpoints.is_none() {
    return;
  }

  // prepare our endpoints
  let endpoints = updater_config
    .endpoints
    .as_ref()
    // this expect can lead to a panic
    // we should have a better handling here
    .expect("Something wrong with endpoints")
    .clone();

  // check updates
  match tauri_updater::builder()
    .urls(&endpoints[..])
    .current_version(APP_VERSION.unwrap_or("0.0.0"))
    .build()
    .await
  {
    Ok(updater) => {
      let app_name = APP_NAME.unwrap_or("Unknown");
      let pubkey = updater_config.pubkey.clone();

      // if dialog enabled only
      if updater.should_update && updater_config.dialog {
        let body = updater.body.clone().unwrap_or_else(|| "".into());
        let dialog = dialog_update(&updater.clone(), app_name, &body.clone(), pubkey).await;
        if dialog.is_err() {
          let _res = webview_manager
            .clone()
            .emit(
              EVENT_STATUS_UPDATE,
              Some(StatusEvent {
                error: Some(dialog.err().unwrap().to_string()),
                status: "ERROR".into(),
              }),
            )
            .await;
          return;
        }
      }
    }
    Err(e) => {
      let error_message = match e {
        tauri_updater::Error::Updater(err) => Some(err),
        _ => Some(String::from("Something went wrong"))
      };

      let _res = webview_manager
        .clone()
        .emit(
          EVENT_STATUS_UPDATE,
          Some(StatusEvent {
            error: error_message,
            status: String::from("ERROR"),
          }),
        )
        .await;      
    },
  }
}

pub(crate) fn listen_events<A: ApplicationExt + 'static>(
  updater_config: UpdaterConfig,
  webview_manager: &WebviewManager<A>,
) {
  let isolated_webview_manager = webview_manager.clone();

  webview_manager.listen(EVENT_CHECK_UPDATE, move |_msg| {
    let webview_manager = isolated_webview_manager.clone();

    // prepare our endpoints
    let endpoints = updater_config
      .endpoints
      .as_ref()
      .expect("Something wrong with endpoints")
      .clone();

    let pubkey = updater_config.pubkey.clone();

    // check updates
    crate::async_runtime::spawn_task(async move {
      let webview_manager = webview_manager.clone();
      let webview_manager_isolation = webview_manager.clone();
      let pubkey = pubkey.clone();

      match tauri_updater::builder()
        .urls(&endpoints[..])
        .current_version(APP_VERSION.unwrap_or("0.0.0"))
        .build()
        .await
      {
        Ok(updater) => {
          // send notification if we need to update
          if updater.should_update {
            let body = updater.body.clone().unwrap_or_else(|| "".into());

            let _res = webview_manager
              .emit(
                EVENT_UPDATE_AVAILABLE,
                Some(UpdateAvailableEvent {
                  body,
                  date: updater.date.clone(),
                  version: updater.version.clone(),
                }),
              )
              .await;

            // listen for update install
            webview_manager.listen(EVENT_INSTALL_UPDATE, move |_msg| {
              let webview_manager = webview_manager_isolation.clone();
              let updater = updater.clone();
              let pubkey = pubkey.clone();

              // send status
              crate::async_runtime::spawn_task(async move {
                // emit {"status": "PENDING"}
                let _res = webview_manager
                  .clone()
                  .emit(
                    EVENT_STATUS_UPDATE,
                    Some(StatusEvent {
                      error: None,
                      status: String::from(EVENT_STATUS_PENDING),
                    }),
                  )
                  .await;

                let update_result = updater.clone().download_and_install(pubkey.clone()).await;

                if update_result.is_err() {
                  // emit {"status": "ERROR", "error": "The error message"}
                  let _res = webview_manager
                    .clone()
                    .emit(
                      EVENT_STATUS_UPDATE,
                      Some(StatusEvent {
                        error: Some(update_result.err().unwrap().to_string()),
                        status: String::from(EVENT_STATUS_ERROR),
                      }),
                    )
                    .await;
                } else {
                  // emit {"status": "DONE"}
                  let _res = webview_manager
                    .clone()
                    .emit(
                      EVENT_STATUS_UPDATE,
                      Some(StatusEvent {
                        error: None,
                        status: String::from(EVENT_STATUS_SUCCESS),
                      }),
                    )
                    .await;
                }
              })
            });
          }
        }
        Err(e) => {
          let error_message = match e {
            tauri_updater::Error::Updater(err) => Some(err),
            _ => Some(String::from("Something went wrong"))
          };
    
          let _res = webview_manager
            .clone()
            .emit(
              EVENT_STATUS_UPDATE,
              Some(StatusEvent {
                error: error_message,
                status: String::from("ERROR"),
              }),
            )
            .await;      
        },
      }
    })
  });
}

async fn dialog_update(
  updater: &tauri_updater::Update,
  app_name: &str,
  body: &str,
  pubkey: Option<String>,
) -> crate::Result<()> {
  let should_install = ask(
    format!(r#"A new version of {} is available! "#, app_name),
    format!(
      r#"{} {} is now available -- you have {}.

Would you like to install it now?

Release Notes:
{}"#,
      // todo(lemarier): we should validate the body and make sure it
      // doesnt contain character like single or double quote (",')
      app_name,
      updater.version,
      updater.current_version,
      body
    ),
  );

  match should_install {
    AskResponse::Yes => {
      updater.download_and_install(pubkey.clone()).await?;

      // Ask user if we need to close the app
      let should_exit = ask(
        "Ready to Restart",
        "The installation was successful, do you want to restart the application now?",
      );
      match should_exit {
        AskResponse::Yes => {
          exit(1);
        }
        AskResponse::No => {
          // Do nothing -- maybe we can emit some event here
        }
      }
    }
    AskResponse::No => {
      // Do nothing -- maybe we can emit some event here
    }
  }

  updater.download_and_install(pubkey.clone()).await?;

  Ok(())
}
