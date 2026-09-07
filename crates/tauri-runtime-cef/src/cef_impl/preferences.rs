// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Chromium profile preferences applied to every webview's request context.
//!
//! The CEF runtime creates Chrome style browsers, so each webview is backed by
//! a real Chrome profile and inherits the browser-shaped behaviour that comes
//! with it: a "Save password?" bubble on any form submit, address and
//! credit-card save bubbles, a "Translate this page?" bubble, and a handful of
//! background requests to Google. None of that belongs in an application
//! webview, so we turn it off per request context right after the profile
//! finishes initializing.
//!
//! # Safe Browsing stays on
//!
//! [`PREFERENCES`] deliberately leaves `safebrowsing.enabled` alone. A Tauri
//! webview routinely loads content the developer does not control - OAuth and
//! SSO flows, embedded third-party pages, iframes, and the popups this runtime
//! supports - so it is not the closed world that would make the protection
//! pointless. Standard protection is also a local hash-prefix database rather
//! than a per-navigation callback to Google, so the privacy cost is far smaller
//! than turning it off would suggest. WebView2 inherits Edge SmartScreen and
//! Tauri does not disable it, so shipping this off would leave CEF the least
//! protected of Tauri's webview backends.
//!
//! TODO: expose a knob on the `Cef` builder for an application that really does
//! want to opt *out* of Safe Browsing. Shipping the protection on by default is
//! the part that cannot wait for it.

use cef::{CefString, ImplPreferenceManager, ImplValue, RequestContext};

/// Builds the `error` out-parameter that every
/// [`ImplPreferenceManager::set_preference`] call has to pass.
///
/// `CefPreferenceManager::SetPreference` marks only `value` as an optional
/// parameter, so CEF's generated C-to-C++ shim opens with
/// `DCHECK(error); if (!error) { return 0; }`. The Rust binding turns a [`None`]
/// error into a null pointer, so passing [`None`] makes the call report failure
/// before the preference service is ever consulted - the preference is never
/// written, whatever the caller asked for.
///
/// [`CefString::default`] is not a substitute: it builds the borrowed-none
/// variant, whose `&mut CefString` to `*mut cef_string_utf16_t` conversion is a
/// null pointer again. `CefString::from("")` builds the owned variant, which
/// converts to a real, writable pointer and frees whatever CEF stores in it when
/// the string is dropped.
pub(crate) fn set_preference_error_slot() -> CefString {
  CefString::from("")
}

/// Chromium profile preferences forced off for every webview, with the reason
/// each one is unwanted in an application webview:
///
/// * `credentials_enable_service` - Chrome offers to save credentials typed
///   into any form; an app's login form is not the browser's business.
/// * `profile.password_manager_leak_detection` - on by default in Chromium, it
///   sends a hashed prefix of credentials typed into any form to Google to check
///   them against known breaches. That is the password manager reaching into the
///   app's own login form, which is exactly what this list exists to prevent.
/// * `autofill.profile_enabled` / `autofill.credit_card_enabled` - the same
///   deal for postal addresses and payment cards, which additionally sync into
///   the user's Google account.
/// * `translate.enabled` - the translate bubble both covers app UI and ships
///   page text off to Google's translation service to decide whether to offer.
/// * `alternate_error_pages.enabled` - on a failed navigation Chrome sends the
///   URL that failed to Google to fetch suggestions for it.
/// * `search.suggest_enabled` - streams typed input to the profile's default
///   search engine; an app has no omnibox for this to serve.
///
/// `safebrowsing.enabled` is deliberately absent - see the module docs.
///
/// Preference names are the Chromium ones as of chromium-151.0.7922.174, the
/// build behind this crate's CEF pin.
const PREFERENCES: &[(&str, bool)] = &[
  ("credentials_enable_service", false),
  ("profile.password_manager_leak_detection", false),
  ("autofill.profile_enabled", false),
  ("autofill.credit_card_enabled", false),
  ("translate.enabled", false),
  ("alternate_error_pages.enabled", false),
  ("search.suggest_enabled", false),
];

/// Applies [`PREFERENCES`] to `request_context`.
///
/// Must be called after the request context has finished initializing - a
/// preference cannot be written before the underlying Chromium `Profile`
/// exists.
///
/// Which preferences a given Chrome build registers as writable varies, so a
/// refused preference is logged at debug and skipped rather than warned about:
/// there is one request context per webview and a missing preference is not
/// something the app developer can act on.
pub(crate) fn apply_app_webview_preferences(request_context: &RequestContext) {
  for (name, enabled) in PREFERENCES {
    if request_context.can_set_preference(Some(&(*name).into())) != 1 {
      log::debug!("the CEF request context does not allow setting the {name} preference");
      continue;
    }

    let Some(value) = cef::value_create() else {
      continue;
    };
    value.set_bool(i32::from(*enabled));

    let mut value = value;
    let mut error = set_preference_error_slot();
    if request_context.set_preference(Some(&(*name).into()), Some(&mut value), Some(&mut error))
      != 1
    {
      log::debug!("failed to apply the {name} preference to the CEF request context: {error}");
    }
  }
}
