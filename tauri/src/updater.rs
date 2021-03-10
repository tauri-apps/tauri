use crate::{
  api::config::UpdaterConfig, api::dialog::ask, api::dialog::AskResponse, ApplicationExt,
  WebviewManager, WebviewDispatcher, ApplicationDispatcherExt,
};
use std::{process::exit, thread::sleep, time::Duration};

const APP_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const APP_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");

/// Spawn the update process
pub(crate) async fn spawn_update_process<A: ApplicationExt + 'static>(
  updater_config: UpdaterConfig,
  webview_manager: &WebviewManager<A>,
) {
  println!("[CHECK UPDATE]");
  // do nothing if our updater is not active or we can't find endpoints
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

  println!("[CHECK UPDATE ENDPOINTS] {:?}", endpoints);

  let webview_manager_isolation = webview_manager.clone();

  // check updates
  match tauri_updater::builder()
      .urls(&endpoints[..])
      .current_version(APP_VERSION.unwrap_or("0.0.0"))
      .build()
      .await
    {
      Ok(updater) => {
        // listen to our events
        println!("[LISTEN EVENTS]");
        let events = listen_events(updater.clone(), &updater_config, &webview_manager_isolation).await;
        if events.is_err() {
          println!("[EVENTS ERROR] {:?}", events.err());
          return;
        }
        let mut body = "".into();
        let app_name = APP_NAME.unwrap_or("Unknown");
        let pubkey = updater_config.pubkey.clone();

        println!("[SHOULD UPDATE] {:?}", updater.should_update);

        // prepare our data if needed
        if updater.should_update {
          body = updater.body.clone().unwrap_or_else(|| "".into());
        }

        // if dialog enabled
        if updater.should_update && updater_config.dialog {
          println!("[DIALOG]");
          let dialog  = dialog_update(updater.clone(), app_name, body.clone(), pubkey).await;
          if dialog.is_err() {
            println!("[EVENTS ERROR] {:?}", dialog.err());
            return;
          }
        }

        if updater.should_update {
          // todo(lemarier): wait the `update-available` event to be registred before checking our update
          let fivesec = Duration::from_millis(5000);
          sleep(fivesec);
          
          // tell the world about our new update
          webview_manager
          .emit(
            "update-available",
            Some(format!(
              r#"{{"version":"{:}", "date":"{:}", "body":"{:}"}}"#,
              updater.version.clone(), updater.date.clone(), body.clone(),
            )),
          )
          .await;
        }
      }
      Err(e) => match e {
        tauri_updater::Error::Updater(err) => {
          // todo emit
          println!("[UPDATER ERROR] {:?}", err);
        }
        _ => {
          // todo emit
          println!("[UPDATER ERROR] {:?}", e);
        },
      },
    }
}

async fn listen_events<A: ApplicationExt + 'static>(
  updater: tauri_updater::Update,
  updater_config: &UpdaterConfig,
  webview_manager: &WebviewManager<A>,
) -> crate::Result<()> {

  let current_webview_isolation = webview_manager.current_webview().await?;
  let pubkey = updater_config.pubkey.clone();

  // we listen to our event to trigger the download
  webview_manager.listen(String::from("updater-install"), move |_msg| {
    // set status to downloading
    // TODO handle error
    emit_status_change(&current_webview_isolation, "PENDING");

    // init download
    // @todo:(lemarier) maybe emit download progress
    // but its a bit more complexe
    updater.download_and_install(pubkey.clone());

    emit_status_change(&current_webview_isolation, "DONE");
  });

  Ok(())
}

async fn dialog_update(
  updater: tauri_updater::Update,
  app_name: &str,
  body: String,
  pubkey: Option<String>,
) -> crate::Result<()> {

  println!("[ASK QUESTION]");

  let should_install = ask(
    &format!(r#"{:} {:} is now available -- you have {:}.
Would you like to install it now?

Release Notes:
{:}"#,
      app_name,
      updater.version,
      updater.current_version,
      body
    ),
    // todo(lemarier): Replace with app name from cargo maybe?
    &format!(r#"A new version of {:} is available! "#, app_name),
  );

  println!("[QUESTION ASKED]");

  match should_install {
    AskResponse::Yes => {
      updater.download_and_install(pubkey.clone()).await?;

      // Ask user if we need to close the app
      let should_exit = ask(
        "The installation was successful, do you want to restart the application now?",
        "Ready to Restart",
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

  println!("[DOWNLOAD]");

  updater.download_and_install(pubkey.clone()).await?;
  println!("[DOWNLOAD DONE]");

  Ok(())
}

// non-async event emitter to be used inside the closures
// https://github.com/rust-lang/rust/issues/62290
fn emit_status_change<A: ApplicationDispatcherExt + 'static>(
  webview_dispatcher: &WebviewDispatcher<A>,
  status: &str,
) {
  super::event::emit(
    webview_dispatcher, 
    "update-install-status", 
    Some(format!(r#"{{"status":"{:}"}}"#, status))
  );
}

async fn download_and_install<A: ApplicationDispatcherExt + 'static>(
  updater: tauri_updater::Update,
  pubkey: Option<String>,
) {
  updater.download_and_install(pubkey.clone()).await;
}

