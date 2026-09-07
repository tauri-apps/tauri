// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A Tauri application running on the Chromium Embedded Framework, showcasing
//! the APIs `tauri-runtime-cef` adds on top of the portable Tauri ones.
//!
//! The CEF-only surface lives in four places:
//!
//! - [`tauri_runtime_cef::Cef`], the runtime attributes passed to
//!   `tauri::Builder::runtime`, in [`runtime_config`].
//! - [`tauri_runtime_cef::WebviewWindowBuilderCefExt`], the extension trait that
//!   adds CEF methods to `tauri::WebviewWindowBuilder`, used below and in
//!   [`commands`].
//! - [`tauri_runtime_cef::WebviewBuilderCefExt`], the same methods on the
//!   `tauri::webview::WebviewBuilder` of a window hosting several webviews,
//!   behind Tauri's `unstable` feature, in [`commands`].
//! - [`tauri_runtime_cef::WebviewCefExt`], the extension trait that adds CEF
//!   methods to a built `tauri::Webview` and `tauri::WebviewWindow`.
//!
//! The extension traits are implemented both for the statically typed
//! `CefRuntime` and for `tauri::DynRuntime`, which is what
//! `tauri::Builder::default()` uses, so nothing here has to name the runtime
//! type. On a non-CEF runtime the same calls fail with
//! `tauri_runtime::Error::RuntimeTypeMismatch` instead of failing to compile.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod events;
mod runtime_config;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use tauri::{
  AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
  webview::{NewWindowResponse, PermissionKind, PermissionResponse},
};
use tauri_runtime_cef::{
  AsCefWindowOpener, RuntimeStyle, WebviewCefExt, WebviewWindowBuilderCefExt,
};

use events::{CefEvent, EventSink};
use runtime_config::RuntimeConfig;

/// Every CEF application is one executable that is also its own renderer, GPU,
/// network and utility process. This attribute runs the helper side of that and
/// returns before the Tauri application is ever built, for any process Chromium
/// launched with a `--type=` switch.
#[tauri_runtime_cef::cef_entry_point]
fn main() {
  let config = RuntimeConfig::from_env();

  tauri::Builder::default()
    // Selecting the runtime. `Cef` is `tauri-runtime-cef`'s implementation of
    // `RuntimeInitAttrs`, so this both picks CEF and configures it.
    .runtime(config.cef())
    .manage(config)
    .invoke_handler(tauri::generate_handler![
      commands::runtime_info,
      commands::native_snapshot,
      commands::send_devtools_message,
      commands::open_accelerator_window,
      commands::open_child_webviews_window,
    ])
    .setup(|app| {
      let sink = EventSink::new(app.handle());
      app.manage(sink.clone());
      // Remembers which native window each label was last sampled under, so the
      // native state panel can tell a re-sample of the same window apart from a
      // replacement that merely reuses the label.
      app.manage(Arc::new(commands::ObservedWindows::default()));
      build_main_window(app.handle(), &sink)?;
      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while building the tauri application")
    .run(|app, event| {
      // A second launch of an app that is already running reaches Chromium's
      // process singleton, which relays the new process's command line. CEF
      // clears that command line, and the runtime puts the deep link URL back on
      // it, so `tauri-cef-example://...` still arrives here.
      if let RunEvent::Opened { urls } = event
        && let Some(sink) = app.try_state::<EventSink>()
      {
        sink.send(CefEvent::DeepLink {
          urls: urls.iter().map(ToString::to_string).collect(),
        });
      }
    });
}

fn build_main_window(app: &AppHandle, sink: &EventSink) -> tauri::Result<()> {
  let console_sink = sink.clone();
  let frame_sink = sink.clone();
  let permission_sink = sink.clone();
  let popup_sink = sink.clone();
  let popup_app = app.clone();

  let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
    .title("Tauri × CEF")
    .inner_size(1180., 880.)
    .min_inner_size(800., 600.)
    // CEF picks a style when none is named. Chrome style is the one this
    // runtime's defaults are written for; Alloy style has no Chrome UI at all,
    // and so neither its accelerator table nor its desktop media picker.
    .browser_runtime_style(RuntimeStyle::Chrome)
    // Synchronous observers on CEF's UI thread: hand the observation off and
    // return. See the module docs of `events`.
    .on_console_message(move |message| console_sink.console("main", message))
    .on_frame_event(move |event| frame_sink.frame("main", event))
    .on_permission_request(move |_webview, kind| {
      let (response, note) = decide_permission(kind);
      permission_sink.send(CefEvent::Permission {
        permission: format!("{kind:?}"),
        response: format!("{response:?}"),
        note: note.to_string(),
      });
      response
    })
    .on_new_window(move |url, features| {
      static NEXT: AtomicU32 = AtomicU32::new(1);
      let label = format!("popup-{}", NEXT.fetch_add(1, Ordering::Relaxed));

      // CEF hands the opener's main-frame URL to this callback directly. Reading
      // it from a blocking webview getter instead can deadlock the UI thread,
      // because CEF's external message pump may run outside a winit dispatch.
      let opener_source = features
        .opener()
        .as_cef_window_opener()
        .and_then(|opener| opener.source_url())
        .map(ToString::to_string);

      // `Allow` leaves the popup to CEF, which opens a browser of its own: a
      // separate native browser with no Tauri window label, no observers of
      // ours on it, and which the opener sees under `Webview::popups()`. The
      // example asks for one through a URL fragment so both paths can be tried.
      let cef_owned = url.fragment() == Some("cef-owned");

      popup_sink.send(CefEvent::Popup {
        url: url.to_string(),
        opener_source,
        label: if cef_owned {
          "(CEF-owned, no Tauri label)".to_string()
        } else {
          label.clone()
        },
      });

      if cef_owned {
        return NewWindowResponse::Allow;
      }

      // Answering with a window of our own keeps the popup a Tauri window, with
      // the size and position the page asked for.
      match WebviewWindowBuilder::new(&popup_app, label, WebviewUrl::External(url.clone()))
        .window_features(features)
        .title(url.as_str())
        .build()
      {
        Ok(window) => NewWindowResponse::Create { window },
        Err(_) => NewWindowResponse::Deny,
      }
    })
    .build()?;

  // Registered after the browser exists, and scoped to that one native browser:
  // a CEF-owned popup is a separate browser whose traffic never arrives here.
  let devtools_sink = sink.clone();
  window.on_dev_tools_protocol(move |protocol| devtools_sink.dev_tools(protocol))?;

  Ok(())
}

/// Answers a permission request, with the three CEF specifics worth knowing.
fn decide_permission(kind: PermissionKind) -> (PermissionResponse, &'static str) {
  match kind {
    // Chromium consults the prompt only while the stored content setting still
    // says "ask", and answering persists the decision to the on-disk profile, so
    // this handler runs once per origin and permission — across restarts too.
    PermissionKind::Notifications | PermissionKind::Geolocation => (
      PermissionResponse::Allow,
      "granted without Chrome's prompt; the decision persists for this origin",
    ),
    // The exception: every getUserMedia() call goes through the media path, so
    // camera and microphone do reach the handler again and a changing answer is
    // honored.
    PermissionKind::Camera | PermissionKind::Microphone => (
      PermissionResponse::Deny,
      "denied; getUserMedia reaches the handler on every call",
    ),
    // An `Allow` is deliberately not honored here. CEF builds the stream from the
    // permission mask, and a desktop video bit with no named source synthesises
    // the whole desktop with no picker at all, so the runtime hands the request
    // back to Chromium: its picker is the only thing that can say what is shared.
    PermissionKind::DisplayCapture => (
      PermissionResponse::Default,
      "handed back to Chromium: an Allow here would share the whole desktop",
    ),
    // Storage Access, FedCM, protocol handler registration, idle detection,
    // WebXR and everything else Tauri has no kind for arrives as `Other`.
    // Hard-denying that whole set breaks third-party SSO, so answer `Default`
    // for what you did not mean to decide about.
    _ => (
      PermissionResponse::Default,
      "left to Chromium's own handling",
    ),
  }
}
