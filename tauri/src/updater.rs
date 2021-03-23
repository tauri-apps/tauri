use crate::{
  api::{
    config::UpdaterConfig,
    dialog::{ask, AskResponse},
  },
  ApplicationExt, WebviewManager,
};
use std::{
  path::PathBuf,
  process::{exit, Command},
};

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
// When you receive this status, this is because the application is runniing last version
pub const EVENT_STATUS_UPTODATE: &str = "UPTODATE";

#[derive(Clone, serde::Serialize)]
struct StatusEvent {
  status: String,
  error: Option<String>,
}

#[derive(Clone, serde::Serialize)]
struct UpdateManifest {
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
        let body = updater.body.clone().unwrap_or_else(|| String::from(""));
        let dialog =
          prompt_for_install(&updater.clone(), package_info.name, &body.clone(), pubkey).await;

        if dialog.is_err() {
          send_status_update(
            webview_manager.clone(),
            EVENT_STATUS_ERROR,
            Some(dialog.err().unwrap().to_string()),
          );

          return;
        }
      }
    }
    Err(e) => {
      send_status_update(
        webview_manager.clone(),
        EVENT_STATUS_ERROR,
        Some(e.to_string()),
      );
    }
  }
}

/// Experimental listener
/// This function should be run on the main thread once.
pub(crate) fn listener<A: ApplicationExt + 'static>(
  updater_config: UpdaterConfig,
  package_info: crate::api::PackageInfo,
  webview_manager: &WebviewManager<A>,
) {
  let isolated_webview_manager = webview_manager.clone();

  // Wait to receive the event `"tauri://update"`
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
            let body = updater.body.clone().unwrap_or_else(|| String::from(""));

            // Emit `tauri://update-available`
            let _ = webview_manager.emit(
              EVENT_UPDATE_AVAILABLE,
              Some(UpdateManifest {
                body,
                date: updater.date.clone(),
                version: updater.version.clone(),
              }),
            );

            // Listen for `tauri://update-install`
            webview_manager.once(EVENT_INSTALL_UPDATE, move |_msg| {
              let webview_manager = webview_manager_isolation.clone();
              let updater = updater.clone();
              let pubkey = pubkey.clone();

              // Start installation
              crate::async_runtime::spawn(async move {
                // emit {"status": "PENDING"}
                send_status_update(webview_manager.clone(), EVENT_STATUS_PENDING, None);

                // Launch updater download process
                // macOS we display the `Ready to restart dialog` asking to restart
                // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
                // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
                let update_result = updater.clone().download_and_install(pubkey.clone()).await;

                if update_result.is_err() {
                  // emit {"status": "ERROR", "error": "The error message"}
                  send_status_update(
                    webview_manager.clone(),
                    EVENT_STATUS_ERROR,
                    Some(update_result.err().unwrap().to_string()),
                  );
                } else {
                  // emit {"status": "DONE"}
                  // todo(lemarier): maybe we should emit the
                  // path of the current EXE so they can restart it
                  send_status_update(webview_manager.clone(), EVENT_STATUS_SUCCESS, None);
                }
              })
            });
          } else {
            send_status_update(webview_manager.clone(), EVENT_STATUS_UPTODATE, None);
          }
        }
        Err(e) => {
          send_status_update(
            webview_manager.clone(),
            EVENT_STATUS_ERROR,
            Some(e.to_string()),
          );
        }
      }
    })
  });
}

// Send a status update via `tauri://update-status` event.
fn send_status_update<A: ApplicationExt + 'static>(
  webview_manager: WebviewManager<A>,
  status: &str,
  error: Option<String>,
) {
  let _ = webview_manager.emit(
    EVENT_STATUS_UPDATE,
    Some(StatusEvent {
      error,
      status: String::from(status),
    }),
  );
}

// Prompt a dialog asking if the user want to install the new version
// Maybe we should add an option to customize it in future versions.
async fn prompt_for_install(
  updater: &tauri_updater::Update,
  app_name: &str,
  body: &str,
  pubkey: Option<String>,
) -> crate::Result<()> {
  // remove single & double quote
  let escaped_body = body.replace(&['\"', '\''][..], "");

  // todo(lemarier): We should review this and make sure we have
  // something more conventional.
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
      // Launch updater download process
      // macOS we display the `Ready to restart dialog` asking to restart
      // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
      // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
      updater.download_and_install(pubkey.clone()).await?;

      // Ask user if we need to restart the application
      let should_exit = ask(
        "Ready to Restart",
        "The installation was successful, do you want to restart the application now?",
      );
      match should_exit {
        AskResponse::Yes => {
          restart_application(updater.current_binary.as_ref());
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

// Tested on macOS and Linux. Windows will not trigger the dialog
// as it'll exit before, to launch the MSI installation.
fn restart_application(binary_to_start: Option<&PathBuf>) {
  // spawn new process
  if let Some(path) = binary_to_start {
    Command::new(path)
      .spawn()
      .expect("application failed to start");
  }

  exit(0);
}
