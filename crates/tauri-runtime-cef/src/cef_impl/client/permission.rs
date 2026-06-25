// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::*;

wrap_permission_handler! {
  pub struct TauriCefPermissionHandler {}

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      // Allow microphone and camera when requested.
      let allowed = requested_permissions
        & (cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE
          as u32
          | cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE
            as u32);
      if allowed != 0 {
        callback.cont(requested_permissions);
        return 1;
      }
      0
    }

    fn on_show_permission_prompt(
      &self,
      _browser: Option<&mut Browser>,
      _prompt_id: u64,
      _requesting_origin: Option<&CefString>,
      _requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      // Allow permission prompt (e.g. microphone/camera).
      callback.cont(PermissionRequestResult::from(
        cef::sys::cef_permission_request_result_t::CEF_PERMISSION_RESULT_ACCEPT,
      ));
      1
    }
  }
}
