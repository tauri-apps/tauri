// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The commands the frontend calls into the CEF-only APIs with.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{
  AppHandle, LogicalPosition, LogicalSize, Manager, State, WebviewUrl, WebviewWindowBuilder,
  webview::WebviewBuilder, window::WindowBuilder,
};
use tauri_runtime_cef::{
  ChromeCommandGroup, NativeDialogObservation, NativeWindowToken, RuntimeStyle,
  WebviewBuilderCefExt, WebviewCefExt, WebviewWindowBuilderCefExt, allocate_devtools_message_id,
  cef::{ImplBrowser, ImplBrowserHost, ImplFrame},
};

use crate::events::EventSink;
use crate::runtime_config::{ConfiguredValue, RuntimeConfig};

/// How long a native observation may take before the command gives up. The CEF
/// UI thread answers in the same event loop turn; a timeout means something is
/// wedged rather than slow.
const NATIVE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
  /// The Chromium version CEF is built on, from `RuntimeHandle::webview_version`.
  chrome_version: String,
  /// The CEF API version this build of the runtime declares.
  cef_api_version: i32,
  configuration: Vec<ConfiguredValue>,
}

#[tauri::command]
pub fn runtime_info(app: AppHandle, config: State<'_, RuntimeConfig>) -> RuntimeInfo {
  RuntimeInfo {
    chrome_version: app
      .webview_version()
      .unwrap_or_else(|error| error.to_string()),
    cef_api_version: config.cef_api_version(),
    configuration: config.describe(),
  }
}

/// The native JavaScript dialog CEF observed on a browser, if any.
///
/// The runtime learns about dialogs from the DevTools `Page` domain it already
/// enables, so it reports only what the protocol states: which kind of dialog
/// opened and whether Chromium has a browser-side handler for it. The message
/// text and the prompt value are page content and are deliberately not retained.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogInfo {
  /// A dialog event was observed for the document in this snapshot. `false`
  /// never means "no dialog", only "nothing was observed".
  known: bool,
  kind: Option<String>,
  has_browser_handler: Option<bool>,
}

impl From<&NativeDialogObservation> for DialogInfo {
  fn from(observation: &NativeDialogObservation) -> Self {
    Self {
      known: observation.known,
      kind: observation
        .dialog
        .as_ref()
        .map(|dialog| format!("{:?}", dialog.kind)),
      has_browser_handler: observation
        .dialog
        .as_ref()
        .map(|dialog| dialog.has_browser_handler),
    }
  }
}

/// What the raw [`cef::Browser`] behind the webview answers.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserInfo {
  /// `browser.main_frame().url()`, straight from CEF rather than from Tauri's
  /// bookkeeping.
  main_frame_url: Option<String>,
  frame_count: usize,
  is_loading: bool,
  /// The browser host's zoom level: 0.0 at 100%, one step per notch.
  zoom_level: f64,
  dev_tools_open: bool,
}

impl BrowserInfo {
  /// Reads the native handle [`tauri_runtime_cef::Webview::browser`] returns.
  /// Everything here is live CEF state, not part of the sampled snapshot.
  fn new(browser: &tauri_runtime_cef::cef::Browser) -> Self {
    let host = browser.host();
    Self {
      main_frame_url: browser
        .main_frame()
        .map(|frame| tauri_runtime_cef::cef::CefString::from(&frame.url()).to_string()),
      frame_count: browser.frame_count(),
      is_loading: browser.is_loading() != 0,
      zoom_level: host.as_ref().map(ImplBrowserHost::zoom_level).unwrap_or(0.),
      dev_tools_open: host.is_some_and(|host| host.has_dev_tools() != 0),
    }
  }
}

/// A CEF-owned popup: a native browser of its own, with no Tauri window label.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupInfo {
  browser_id: i32,
  opened_by_this_browser: bool,
  visible: Option<bool>,
  /// A CEF-owned popup has no Tauri label of its own, so whatever is observed
  /// of it — its dialogs included — is observed here, on its opener's snapshot.
  dialogs: DialogInfo,
  /// Whether `Webview::for_document` selects this popup back out of the family
  /// when handed the popup's own document token. `None` when the popup admitted
  /// no document to be selected by.
  selected_by_its_document: Option<bool>,
  browser: BrowserInfo,
}

/// The native state CEF sampled for one `with_webview` callback.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
  browser_id: i32,
  window_label: Option<String>,
  /// Whether the runtime could observe a native window lifetime at all, and
  /// whether it is the same one the previous sample of this label saw.
  native_window_observed: bool,
  same_native_window_as_previous_sample: Option<bool>,
  document_admitted: bool,
  /// Whether `Webview::for_document` selects this browser back out of the family
  /// when handed this snapshot's own document token.
  selected_by_its_document: Option<bool>,
  parent_matches: Option<bool>,
  visible: Option<bool>,
  bounds: Option<serde_json::Value>,
  dialogs: DialogInfo,
  browser: BrowserInfo,
  popups: Vec<PopupInfo>,
}

impl SnapshotInfo {
  fn new(webview: &tauri_runtime_cef::Webview, previous: Option<NativeWindowToken>) -> Self {
    let snapshot = webview.snapshot();
    Self {
      browser_id: snapshot.browser_id,
      // A CEF-owned popup has no Tauri window label.
      window_label: snapshot.window_label.clone(),
      // The label is not the identity: a window closed and rebuilt under the
      // same label is a different native window, and only the token says so.
      native_window_observed: snapshot.window.is_some(),
      same_native_window_as_previous_sample: snapshot
        .window
        .as_ref()
        .zip(previous.as_ref())
        .map(|(current, previous)| current == previous),
      // `None` means no all-frame document generation could be admitted: some
      // frame is detached or the load is not complete.
      document_admitted: snapshot.document.is_some(),
      // The document token names one document of one browser in this family, so
      // handing back the token of this very snapshot selects this very webview.
      // A caller that carried a token from an earlier callback — the point of
      // the type — finds out here whether it still names anything.
      selected_by_its_document: snapshot.document.as_ref().map(|document| {
        webview
          .for_document(document)
          .is_some_and(|selected| selected.snapshot().browser_id == snapshot.browser_id)
      }),
      parent_matches: snapshot.parent_matches,
      visible: snapshot.visible,
      bounds: snapshot
        .bounds
        .and_then(|bounds| serde_json::to_value(bounds).ok()),
      dialogs: (&snapshot.dialogs).into(),
      browser: BrowserInfo::new(&webview.browser()),
      popups: webview
        .popups()
        .iter()
        .map(|popup| {
          let popup_snapshot = popup.snapshot();
          PopupInfo {
            browser_id: popup_snapshot.browser_id,
            opened_by_this_browser: popup
              .opener()
              .is_some_and(|opener| opener.is_same_browser(webview.frame_navigation_state())),
            visible: popup_snapshot.visible,
            dialogs: (&popup_snapshot.dialogs).into(),
            // Selection walks the whole family, so the opener resolves a popup's
            // token even though the token came off the popup's own snapshot.
            selected_by_its_document: popup_snapshot.document.as_ref().map(|document| {
              webview
                .for_document(document)
                .is_some_and(|selected| selected.snapshot().browser_id == popup_snapshot.browser_id)
            }),
            browser: BrowserInfo::new(&popup.browser()),
          }
        })
        .collect(),
    }
  }
}

/// The native window each label was last observed under, so a sample can say
/// whether it is still looking at the same native window.
#[derive(Default)]
pub struct ObservedWindows(Mutex<HashMap<String, NativeWindowToken>>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotOptions {
  /// The webview to sample, or the calling one when absent. Any label works,
  /// including a child webview of a multiwebview window, so a webview can be
  /// inspected from a window that is not it.
  label: Option<String>,
}

/// Samples the native CEF state of a webview.
///
/// The closure runs on CEF's UI thread, so the result comes back through a
/// channel this command waits on from a blocking task rather than from the
/// application's main thread, which is the thread that has to keep pumping CEF.
#[tauri::command]
pub async fn native_snapshot(
  app: AppHandle,
  webview: tauri::Webview,
  observed: State<'_, Arc<ObservedWindows>>,
  options: SnapshotOptions,
) -> Result<SnapshotInfo, String> {
  let (label, webview) = match options.label {
    // `get_webview` rather than `get_webview_window`, so a child webview of a
    // multiwebview window can be sampled too: its label is its own, not its
    // window's.
    Some(label) => {
      let webview = app
        .get_webview(&label)
        .ok_or_else(|| format!("no webview labeled {label}"))?;
      (label, webview)
    }
    None => (webview.label().to_string(), webview),
  };

  let observed = Arc::clone(observed.inner());
  let previous = observed
    .0
    .lock()
    .ok()
    .and_then(|windows| windows.get(&label).cloned());

  let (sender, receiver) = std::sync::mpsc::channel();
  webview
    .with_cef_webview(move |cef_webview| {
      if let Ok(mut windows) = observed.0.lock()
        && let Some(window) = cef_webview.snapshot().window.clone()
      {
        windows.insert(label, window);
      }
      let _ = sender.send(SnapshotInfo::new(cef_webview, previous));
    })
    .map_err(|error| error.to_string())?;

  tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(NATIVE_TIMEOUT))
    .await
    .map_err(|error| error.to_string())?
    .map_err(|_| "the CEF UI thread did not answer in time".to_string())
}

/// Sends one Chrome DevTools Protocol request to the calling webview's browser.
///
/// The identifier comes from the runtime's allocator rather than a counter of
/// our own: every observer on this browser sees every result, so a hardcoded
/// `id` can consume the result of a request somebody else sent.
#[tauri::command]
pub async fn send_devtools_message(
  webview: tauri::Webview,
  method: String,
  params: Option<serde_json::Value>,
) -> Result<i32, String> {
  let message_id = allocate_devtools_message_id().map_err(|error| error.to_string())?;
  let message = serde_json::json!({
    "id": message_id,
    "method": method,
    "params": params.unwrap_or_else(|| serde_json::json!({})),
  })
  .to_string();

  tauri::async_runtime::spawn_blocking(move || webview.send_dev_tools_message(message.as_bytes()))
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

  Ok(message_id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorWindowOptions {
  /// Names of the [`ChromeCommandGroup`] variants to keep live in the window.
  groups: Vec<String>,
  /// `"chrome"` or `"alloy"`.
  runtime_style: String,
  /// Whether Ctrl+Plus, Ctrl+Minus and Ctrl+0 zoom. This one is a portable Tauri
  /// attribute the CEF runtime honors, and it defaults to `false`.
  zoom_hotkeys: bool,
}

/// Maps the group names the frontend uses onto the runtime's enum.
fn chrome_command_groups(names: &[String]) -> Result<Vec<ChromeCommandGroup>, String> {
  names
    .iter()
    .map(|group| match group.as_str() {
      "windowAndTab" => Ok(ChromeCommandGroup::WindowAndTab),
      "document" => Ok(ChromeCommandGroup::Document),
      "browserChrome" => Ok(ChromeCommandGroup::BrowserChrome),
      "browserSurface" => Ok(ChromeCommandGroup::BrowserSurface),
      "history" => Ok(ChromeCommandGroup::History),
      other => Err(format!("unknown Chrome command group: {other}")),
    })
    .collect()
}

/// Opens a window that keeps the named families of Chrome accelerators.
///
/// A Chrome style browser keeps its whole accelerator table live even hosted as
/// a child view with no browser UI, so the runtime swallows the commands that
/// make no sense in an app window. This is the knob that gives a family back.
#[tauri::command]
pub fn open_accelerator_window(
  app: AppHandle,
  sink: State<'_, EventSink>,
  options: AcceleratorWindowOptions,
) -> Result<String, String> {
  static NEXT: AtomicU32 = AtomicU32::new(1);
  let label = format!("accelerators-{}", NEXT.fetch_add(1, Ordering::Relaxed));

  let groups = chrome_command_groups(&options.groups)?;

  let console_sink = sink.inner().clone();
  let console_label = label.clone();
  // Lets the page name the families it kept; nothing CEF-specific.
  let allowed = serde_json::to_string(&options.groups).map_err(|error| error.to_string())?;

  WebviewWindowBuilder::new(
    &app,
    label.clone(),
    WebviewUrl::App("accelerators.html".into()),
  )
  .title("Chrome accelerators")
  .inner_size(680., 620.)
  .initialization_script(format!("window.__ALLOWED_GROUPS__ = {allowed};"))
  .browser_runtime_style(if options.runtime_style == "alloy" {
    RuntimeStyle::Alloy
  } else {
    RuntimeStyle::Chrome
  })
  // Anything not named here is swallowed before it reaches Chromium's command
  // controller, and reported to the page as a disabled command.
  .allow_chrome_commands(groups)
  .zoom_hotkeys_enabled(options.zoom_hotkeys)
  .on_console_message(move |message| console_sink.console(&console_label, message))
  .build()
  .map_err(|error| error.to_string())?;

  Ok(label)
}

/// Opens one window hosting two sibling CEF browsers, configured through
/// [`WebviewBuilderCefExt`].
///
/// `WebviewWindowBuilderCefExt` covers the common case of one webview filling
/// one window. A window with several child webviews needs the same methods on
/// `tauri::webview::WebviewBuilder` instead, which is what this trait is —
/// behind Tauri's `unstable` feature, like the multiwebview API it extends.
///
/// The two children differ only in their runtime style, which is a per-browser
/// choice rather than a per-application one: Chrome style on the left, with a
/// family of accelerators kept, and Alloy style on the right, which has no
/// Chrome UI and so no accelerator table to keep anything from.
#[tauri::command]
pub fn open_child_webviews_window(
  app: AppHandle,
  sink: State<'_, EventSink>,
) -> Result<String, String> {
  static NEXT: AtomicU32 = AtomicU32::new(1);
  let label = format!("children-{}", NEXT.fetch_add(1, Ordering::Relaxed));

  let (width, height) = (900., 520.);
  let window = WindowBuilder::new(&app, &label)
    .title("Child webviews")
    .inner_size(width, height)
    .build()
    .map_err(|error| error.to_string())?;

  for (side, style, url) in [
    (
      "chrome",
      RuntimeStyle::Chrome,
      "child.html?style=Chrome&commands=History",
    ),
    ("alloy", RuntimeStyle::Alloy, "child.html?style=Alloy"),
  ] {
    let child_label = format!("{label}-{side}");
    let console_sink = sink.inner().clone();
    let frame_sink = sink.inner().clone();
    let console_label = child_label.clone();
    let frame_label = child_label.clone();

    let builder = WebviewBuilder::new(child_label, WebviewUrl::App(url.into()))
      .auto_resize()
      // Every method of `WebviewWindowBuilderCefExt` has a counterpart here.
      .browser_runtime_style(style)
      .allow_chrome_commands([ChromeCommandGroup::History])
      .on_console_message(move |message| console_sink.console(&console_label, message))
      .on_frame_event(move |event| frame_sink.frame(&frame_label, event));

    window
      .add_child(
        builder,
        LogicalPosition::new(if side == "chrome" { 0. } else { width / 2. }, 0.),
        LogicalSize::new(width / 2., height),
      )
      .map_err(|error| error.to_string())?;
  }

  Ok(label)
}
