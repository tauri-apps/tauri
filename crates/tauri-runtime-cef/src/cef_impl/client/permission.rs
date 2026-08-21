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
      _requested_permissions: u32,
      _callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      // Use CEF's default media permission handling.
      0
    }

    fn on_show_permission_prompt(
      &self,
      _browser: Option<&mut Browser>,
      _prompt_id: u64,
      _requesting_origin: Option<&CefString>,
      _requested_permissions: u32,
      _callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      // Use CEF's default permission prompt handling.
      0
    }
  }
}
