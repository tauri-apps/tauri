// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::*;

/// Media capture permissions granted to Alloy style browsers.
const ALLOY_MEDIA_PERMISSIONS: u32 =
  cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32
    | cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE
      as u32;

wrap_permission_handler! {
  pub struct TauriCefPermissionHandler {}

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      // Chrome style displays the permission request UI and records the outcome as a
      // content setting. That content setting is what keeps `enumerateDevices()` from
      // returning a redacted device list, so let CEF handle the request.
      if !is_alloy_style(browser) {
        return 0;
      }

      // Alloy style has no permission UI and its default handling denies the request,
      // so grant camera and microphone capture here. Desktop capture is not granted.
      let Some(callback) = callback else {
        return 0;
      };

      let allowed = requested_permissions & ALLOY_MEDIA_PERMISSIONS;
      if allowed == 0 {
        return 0;
      }

      callback.cont(allowed);
      1
    }

    fn on_show_permission_prompt(
      &self,
      browser: Option<&mut Browser>,
      _prompt_id: u64,
      _requesting_origin: Option<&CefString>,
      _requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      // Chrome style displays the permission prompt UI.
      if !is_alloy_style(browser) {
        return 0;
      }

      // Alloy style has no prompt UI, and its default handling is
      // `CEF_PERMISSION_RESULT_IGNORE`, which can leave the page's promise unresolved.
      // Accept instead.
      let Some(callback) = callback else {
        return 0;
      };

      callback.cont(PermissionRequestResult::ACCEPT);
      1
    }
  }
}

/// Whether the browser uses the Alloy runtime style, which provides no permission UI.
///
/// Falls back to `false` when the style cannot be determined, so that permission
/// handling is deferred to CEF, which is correct for the Chrome style Tauri webviews
/// use by default.
fn is_alloy_style(browser: Option<&mut Browser>) -> bool {
  browser
    .and_then(|browser| browser.host())
    .is_some_and(|host| host.runtime_style() == RuntimeStyle::ALLOY)
}
