use crate::{
  api::{
    config::UpdaterConfig,
    dialog::{ask, AskResponse},
  },
  ApplicationExt, WebviewManager,
};
use std::process::{exit, Command};

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

/// Check if there is any new update with builtin dialog.
pub(crate) async fn check_update_with_dialog<A: ApplicationExt + 'static>(
  updater_config: UpdaterConfig,
  package_info: crate::api::PackageInfo,
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
    .current_version(package_info.version)
    .build()
    .await
  {
    Ok(updater) => {
      let pubkey = updater_config.pubkey.clone();

      // if dialog enabled only
      if updater.should_update && updater_config.dialog {
        let body = updater.body.clone().unwrap_or_else(|| "".into());
        let dialog =
          ask_if_should_install(&updater.clone(), package_info.name, &body.clone(), pubkey).await;
        if dialog.is_err() {
          let _ = webview_manager.clone().emit(
            EVENT_STATUS_UPDATE,
            Some(StatusEvent {
              error: Some(dialog.err().unwrap().to_string()),
              status: "ERROR".into(),
            }),
          );
          return;
        }
      }
    }
    Err(e) => {
      let error_message = match e {
        tauri_updater::Error::Updater(err) => Some(err),
        _ => Some(String::from("Something went wrong")),
      };

      let _ = webview_manager.clone().emit(
        EVENT_STATUS_UPDATE,
        Some(StatusEvent {
          error: error_message,
          status: String::from("ERROR"),
        }),
      );
    }
  }
}

/// Experimental listener
pub(crate) fn listener<A: ApplicationExt + 'static>(
  updater_config: UpdaterConfig,
  package_info: crate::api::PackageInfo,
  webview_manager: &WebviewManager<A>,
) {
  let isolated_webview_manager = webview_manager.clone();

  webview_manager.unlisten_by_name(EVENT_CHECK_UPDATE);
  webview_manager.listen(EVENT_CHECK_UPDATE, move |_msg| {
    let webview_manager = isolated_webview_manager.clone();
    let package_info = package_info.clone();

    // prepare our endpoints
    let endpoints = updater_config
      .endpoints
      .as_ref()
      .expect("Something wrong with endpoints")
      .clone();

    let pubkey = updater_config.pubkey.clone();

    // check updates
    crate::async_runtime::spawn(async move {
      let webview_manager = webview_manager.clone();
      let webview_manager_isolation = webview_manager.clone();
      let pubkey = pubkey.clone();

      match tauri_updater::builder()
        .urls(&endpoints[..])
        .current_version(package_info.version)
        .build()
        .await
      {
        Ok(updater) => {
          // send notification if we need to update
          if updater.should_update {
            let body = updater.body.clone().unwrap_or_else(|| "".into());

            let _ = webview_manager.emit(
              EVENT_UPDATE_AVAILABLE,
              Some(UpdateAvailableEvent {
                body,
                date: updater.date.clone(),
                version: updater.version.clone(),
              }),
            );

            // listen for update install
            webview_manager.unlisten_by_name(EVENT_INSTALL_UPDATE);
            webview_manager.once(EVENT_INSTALL_UPDATE, move |_msg| {
              let webview_manager = webview_manager_isolation.clone();
              let updater = updater.clone();
              let pubkey = pubkey.clone();

              // send status
              crate::async_runtime::spawn(async move {
                // emit {"status": "PENDING"}
                let _ = webview_manager.clone().emit(
                  EVENT_STATUS_UPDATE,
                  Some(StatusEvent {
                    error: None,
                    status: String::from(EVENT_STATUS_PENDING),
                  }),
                );

                let update_result = updater.clone().download_and_install(pubkey.clone()).await;

                if update_result.is_err() {
                  // emit {"status": "ERROR", "error": "The error message"}
                  let _ = webview_manager.clone().emit(
                    EVENT_STATUS_UPDATE,
                    Some(StatusEvent {
                      error: Some(update_result.err().unwrap().to_string()),
                      status: String::from(EVENT_STATUS_ERROR),
                    }),
                  );
                } else {
                  // emit {"status": "DONE"}
                  // todo(lemarier): maybe we should emit the
                  // path of the current EXE so they can restart it
                  let _ = webview_manager.clone().emit(
                    EVENT_STATUS_UPDATE,
                    Some(StatusEvent {
                      error: None,
                      status: String::from(EVENT_STATUS_SUCCESS),
                    }),
                  );
                }
              })
            });
          }
        }
        Err(e) => {
          let error_message = match e {
            tauri_updater::Error::Updater(err) => Some(err),
            _ => Some(String::from("Something went wrong")),
          };

          let _ = webview_manager.clone().emit(
            EVENT_STATUS_UPDATE,
            Some(StatusEvent {
              error: error_message,
              status: String::from("ERROR"),
            }),
          );
        }
      }
    })
  });
}

async fn ask_if_should_install(
  updater: &tauri_updater::Update,
  app_name: &str,
  body: &str,
  pubkey: Option<String>,
) -> crate::Result<()> {
  // remove single & double quote
  let escaped_body = body.replace(&['\"', '\''][..], "");

  let should_install = ask(
    format!(r#"A new version of {} is available! "#, app_name),
    format!(
      r#"{} {} is now available -- you have {}.

Would you like to install it now?

Release Notes:
{}"#,
      app_name, updater.version, updater.current_version, escaped_body,
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
          restart_application();
          // safely exit even if the process
          // should be killed
          return Ok(());
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

  Ok(())
}

// Tested on macos only and seems to works fine -- at least it restart!
// I guess on windows it'll require some tweaking
fn restart_application() {
  // spawn new process
  if let Ok(current_process) = std::env::current_exe() {
    Command::new(current_process)
      .spawn()
      .expect("application failed to start");
  }

  exit(1);
}
