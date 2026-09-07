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
//! # Safe Browsing
//!
//! [`PREFERENCES`] also disables `safebrowsing.enabled`. An app webview loads
//! the application's own bundled content, which the developer already trusts
//! and ships themselves, so sending its navigations to Google's Safe Browsing
//! service buys no protection and costs privacy and network traffic. This is
//! the wrong default for an app that points its webviews at arbitrary remote
//! web content the developer does not control - such an app should want Safe
//! Browsing back on.
//!
//! TODO: expose an opt-in knob on the `Cef` builder so those apps can
//! re-enable Safe Browsing (and, if useful, the rest of this list). That is a
//! deliberate follow-up rather than an oversight: it needs `runtime.rs`, which
//! this change does not own.

use cef::{ImplPreferenceManager, ImplValue, RequestContext};

/// Chromium profile preferences forced off for every webview, with the reason
/// each one is unwanted in an application webview:
///
/// * `credentials_enable_service` - Chrome offers to save credentials typed
///   into any form; an app's login form is not the browser's business.
/// * `autofill.profile_enabled` / `autofill.credit_card_enabled` - the same
///   deal for postal addresses and payment cards, which additionally sync into
///   the user's Google account.
/// * `translate.enabled` - the translate bubble both covers app UI and ships
///   page text off to Google's translation service to decide whether to offer.
/// * `alternate_error_pages.enabled` - on a failed navigation Chrome sends the
///   URL that failed to Google to fetch suggestions for it.
/// * `search.suggest_enabled` - streams typed input to the profile's default
///   search engine; an app has no omnibox for this to serve.
/// * `safebrowsing.enabled` - reputation lookups against Google for content the
///   app already bundles and trusts (see the module docs before changing this).
///
/// Preference names are the Chromium ones as of chromium-151.0.7922.174, the
/// build behind this crate's CEF pin.
const PREFERENCES: &[(&str, bool)] = &[
  ("credentials_enable_service", false),
  ("autofill.profile_enabled", false),
  ("autofill.credit_card_enabled", false),
  ("translate.enabled", false),
  ("alternate_error_pages.enabled", false),
  ("search.suggest_enabled", false),
  ("safebrowsing.enabled", false),
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
    if request_context.set_preference(Some(&(*name).into()), Some(&mut value), None) != 1 {
      log::debug!("failed to apply the {name} preference to the CEF request context");
    }
  }
}
