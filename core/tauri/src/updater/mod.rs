// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri updater.
//!
//! The updater is focused on making Tauri's application updates **as safe and transparent as updates to a website**.
//!
//! Instead of publishing a feed of versions from which your app must select, Tauri updates to the version your server tells it to. This allows you to intelligently update your clients based on the request you give to Tauri.
//!
//! The server can remotely drive behaviors like rolling back or phased rollouts.
//!
//! The update JSON Tauri requests should be dynamically generated based on criteria in the request, and whether an update is required.
//!
//! Tauri's installer is also designed to be fault-tolerant, and ensure that any updates installed are valid and safe.
//!
//! # Configuration
//!
//! Once you have your Tauri project ready, you need to configure the updater.
//!
//! Add this in tauri.conf.json
//! ```json
//! "updater": {
//!     "active": true,
//!     "endpoints": [
//!         "https://releases.myapp.com/{target}}/{current_version}}"
//!     ],
//!     "dialog": true,
//!     "pubkey": ""
//! }
//! ```
//!
//! The required keys are "active" and "endpoints", others are optional.
//!
//! "active" must be a boolean. By default, it's set to false.
//!
//! "endpoints" must be an array. The string `{{target}}` and `{{current_version}}` are automatically replaced in the URL allowing you determine [server-side](#update-server-json-format) if an update is available. If multiple endpoints are specified, the updater will fallback if a server is not responding within the optional timeout.
//!
//! "dialog" if present must be a boolean. By default, it's set to true. If enabled, [events](#events) are turned-off as the updater will handle everything. If you need the custom events, you MUST turn off the built-in dialog.
//!
//! "pubkey" if present must be a valid public-key generated with Tauri cli. See [Signing updates](#signing-updates).
//!
//! ## Update Requests
//!
//! Tauri is indifferent to the request the client application provides for update checking.
//!
//! `Accept: application/json` is added to the request headers because Tauri is responsible for parsing the response.
//!
//! For the requirements imposed on the responses and the body format of an update, response see [Server Support](#server-support).
//!
//! Your update request must *at least* include a version identifier so that the server can determine whether an update for this specific version is required.
//!
//! It may also include other identifying criteria such as operating system version, to allow the server to deliver as fine-grained an update as you would like.
//!
//! How you include the version identifier or other criteria is specific to the server that you are requesting updates from. A common approach is to use query parameters, [Configuration](#configuration) shows an example of this.
//!
//! ## Built-in dialog
//!
//! By default, updater uses a built-in dialog API from Tauri.
//!
//! ![New Update](https://i.imgur.com/UMilB5A.png)
//!
//! The dialog release notes is represented by the update `note` provided by the [server](#server-support).
//!
//! If the user accepts, the download and install are initialized. The user will be then prompted to restart the application.
//!
//! ## Javascript API
//!
//! **Attention, you need to _disable built-in dialog_ in your [tauri configuration](#configuration), otherwise, events aren't emitted and the javascript API will NOT work.**
//!
//!
//! ```javascript
//! import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
//!
//! try {
//!     const { shouldUpdate, manifest } = await checkUpdate();
//!
//!     if (shouldUpdate) {
//!         // display dialog
//!         await installUpdate();
//!         // install complete, ask to restart
//!     }
//! } catch(error) {
//!     console.log(error);
//! }
//! ```
//!
//! ## Events
//!
//! **Attention, you need to _disable built-in dialog_ in your [tauri configuration](#configuration), otherwise, events aren't emitted.**
//!
//! To know when an update is ready to be installed, you can subscribe to these events:
//!
//! ### Initialize updater and check if a new version is available
//!
//! Event : `tauri://update`
//!
//! #### Rust
//! ```no_run
//! tauri::Builder::default()
//!   .setup(|app| {
//!     let handle = app.handle();
//!     tauri::async_runtime::spawn(async move {
//!       let response = handle.updater().check().await;
//!     });
//!     Ok(())
//!   });
//! ```
//!
//! #### Javascript
//! ```js
//! import { emit } from "@tauri-apps/api/event";
//! emit("tauri://update");
//! ```
//!
//! **If a new version is available, the event `tauri://update-available` is emitted.**
//!
//! ### Listen New Update Available
//!
//! Event : `tauri://update-available`
//!
//! Emitted data:
//! ```text
//! version    Version announced by the server
//! date       Date announced by the server
//! body       Note announced by the server
//! ```
//!
//! #### Rust
//! ```no_run
//! let app = tauri::Builder::default()
//!   // on an actual app, remove the string argument
//!   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
//!   .expect("error while building tauri application");
//! app.run(|_app_handle, event| match event {
//!   tauri::RunEvent::Updater(updater_event) => {
//!     match updater_event {
//!       tauri::UpdaterEvent::UpdateAvailable { body, date, version } => {
//!         println!("update available {} {:?} {}", body, date, version);
//!       }
//!       _ => (),
//!     }
//!   }
//!   _ => {}
//! });
//! ```
//!
//! #### Javascript
//! ```js
//! import { listen } from "@tauri-apps/api/event";
//! listen("tauri://update-available", function (res) {
//!     console.log("New version available: ", res);
//! });
//! ```
//!
//! ### Emit Install and Download
//!
//! You need to emit this event to initialize the download and listen to the [install progress](#listen-install-progress).
//!
//! Event : `tauri://update-install`
//!
//! #### Rust
//! ```no_run
//! tauri::Builder::default()
//!   .setup(|app| {
//!     let handle = app.handle();
//!     tauri::async_runtime::spawn(async move {
//!       match handle.updater().check().await {
//!         Ok(update) => {
//!           if update.is_update_available() {
//!             update.download_and_install().await.unwrap();
//!           }
//!         }
//!         Err(e) => {
//!           println!("failed to update: {}", e);
//!         }
//!       }
//!     });
//!     Ok(())
//!   });
//! ```
//!
//! #### Javascript
//! ```js
//! import { emit } from "@tauri-apps/api/event";
//! emit("tauri://update-install");
//! ```
//!
//! ### Listen Download Progress
//!
//! The event payload informs the length of the chunk that was just downloaded, and the total download size if known.
//!
//! #### Rust
//! ```no_run
//! let app = tauri::Builder::default()
//!   // on an actual app, remove the string argument
//!   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
//!   .expect("error while building tauri application");
//! app.run(|_app_handle, event| match event {
//!   tauri::RunEvent::Updater(updater_event) => {
//!     match updater_event {
//!       tauri::UpdaterEvent::DownloadProgress { chunk_length, content_length } => {
//!         println!("downloaded {} of {:?}", chunk_length, content_length);
//!       }
//!       _ => (),
//!     }
//!   }
//!   _ => {}
//! });
//! ```
//!
//! #### Javascript
//!
//! Event : `tauri://update-download-progress`
//!
//! Emitted data:
//! ```text
//! chunkLength       number
//! contentLength     number/null
//! ```
//!
//! ```js
//! import { listen } from "@tauri-apps/api/event";
//! listen<{ chunkLength: number, contentLength?: number }>("tauri://update-download-progress", function (event) {
//!     console.log(`downloaded ${event.payload.chunkLength} of ${event.payload.contentLength}`);
//! });
//! ```
//!
//! ### Listen Install Progress
//!
//! **Pending** is emitted when the download is started and **Done** when the install is complete. You can then ask to restart the application.
//!
//! **Downloaded** is emitted when the download has finished.
//!
//! **UpToDate** is emitted when the app already has the latest version installed and an update is not needed.
//!
//! **Error** is emitted when there is an error with the updater. We suggest to listen to this event even if the dialog is enabled.
//!
//! #### Rust
//! ```no_run
//! let app = tauri::Builder::default()
//!   // on an actual app, remove the string argument
//!   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
//!   .expect("error while building tauri application");
//! app.run(|_app_handle, event| match event {
//!   tauri::RunEvent::Updater(updater_event) => {
//!     match updater_event {
//!       tauri::UpdaterEvent::UpdateAvailable { body, date, version } => {
//!         println!("update available {} {:?} {}", body, date, version);
//!       }
//!       tauri::UpdaterEvent::Pending => {
//!         println!("update is pending!");
//!       }
//!       tauri::UpdaterEvent::Downloaded => {
//!         println!("update has been downloaded!");
//!       }
//!       tauri::UpdaterEvent::Updated => {
//!         println!("app has been updated");
//!       }
//!       tauri::UpdaterEvent::AlreadyUpToDate => {
//!         println!("app is already up to date");
//!       }
//!       tauri::UpdaterEvent::Error(error) => {
//!         println!("failed to update: {}", error);
//!       }
//!       _ => (),
//!     }
//!   }
//!   _ => {}
//! });
//! ```
//!
//! #### Javascript
//! Event : `tauri://update-status`
//!
//! Emitted data:
//! ```text
//! status    ERROR | PENDING | UPTODATE | DONE
//! error     string/null
//! ```
//!
//! ```js
//! import { listen } from "@tauri-apps/api/event";
//! listen<{ status: string, error?: string }>("tauri://update-status", function (res) {
//!     console.log("New status: ", res);
//! });
//! ```
//!
//! # Server Support
//!
//! Your server should determine whether an update is required based on the [Update Request](#update-requests) your client issues.
//!
//! If an update is required your server should respond with a status code of [200 OK](http://tools.ietf.org/html/rfc2616#section-10.2.1) and include the [update JSON](#update-server-json-format) in the body. To save redundantly downloading the same version multiple times your server must not inform the client to update.
//!
//! If no update is required your server must respond with a status code of [204 No Content](http://tools.ietf.org/html/rfc2616#section-10.2.5).
//!
//! ## Update Server JSON Format
//!
//! When an update is available, Tauri expects the following schema in response to the update request provided:
//!
//! ```json
//! {
//!     "url": "https://mycompany.example.com/myapp/releases/myrelease.tar.gz",
//!     "version": "0.0.1",
//!     "notes": "Theses are some release notes",
//!     "pub_date": "2020-09-18T12:29:53+01:00",
//!     "signature": ""
//! }
//! ```
//!
//! The only required keys are "url" and "version", the others are optional.
//!
//! "pub_date" if present must be formatted according to ISO 8601.
//!
//! "signature" if present must be a valid signature generated with Tauri cli. See [Signing updates](#signing-updates).
//!
//! ## Update File JSON Format
//!
//! The alternate update technique uses a plain JSON file meaning you can store your update metadata on S3, gist, or another static file store. Tauri will check against the name/version field and if the version is smaller than the current one and the platform is available, the update will be triggered. The format of this file is detailed below:
//!
//! ```json
//! {
//!   "name":"v1.0.0",
//!   "notes":"Test version",
//!   "pub_date":"2020-06-22T19:25:57Z",
//!   "platforms": {
//!     "darwin-aarch64": {
//!       "signature":"",
//!       "url":"https://github.com/tauri-apps/tauri-test/releases/download/v1.0.0/app-aarch64.app.tar.gz"
//!     },
//!     "darwin-intel": {
//!       "signature":"",
//!       "url":"https://github.com/tauri-apps/tauri-test/releases/download/v1.0.0/app-x86_64.app.tar.gz"
//!     },
//!     "linux-x86_64": {
//!       "signature":"",
//!       "url":"https://github.com/tauri-apps/tauri-test/releases/download/v1.0.0/app.AppImage.tar.gz"
//!     },
//!     "windows-x86_64": {
//!       "signature":"",
//!       "url":"https://github.com/tauri-apps/tauri-test/releases/download/v1.0.0/app.x64.msi.zip"
//!     },
//!     "windows-i686": {
//!       "signature":"",
//!       "url":"https://github.com/tauri-apps/tauri-test/releases/download/v1.0.0/app.x86.msi.zip"
//!     }
//!   }
//! }
//! ```
//!
//!
//! # Bundler (Artifacts)
//!
//! The Tauri bundler will automatically generate update artifacts if the updater is enabled in `tauri.conf.json`
//!
//! If the bundler can locate your private and pubkey, your update artifacts will be automatically signed.
//!
//! The signature can be found in the `sig` file. The signature can be uploaded to GitHub safely or made public as long as your private key is secure.
//!
//! You can see how it's [bundled with the CI](https://github.com/tauri-apps/tauri/blob/feature/new_updater/.github/workflows/artifacts-updater.yml#L44) and a [sample tauri.conf.json](https://github.com/tauri-apps/tauri/blob/feature/new_updater/examples/updater/src-tauri/tauri.conf.json#L52)
//!
//! ## macOS
//!
//! On MACOS we create a .tar.gz from the whole application. (.app)
//!
//! ```text
//! target/release/bundle
//! └── osx
//!     └── app.app
//!     └── app.app.tar.gz (update bundle)
//!     └── app.app.tar.gz.sig (if signature enabled)
//! ```
//!
//! ## Windows
//!
//! On Windows we create a .zip from the MSI, when downloaded and validated, we run the MSI install.
//!
//! ```text
//! target/release
//! └── app.x64.msi
//! └── app.x64.msi.zip (update bundle)
//! └── app.x64.msi.zip.sig (if signature enabled)
//! ```
//!
//! ## Linux
//!
//! On Linux, we create a .tar.gz from the AppImage.
//!
//! ```text
//! target/release/bundle
//! └── appimage
//!     └── app.AppImage
//!     └── app.AppImage.tar.gz (update bundle)
//!     └── app.AppImage.tar.gz.sig (if signature enabled)
//! ```
//!
//! # Signing updates
//!
//! We offer a built-in signature to ensure your update is safe to be installed.
//!
//! To sign your updates, you need two things.
//!
//! The *Public-key* (pubkey) should be added inside your `tauri.conf.json` to validate the update archive before installing.
//!
//! The *Private key* (privkey) is used to sign your update and should NEVER be shared with anyone. Also, if you lost this key, you'll NOT be able to publish a new update to the current user base (if pubkey is set in tauri.conf.json). It's important to save it at a safe place and you can always access it.
//!
//! To generate your keys you need to use the Tauri cli.
//!
//! ```bash
//! # Generate the keys, follow the instructions
//! tauri signer generate
//! tauri signer sign -g -w ~/.tauri/myapp.key
//! ```
//!
//! You have multiple options available
//! ```bash
//! Tauri updates signer.
//!
//! USAGE:
//!     tauri signer sign [FLAGS] [OPTIONS]
//!
//! FLAGS:
//!         --force          Overwrite private key even if it exists on the specified path
//!     -g, --generate       Generate keypair to sign files
//!     -h, --help           Prints help information
//!         --no-password    Set empty password for your private key
//!     -V, --version        Prints version information
//!
//! OPTIONS:
//!     -p, --password <password>                    Set private key password when signing
//!     -k, --private-key <private-key>              Load the private key from a string
//!     -f, --private-key-path <private-key-path>    Load the private key from a file
//!         --sign-file <sign-file>                  Sign the specified file
//!     -w, --write-keys <write-keys>                Write private key to a file
//! ```
//!
//! ***
//!
//! Environment variables used to sign with `tauri-bundler`:
//! If they are set, and `tauri.conf.json` expose the public key, the bundler will automatically generate and sign the updater artifacts.
//!
//! `TAURI_PRIVATE_KEY`  Path or String of your private key
//!
//! `TAURI_KEY_PASSWORD`  Your private key password (optional)

mod core;
mod error;

use std::time::Duration;

use http::header::{HeaderName, HeaderValue};
use semver::Version;
use time::OffsetDateTime;

pub use self::{core::RemoteRelease, error::Error};
/// Alias for [`std::result::Result`] using our own [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

use crate::{
  api::dialog::blocking::ask, runtime::EventLoopProxy, AppHandle, EventLoopMessage, Manager,
  Runtime, UpdaterEvent,
};

/// Check for new updates
pub const EVENT_CHECK_UPDATE: &str = "tauri://update";
/// New update available
pub const EVENT_UPDATE_AVAILABLE: &str = "tauri://update-available";
/// Used to initialize an update *should run check-update first (once you received the update available event)*
pub const EVENT_INSTALL_UPDATE: &str = "tauri://update-install";
/// Send updater status or error even if dialog is enabled, you should
/// always listen for this event. It'll send you the install progress
/// and any error triggered during update check and install
pub const EVENT_STATUS_UPDATE: &str = "tauri://update-status";
/// The name of the event that is emitted on download progress.
pub const EVENT_DOWNLOAD_PROGRESS: &str = "tauri://update-download-progress";
/// this is the status emitted when the download start
pub const EVENT_STATUS_PENDING: &str = "PENDING";
/// When you got this status, something went wrong
/// you can find the error message inside the `error` field.
pub const EVENT_STATUS_ERROR: &str = "ERROR";
/// The update has been downloaded.
pub const EVENT_STATUS_DOWNLOADED: &str = "DOWNLOADED";
/// When you receive this status, you should ask the user to restart
pub const EVENT_STATUS_SUCCESS: &str = "DONE";
/// When you receive this status, this is because the application is running last version
pub const EVENT_STATUS_UPTODATE: &str = "UPTODATE";

/// Gets the target string used on the updater.
pub fn target() -> Option<String> {
  if let (Some(target), Some(arch)) = (core::get_updater_target(), core::get_updater_arch()) {
    Some(format!("{}-{}", target, arch))
  } else {
    None
  }
}

#[derive(Clone, serde::Serialize)]
struct StatusEvent {
  status: String,
  error: Option<String>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgressEvent {
  chunk_length: usize,
  content_length: Option<u64>,
}

#[derive(Clone, serde::Serialize)]
struct UpdateManifest {
  version: String,
  date: Option<String>,
  body: String,
}

/// An update check builder.
#[derive(Debug)]
pub struct UpdateBuilder<R: Runtime> {
  inner: core::UpdateBuilder<R>,
  events: bool,
}

impl<R: Runtime> UpdateBuilder<R> {
  /// Do not use the event system to emit information or listen to install the update.
  pub fn skip_events(mut self) -> Self {
    self.events = false;
    self
  }

  /// Sets the current platform's target name for the updater.
  ///
  /// The target is injected in the endpoint URL by replacing `{{target}}`.
  /// Note that this does not affect the `{{arch}}` variable.
  ///
  /// If the updater response JSON includes the `platforms` field,
  /// that object must contain a value for the target key.
  ///
  /// By default Tauri uses `$OS_NAME` as the replacement for `{{target}}`
  /// and `$OS_NAME-$ARCH` as the key in the `platforms` object,
  /// where `$OS_NAME` is the current operating system name "linux", "windows" or "darwin")
  /// and `$ARCH` is one of the supported architectures ("i686", "x86_64", "armv7" or "aarch64").
  ///
  /// See [`Builder::updater_target`](crate::Builder#method.updater_target) for a way to set the target globally.
  ///
  /// # Examples
  ///
  /// ## Use a macOS Universal binary target name
  ///
  /// In this example, we set the updater target only on macOS.
  /// On other platforms, we set the default target.
  /// Note that `{{target}}` will be replaced with `darwin-universal`,
  /// but `{{arch}}` is still the running platform's architecture.
  ///
  /// ```no_run
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     tauri::async_runtime::spawn(async move {
  ///       let builder = tauri::updater::builder(handle).target(if cfg!(target_os = "macos") {
  ///         "darwin-universal".to_string()
  ///       } else {
  ///         tauri::updater::target().unwrap()
  ///       });
  ///       match builder.check().await {
  ///         Ok(update) => {}
  ///         Err(error) => {}
  ///       }
  ///     });
  ///     Ok(())
  ///   });
  /// ```
  ///
  /// ## Append debug information to the target
  ///
  /// This allows you to provide updates for both debug and release applications.
  ///
  /// ```no_run
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     tauri::async_runtime::spawn(async move {
  ///       let kind = if cfg!(debug_assertions) { "debug" } else { "release" };
  ///       let builder = tauri::updater::builder(handle).target(format!("{}-{}", tauri::updater::target().unwrap(), kind));
  ///       match builder.check().await {
  ///         Ok(update) => {}
  ///         Err(error) => {}
  ///       }
  ///     });
  ///     Ok(())
  ///   });
  /// ```
  ///
  /// ## Use the platform's target triple
  ///
  /// ```no_run
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     tauri::async_runtime::spawn(async move {
  ///       let builder = tauri::updater::builder(handle).target(tauri::utils::platform::target_triple().unwrap());
  ///       match builder.check().await {
  ///         Ok(update) => {}
  ///         Err(error) => {}
  ///       }
  ///     });
  ///     Ok(())
  ///   });
  /// ```
  pub fn target(mut self, target: impl Into<String>) -> Self {
    self.inner = self.inner.target(target);
    self
  }

  /// Sets a closure that is invoked to compare the current version and the latest version returned by the updater server.
  /// The first argument is the current version, and the second one is the latest version.
  ///
  /// The closure must return `true` if the update should be installed.
  ///
  /// # Examples
  ///
  /// - Always install the version returned by the server:
  ///
  /// ```no_run
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     tauri::updater::builder(app.handle()).should_install(|_current, _latest| true);
  ///     Ok(())
  ///   });
  /// ```
  pub fn should_install<F: FnOnce(&Version, &RemoteRelease) -> bool + Send + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.inner = self.inner.should_install(f);
    self
  }

  /// Sets the timeout for the requests to the updater endpoints.
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.inner = self.inner.timeout(timeout);
    self
  }

  /// Add a `Header` to the request.
  pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
  where
    HeaderName: TryFrom<K>,
    <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
    HeaderValue: TryFrom<V>,
    <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
  {
    self.inner = self.inner.header(key, value)?;
    Ok(self)
  }

  /// Check if an update is available.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     tauri::async_runtime::spawn(async move {
  ///       match tauri::updater::builder(handle).check().await {
  ///         Ok(update) => {}
  ///         Err(error) => {}
  ///       }
  ///     });
  ///     Ok(())
  ///   });
  /// ```
  pub async fn check(self) -> Result<UpdateResponse<R>> {
    let handle = self.inner.app.clone();
    let events = self.events;
    // check updates
    match self.inner.build().await {
      Ok(update) => {
        if events {
          // send notification if we need to update
          if update.should_update {
            let body = update.body.clone().unwrap_or_else(|| String::from(""));

            // Emit `tauri://update-available`
            let _ = handle.emit_all(
              EVENT_UPDATE_AVAILABLE,
              UpdateManifest {
                body: body.clone(),
                date: update.date.map(|d| d.to_string()),
                version: update.version.clone(),
              },
            );
            let _ = handle.create_proxy().send_event(EventLoopMessage::Updater(
              UpdaterEvent::UpdateAvailable {
                body,
                date: update.date,
                version: update.version.clone(),
              },
            ));

            // Listen for `tauri://update-install`
            let update_ = update.clone();
            handle.once_global(EVENT_INSTALL_UPDATE, move |_msg| {
              crate::async_runtime::spawn(async move {
                let _ = download_and_install(update_).await;
              });
            });
          } else {
            send_status_update(&handle, UpdaterEvent::AlreadyUpToDate);
          }
        }
        Ok(UpdateResponse { update })
      }
      Err(e) => {
        if self.events {
          send_status_update(&handle, UpdaterEvent::Error(e.to_string()));
        }
        Err(e)
      }
    }
  }
}

/// The response of an updater check.
pub struct UpdateResponse<R: Runtime> {
  update: core::Update<R>,
}

impl<R: Runtime> Clone for UpdateResponse<R> {
  fn clone(&self) -> Self {
    Self {
      update: self.update.clone(),
    }
  }
}

impl<R: Runtime> UpdateResponse<R> {
  /// Whether the updater found a newer release or not.
  pub fn is_update_available(&self) -> bool {
    self.update.should_update
  }

  /// The current version of the application as read by the updater.
  pub fn current_version(&self) -> &Version {
    &self.update.current_version
  }

  /// The latest version of the application found by the updater.
  pub fn latest_version(&self) -> &str {
    &self.update.version
  }

  /// The update date.
  pub fn date(&self) -> Option<&OffsetDateTime> {
    self.update.date.as_ref()
  }

  /// The update description.
  pub fn body(&self) -> Option<&String> {
    self.update.body.as_ref()
  }

  /// Downloads and installs the update.
  pub async fn download_and_install(self) -> Result<()> {
    download_and_install(self.update).await
  }
}

/// Check if there is any new update with builtin dialog.
pub(crate) async fn check_update_with_dialog<R: Runtime>(handle: AppHandle<R>) {
  let updater_config = handle.config().tauri.updater.clone();
  let package_info = handle.package_info().clone();
  if let Some(endpoints) = updater_config.endpoints.clone() {
    let endpoints = endpoints
      .iter()
      .map(|e| e.to_string())
      .collect::<Vec<String>>();

    let mut builder = self::core::builder(handle.clone())
      .urls(&endpoints[..])
      .current_version(package_info.version);
    if let Some(target) = &handle.updater_settings.target {
      builder = builder.target(target);
    }

    // check updates
    match builder.build().await {
      Ok(updater) => {
        let pubkey = updater_config.pubkey.clone();

        // if dialog enabled only
        if updater.should_update && updater_config.dialog {
          let body = updater.body.clone().unwrap_or_else(|| String::from(""));
          let dialog =
            prompt_for_install(&updater.clone(), &package_info.name, &body.clone(), pubkey).await;

          if let Err(e) = dialog {
            send_status_update(&handle, UpdaterEvent::Error(e.to_string()));
          }
        }
      }
      Err(e) => {
        send_status_update(&handle, UpdaterEvent::Error(e.to_string()));
      }
    }
  }
}

/// Updater listener
/// This function should be run on the main thread once.
pub(crate) fn listener<R: Runtime>(handle: AppHandle<R>) {
  // Wait to receive the event `"tauri://update"`
  let handle_ = handle.clone();
  handle.listen_global(EVENT_CHECK_UPDATE, move |_msg| {
    let handle_ = handle_.clone();
    crate::async_runtime::spawn(async move {
      let _ = builder(handle_.clone()).check().await;
    });
  });
}

pub(crate) async fn download_and_install<R: Runtime>(update: core::Update<R>) -> Result<()> {
  // Start installation
  // emit {"status": "PENDING"}
  send_status_update(&update.app, UpdaterEvent::Pending);

  let handle = update.app.clone();
  let handle_ = handle.clone();

  // Launch updater download process
  // macOS we display the `Ready to restart dialog` asking to restart
  // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
  // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
  let update_result = update
    .download_and_install(
      update.app.config().tauri.updater.pubkey.clone(),
      move |chunk_length, content_length| {
        send_download_progress_event(&handle, chunk_length, content_length);
      },
      move || {
        send_status_update(&handle_, UpdaterEvent::Downloaded);
      },
    )
    .await;

  if let Err(err) = &update_result {
    // emit {"status": "ERROR", "error": "The error message"}
    send_status_update(&update.app, UpdaterEvent::Error(err.to_string()));
  } else {
    // emit {"status": "DONE"}
    send_status_update(&update.app, UpdaterEvent::Updated);
  }
  update_result
}

/// Initializes the [`UpdateBuilder`] using the app configuration.
pub fn builder<R: Runtime>(handle: AppHandle<R>) -> UpdateBuilder<R> {
  let updater_config = &handle.config().tauri.updater;
  let package_info = handle.package_info().clone();

  // prepare our endpoints
  let endpoints = updater_config
    .endpoints
    .as_ref()
    .expect("Something wrong with endpoints")
    .iter()
    .map(|e| e.to_string())
    .collect::<Vec<String>>();

  let mut builder = self::core::builder(handle.clone())
    .urls(&endpoints[..])
    .current_version(package_info.version);
  if let Some(target) = &handle.updater_settings.target {
    builder = builder.target(target);
  }
  UpdateBuilder {
    inner: builder,
    events: true,
  }
}

// Send a status update via `tauri://update-download-progress` event.
fn send_download_progress_event<R: Runtime>(
  handle: &AppHandle<R>,
  chunk_length: usize,
  content_length: Option<u64>,
) {
  let _ = handle.emit_all(
    EVENT_DOWNLOAD_PROGRESS,
    DownloadProgressEvent {
      chunk_length,
      content_length,
    },
  );
  let _ =
    handle
      .create_proxy()
      .send_event(EventLoopMessage::Updater(UpdaterEvent::DownloadProgress {
        chunk_length,
        content_length,
      }));
}

// Send a status update via `tauri://update-status` event.
fn send_status_update<R: Runtime>(handle: &AppHandle<R>, message: UpdaterEvent) {
  let _ = handle.emit_all(
    EVENT_STATUS_UPDATE,
    if let UpdaterEvent::Error(error) = &message {
      StatusEvent {
        error: Some(error.clone()),
        status: message.clone().status_message().into(),
      }
    } else {
      StatusEvent {
        error: None,
        status: message.clone().status_message().into(),
      }
    },
  );
  let _ = handle
    .create_proxy()
    .send_event(EventLoopMessage::Updater(message));
}

// Prompt a dialog asking if the user want to install the new version
// Maybe we should add an option to customize it in future versions.
async fn prompt_for_install<R: Runtime>(
  update: &self::core::Update<R>,
  app_name: &str,
  body: &str,
  pubkey: String,
) -> Result<()> {
  let windows = update.app.windows();
  let parent_window = windows.values().next();

  // todo(lemarier): We should review this and make sure we have
  // something more conventional.
  let should_install = ask(
    parent_window,
    format!(r#"A new version of {} is available! "#, app_name),
    format!(
      r#"{} {} is now available -- you have {}.

Would you like to install it now?

Release Notes:
{}"#,
      app_name, update.version, update.current_version, body,
    ),
  );

  if should_install {
    // Launch updater download process
    // macOS we display the `Ready to restart dialog` asking to restart
    // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
    // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
    update
      .download_and_install(pubkey.clone(), |_, _| (), || ())
      .await?;

    // Ask user if we need to restart the application
    let should_exit = ask(
      parent_window,
      "Ready to Restart",
      "The installation was successful, do you want to restart the application now?",
    );
    if should_exit {
      update.app.restart();
    }
  }

  Ok(())
}
