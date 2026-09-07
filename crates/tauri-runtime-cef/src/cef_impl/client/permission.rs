// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The application's permission policy, as CEF asks for it.
//!
//! CEF has two entry points and they behave very differently once answered, which
//! is the single most surprising thing about this file.
//!
//! # The prompt path answers once per origin, forever
//!
//! `on_show_permission_prompt` is reached only when Chromium's stored content
//! setting for that (origin, permission) is still "ask". `cont` then persists the
//! answer — `ACCEPT` through `PermissionRequestManager::Accept()`, exactly as a
//! click on Chrome's Allow button would — into the on-disk profile. Chromium
//! reads that setting on the next request and never asks again.
//!
//! So the application's handler is consulted **once per origin and permission**,
//! and the answer outlives the process: a handler whose answer depends on app
//! state (a user having signed in, a setting having been toggled) is silently
//! ignored from its second request onwards, including across restarts.
//!
//! Both answers persist, not just the grant. `DENY` reaches
//! `PermissionRequestManager::Deny()`, which stores BLOCK, so a handler that
//! refuses once has refused for good: Chromium auto-denies the next request
//! without reaching this file, and the application cannot later change its mind by
//! answering `Allow`. `CEF_PERMISSION_RESULT_DISMISS` would leave the setting at
//! "ask", but it is not what a deliberate refusal means, and a policy that has to
//! be re-asked on every request is what the media path below already provides.
//!
//! There is no callback here for "the app changed its mind". An app that has to
//! revoke a grant, or undo a refusal, rewrites the content setting itself:
//! `Webview::browser()` reaches the `cef::Browser`, and from it
//! `host().request_context().set_content_setting(...)`.
//!
//! # The media path answers every call
//!
//! `on_request_media_access_permission` is different: Chromium routes every
//! `getUserMedia()` call through it, so camera and microphone requests do reach
//! the handler each time and a changing answer is honored. `MediaAccessCallback`
//! persists nothing, which is why this path — and only this path — writes the
//! content settings itself (see [`allow_content_settings`]).
//!
//! # Unmapped request types are [`PermissionKind::Other`]
//!
//! [`PERMISSION_KINDS`] is a partial map: Chromium has more request types than
//! Tauri has kinds. Storage Access and Top Level Storage Access, FedCM (identity
//! provider), protocol handler registration, idle detection, local and loopback
//! network access, web app installation, the AR and VR sessions behind WebXR,
//! hand tracking, keyboard lock and disk quota are the whole of what is left
//! over, and every one of them arrives as `PermissionKind::Other` — as does any
//! request type a future CEF build adds.
//!
//! Failing closed is deliberate — a new request type must never be granted behind
//! the application's back — but it means a handler written for another platform as
//! `match kind { Camera => Allow, _ => Deny }` hard-denies all of the above on
//! CEF, and denying Storage Access or FedCM breaks third-party SSO flows outright.
//! A handler that only means to answer about the kinds it names should return
//! [`PermissionResponse::Default`] for the rest.

use std::sync::Arc;

use cef::sys::cef_media_access_permission_types_t as MediaPermissionType;
use cef::sys::cef_permission_request_types_t as PermissionType;
use cef::*;
use tauri_runtime::webview::{PermissionKind, PermissionResponse};

/// The application's answer to a permission request, as
/// `PendingWebview::permission_request_handler` carries it.
///
/// `tauri_runtime::webview` declares the same alias but keeps it private, so it is
/// redeclared here; both name one and the same type.
pub(crate) type PermissionRequestHandler =
  dyn Fn(PermissionKind) -> PermissionResponse + Send + Sync;

const AUDIO_CAPTURE: u32 = MediaPermissionType::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32;
const VIDEO_CAPTURE: u32 = MediaPermissionType::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE as u32;
const DESKTOP_AUDIO_CAPTURE: u32 =
  MediaPermissionType::CEF_MEDIA_PERMISSION_DESKTOP_AUDIO_CAPTURE as u32;
const DESKTOP_VIDEO_CAPTURE: u32 =
  MediaPermissionType::CEF_MEDIA_PERMISSION_DESKTOP_VIDEO_CAPTURE as u32;

/// The bits a `getDisplayMedia()` request sets. Never granted from here; see
/// [`grants_desktop_capture`].
const DESKTOP_CAPTURE: u32 = DESKTOP_AUDIO_CAPTURE | DESKTOP_VIDEO_CAPTURE;

/// Media capture permissions granted to Alloy style browsers. Desktop capture is
/// deliberately excluded.
const ALLOY_MEDIA_PERMISSIONS: u32 = AUDIO_CAPTURE | VIDEO_CAPTURE;

/// The [`PermissionKind`] the application is asked about for each permission request
/// type.
///
/// Chromium's request types are finer grained than Tauri's kinds — the plain camera
/// stream and its pan-tilt-zoom control are both `Camera` — and most of them have no
/// Tauri counterpart at all. A request type absent from this table is reported as
/// [`PermissionKind::Other`], which is also what a request type added by a future
/// CEF build maps to, so a new type is never silently granted behind the
/// application's back.
///
/// [`PermissionKind::Autoplay`] has no entry: Chromium gates autoplay through its
/// media engagement policy rather than through a permission request, so no CEF
/// request carries it.
const PERMISSION_KINDS: &[(u32, PermissionKind)] = &[
  (
    PermissionType::CEF_PERMISSION_TYPE_CAMERA_PAN_TILT_ZOOM as u32,
    PermissionKind::Camera,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CAMERA_STREAM as u32,
    PermissionKind::Camera,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_MIC_STREAM as u32,
    PermissionKind::Microphone,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CAPTURED_SURFACE_CONTROL as u32,
    PermissionKind::DisplayCapture,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_GEOLOCATION as u32,
    PermissionKind::Geolocation,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_NOTIFICATIONS as u32,
    PermissionKind::Notifications,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CLIPBOARD as u32,
    PermissionKind::ClipboardRead,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_MIDI_SYSEX as u32,
    PermissionKind::Midi,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_SENSORS as u32,
    PermissionKind::Sensors,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_LOCAL_FONTS as u32,
    PermissionKind::LocalFonts,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_WINDOW_MANAGEMENT as u32,
    PermissionKind::WindowManagement,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_POINTER_LOCK as u32,
    PermissionKind::PointerLock,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_MULTIPLE_DOWNLOADS as u32,
    PermissionKind::AutomaticDownloads,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_FILE_SYSTEM_ACCESS as u32,
    PermissionKind::FileSystemAccess,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_PROTECTED_MEDIA_IDENTIFIER as u32,
    PermissionKind::MediaKeySystemAccess,
  ),
];

/// The [`PermissionKind`] the application is asked about for each media access type.
///
/// Both desktop capture types are one kind: `getDisplayMedia` is a single Tauri
/// permission whether the page asks for the screen's video, its audio, or both.
const MEDIA_PERMISSION_KINDS: &[(u32, PermissionKind)] = &[
  (AUDIO_CAPTURE, PermissionKind::Microphone),
  (VIDEO_CAPTURE, PermissionKind::Camera),
  (DESKTOP_AUDIO_CAPTURE, PermissionKind::DisplayCapture),
  (DESKTOP_VIDEO_CAPTURE, PermissionKind::DisplayCapture),
];

/// How the application answered a whole CEF permission request.
///
/// CEF asks about a bitmask of types and is answered once for all of them, so the
/// per-kind answers have to be combined. [`Self::Denied`] wins over everything: no
/// reply denies one type while leaving the others to the platform, and a refusal
/// must never end up granting the rest of the request. A request is granted only
/// when the application allowed every type in it, so what is granted is exactly the
/// subset it allowed. Anything else leaves part of the request unanswered, and the
/// platform default runs for it unchanged.
enum AppDecision {
  /// The application denied at least one of the requested permissions.
  Denied,
  /// The application allowed every requested permission.
  Allowed,
  /// The application left at least one requested permission to the platform, and
  /// denied none. Also the answer when the webview has no permission handler.
  NoOpinion,
}

wrap_permission_handler! {
  pub struct TauriCefPermissionHandler {
    permission_request_handler: Option<Arc<PermissionRequestHandler>>,
  }

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      let host = browser_host(browser);

      match self.decide(requested_permissions, media_permission_kind) {
        // Answer for the application, whatever the runtime style: an explicit
        // answer is the whole point of the handler, so neither Chrome's prompt nor
        // Alloy's blanket grant may override it.
        AppDecision::Denied => {
          let Some(callback) = callback else {
            return 0;
          };
          callback.cont(0);
          1
        }
        AppDecision::Allowed => {
          // An `Allow` grants the camera and the microphone, but never the desktop:
          // no `PermissionKind` answer can name what a screen share would expose,
          // so that choice stays with CEF's picker. See `grants_desktop_capture`.
          if grants_desktop_capture(requested_permissions) {
            return 0;
          }

          let Some(callback) = callback else {
            return 0;
          };
          allow_content_settings(
            host.as_ref(),
            requesting_origin,
            &media_content_settings(requested_permissions),
          );
          callback.cont(requested_permissions);
          1
        }
        AppDecision::NoOpinion => {
          // Chrome style displays the permission request UI and records the outcome as a
          // content setting. That content setting is what keeps `enumerateDevices()` from
          // returning a redacted device list, so let CEF handle the request.
          if !is_alloy_style(host.as_ref()) {
            return 0;
          }

          // Alloy style has no permission UI and its default handling denies the request,
          // so grant camera and microphone capture here. Desktop capture defers to that
          // default handling, which refuses it — see `grants_desktop_capture`. Deferring
          // the whole request also keeps the granted mask equal to the requested one,
          // which CEF requires of a `getUserMedia` answer.
          if grants_desktop_capture(requested_permissions) {
            return 0;
          }

          let Some(callback) = callback else {
            return 0;
          };

          let allowed = requested_permissions & ALLOY_MEDIA_PERMISSIONS;
          if allowed == 0 {
            return 0;
          }

          allow_content_settings(
            host.as_ref(),
            requesting_origin,
            &media_content_settings(allowed),
          );

          callback.cont(allowed);
          1
        }
      }
    }

    fn on_show_permission_prompt(
      &self,
      browser: Option<&mut Browser>,
      _prompt_id: u64,
      _requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      let host = browser_host(browser);

      match self.decide(requested_permissions, permission_kind) {
        // Answer for the application, whatever the runtime style, and without
        // showing Chrome's prompt: the application already decided.
        AppDecision::Denied => {
          let Some(callback) = callback else {
            return 0;
          };
          callback.cont(PermissionRequestResult::DENY);
          1
        }
        AppDecision::Allowed => {
          let Some(callback) = callback else {
            return 0;
          };
          // No content setting is written here. `cont(ACCEPT)` reaches
          // `PermissionRequestManager::Accept()`, the very path a user's click on
          // Chrome's Allow button takes, and that path persists the grant itself —
          // so `navigator.permissions.query()` already agrees with it. Writing one
          // on top would be duplicative at best and wrong at worst: the write names
          // no top-level URL, which is a wildcard secondary pattern, and Chromium
          // scopes a storage-access grant to the (embedded origin, top-level site)
          // pair. A handler answering `Allow` to a `PermissionKind::Other` request
          // would have granted that origin storage access on every top-level site.
          callback.cont(PermissionRequestResult::ACCEPT);
          1
        }
        AppDecision::NoOpinion => {
          // Chrome style displays the permission prompt UI.
          if !is_alloy_style(host.as_ref()) {
            return 0;
          }

          // Alloy style has no prompt UI, and its default handling is
          // `CEF_PERMISSION_RESULT_IGNORE`, which can leave the page's promise unresolved.
          // Accept instead, matching the behavior Alloy browsers had before permission
          // prompts were deferred to CEF.
          let Some(callback) = callback else {
            return 0;
          };

          // As above: accepting is what persists the grant, and the extra write
          // would be both redundant and over-broad.
          callback.cont(PermissionRequestResult::ACCEPT);
          1
        }
      }
    }
  }
}

impl TauriCefPermissionHandler {
  /// Asks the application about every type set in `requested` and combines the
  /// answers, as [`AppDecision`] describes.
  ///
  /// `kind` names the [`PermissionKind`] of one request type; the two CEF request
  /// bitmasks number their types differently, so each entry point passes its own.
  fn decide(&self, requested: u32, kind: fn(u32) -> PermissionKind) -> AppDecision {
    let Some(handler) = &self.permission_request_handler else {
      return AppDecision::NoOpinion;
    };

    // A refusal is seen wherever it sits in the bitmask, and short-circuits there:
    // nothing after it could weaken it, so the types past it are not asked about. A
    // handler that logs or keeps state therefore sees only a prefix of a denied
    // request — up to and including the type it refused — never the whole of it.
    // Left as it is on purpose: the handler is a policy predicate, not an event
    // feed, so asking it about types whose answer cannot change the outcome would
    // be work with no result.
    let mut asked = false;
    let mut allowed_all = true;
    for permission in requested_permissions(requested) {
      asked = true;
      match handler(kind(permission)) {
        PermissionResponse::Deny => return AppDecision::Denied,
        PermissionResponse::Allow => {}
        PermissionResponse::Default => allowed_all = false,
      }
    }

    if asked && allowed_all {
      AppDecision::Allowed
    } else {
      AppDecision::NoOpinion
    }
  }
}

/// Whether answering `requested` through [`MediaAccessCallback::cont`] would hand
/// the page a desktop stream, which is never this handler's to give.
///
/// CEF builds the granted stream straight from the mask: a set
/// `DESKTOP_VIDEO_CAPTURE` bit with no requested device id synthesises a
/// `DesktopMediaID(TYPE_SCREEN, kFullDesktopScreenId)` and returns it, so
/// `getDisplayMedia()` resolves with the whole desktop and *no picker at all*.
/// Chrome style would have shown Chromium's desktop media picker, and Alloy style
/// refused desktop capture outright; neither can be reconstructed from a
/// [`PermissionKind`], which names no screen, window or tab.
///
/// So an app writing `.on_permission_request(|_| PermissionResponse::Allow)` — a
/// plausible "it is all my own content" rule — would silently give any page in the
/// webview, including remote content reached through a redirect, a full-desktop
/// stream. Deferring the request to CEF instead keeps the picker in front of the
/// user, which is the only thing that can name what is actually shared.
///
/// The whole request is deferred, never a part of it: CEF documents that when a
/// request carries the device capture bits — that is, when it came from
/// `getUserMedia()` — `allowed_permissions` must equal `required_permissions`, so
/// granting the device half of a mixed mask while withholding the desktop half is
/// not a legal answer. Deferring costs nothing, because the device bits CEF would
/// then handle itself are exactly the ones its own prompt covers.
///
/// A `Deny` is unaffected: `cont(0)` refuses every bit in the request, desktop
/// capture included, and refusing is always a legal answer.
fn grants_desktop_capture(requested: u32) -> bool {
  requested & DESKTOP_CAPTURE != 0
}

/// The individual permission types set in a CEF request bitmask.
fn requested_permissions(requested: u32) -> impl Iterator<Item = u32> {
  (0..u32::BITS)
    .map(move |bit| requested & (1 << bit))
    .filter(|permission| *permission != 0)
}

/// The [`PermissionKind`] of one `cef_permission_request_types_t` value.
fn permission_kind(permission: u32) -> PermissionKind {
  lookup_permission_kind(PERMISSION_KINDS, permission)
}

/// The [`PermissionKind`] of one `cef_media_access_permission_types_t` value.
fn media_permission_kind(permission: u32) -> PermissionKind {
  lookup_permission_kind(MEDIA_PERMISSION_KINDS, permission)
}

fn lookup_permission_kind(table: &[(u32, PermissionKind)], permission: u32) -> PermissionKind {
  table
    .iter()
    .find(|(candidate, _)| *candidate == permission)
    .map(|(_, kind)| *kind)
    .unwrap_or(PermissionKind::Other)
}

/// The content settings recording granted media capture.
///
/// Desktop capture has none: `getDisplayMedia` is gated by Chromium's source
/// picker rather than by a content setting, so there is nothing to record for it.
fn media_content_settings(granted: u32) -> Vec<ContentSettingTypes> {
  let mut settings = Vec::with_capacity(2);
  if granted & AUDIO_CAPTURE != 0 {
    settings.push(ContentSettingTypes::MEDIASTREAM_MIC);
  }
  if granted & VIDEO_CAPTURE != 0 {
    settings.push(ContentSettingTypes::MEDIASTREAM_CAMERA);
  }
  settings
}

/// The host of `browser`, when CEF hands one out.
fn browser_host(browser: Option<&mut Browser>) -> Option<BrowserHost> {
  browser.and_then(|browser| browser.host())
}

/// Whether `host` uses the Alloy runtime style, which provides no permission UI.
///
/// A browser whose host CEF did not hand out reports `false`, so that permission
/// handling is deferred to CEF, which is correct for the Chrome style Tauri
/// webviews use by default.
fn is_alloy_style(host: Option<&BrowserHost>) -> bool {
  host.is_some_and(|host| host.runtime_style() == RuntimeStyle::ALLOY)
}

/// Records granted media capture as content settings for `requesting_origin`.
///
/// `MediaAccessCallback::cont` grants the stream and nothing else: the grant is
/// invisible to Chromium's permission layer, so `navigator.permissions.query()`
/// keeps reporting `prompt` and `enumerateDevices()` keeps returning a redacted
/// list even though `getUserMedia` works. Writing the content setting is what
/// Chrome style does when the user accepts its prompt.
///
/// The permission *prompt* path needs none of this and must not use it.
/// `PermissionPromptCallback::cont(ACCEPT)` reaches
/// `PermissionRequestManager::Accept()`, which persists the grant itself, and the
/// secondary pattern below is a wildcard — harmless for the two media settings,
/// which Chromium scopes to the requesting origin alone, but wrong for anything
/// Chromium scopes to an (origin, top-level site) pair.
///
/// # The setting outlives the answer that wrote it
///
/// The write is persistent while the application's answer is not: the media path
/// is consulted on every `getUserMedia()` call, so a handler may allow once and
/// deny afterwards. Denying still refuses the stream — that check runs before this
/// — but the content setting stays ALLOW, so `navigator.permissions.query()` keeps
/// reporting `granted` and `enumerateDevices()` keeps returning unredacted device
/// labels for that origin. An application that revokes camera or microphone access
/// for good should rewrite the setting itself; see the module docs.
///
/// Does nothing when the origin is unknown, because `set_content_setting` with no URL
/// changes the default for every origin rather than for this one, and nothing when CEF
/// handed out no host, because the request context is reached through it.
fn allow_content_settings(
  host: Option<&BrowserHost>,
  requesting_origin: Option<&CefString>,
  settings: &[ContentSettingTypes],
) {
  if requesting_origin.is_none() || settings.is_empty() {
    return;
  }

  let Some(context) = host.and_then(|host| host.request_context()) else {
    return;
  };

  for setting in settings {
    context.set_content_setting(
      requesting_origin,
      None,
      *setting,
      ContentSettingValues::ALLOW,
    );
  }
}
