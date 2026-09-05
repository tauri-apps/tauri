// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::sys::cef_permission_request_types_t as PermissionType;
use cef::*;

const AUDIO_CAPTURE: u32 =
  cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32;
const VIDEO_CAPTURE: u32 =
  cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE as u32;

/// Media capture permissions granted to Alloy style browsers. Desktop capture is
/// deliberately excluded.
const ALLOY_MEDIA_PERMISSIONS: u32 = AUDIO_CAPTURE | VIDEO_CAPTURE;

/// The content setting that records each permission request type.
///
/// Kept in sync with Chromium's `permissions::RequestTypeToContentSettingsType`, which
/// is what `cef_permission_request_types_t` mirrors. Request types with no content
/// setting of their own are absent and are simply not recorded.
const PERMISSION_CONTENT_SETTINGS: &[(u32, ContentSettingTypes)] = &[
  (
    PermissionType::CEF_PERMISSION_TYPE_AR_SESSION as u32,
    ContentSettingTypes::AR,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CAMERA_PAN_TILT_ZOOM as u32,
    ContentSettingTypes::CAMERA_PAN_TILT_ZOOM,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CAMERA_STREAM as u32,
    ContentSettingTypes::MEDIASTREAM_CAMERA,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CAPTURED_SURFACE_CONTROL as u32,
    ContentSettingTypes::CAPTURED_SURFACE_CONTROL,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_CLIPBOARD as u32,
    ContentSettingTypes::CLIPBOARD_READ_WRITE,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_TOP_LEVEL_STORAGE_ACCESS as u32,
    ContentSettingTypes::TOP_LEVEL_STORAGE_ACCESS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_DISK_QUOTA as u32,
    ContentSettingTypes::PERSISTENT_STORAGE,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_LOCAL_FONTS as u32,
    ContentSettingTypes::LOCAL_FONTS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_GEOLOCATION as u32,
    ContentSettingTypes::GEOLOCATION,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_HAND_TRACKING as u32,
    ContentSettingTypes::HAND_TRACKING,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_IDENTITY_PROVIDER as u32,
    ContentSettingTypes::FEDERATED_IDENTITY_API,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_IDLE_DETECTION as u32,
    ContentSettingTypes::IDLE_DETECTION,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_MIC_STREAM as u32,
    ContentSettingTypes::MEDIASTREAM_MIC,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_MIDI_SYSEX as u32,
    ContentSettingTypes::MIDI_SYSEX,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_MULTIPLE_DOWNLOADS as u32,
    ContentSettingTypes::AUTOMATIC_DOWNLOADS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_NOTIFICATIONS as u32,
    ContentSettingTypes::NOTIFICATIONS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_KEYBOARD_LOCK as u32,
    ContentSettingTypes::KEYBOARD_LOCK,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_POINTER_LOCK as u32,
    ContentSettingTypes::POINTER_LOCK,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_PROTECTED_MEDIA_IDENTIFIER as u32,
    ContentSettingTypes::PROTECTED_MEDIA_IDENTIFIER,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_REGISTER_PROTOCOL_HANDLER as u32,
    ContentSettingTypes::PROTOCOL_HANDLERS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_STORAGE_ACCESS as u32,
    ContentSettingTypes::STORAGE_ACCESS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_VR_SESSION as u32,
    ContentSettingTypes::VR,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_WEB_APP_INSTALLATION as u32,
    ContentSettingTypes::WEB_APP_INSTALLATION,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_WINDOW_MANAGEMENT as u32,
    ContentSettingTypes::WINDOW_MANAGEMENT,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_FILE_SYSTEM_ACCESS as u32,
    ContentSettingTypes::FILE_SYSTEM_WRITE_GUARD,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_LOCAL_NETWORK_ACCESS_DEPRECATED as u32,
    ContentSettingTypes::LOCAL_NETWORK_ACCESS,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_LOCAL_NETWORK as u32,
    ContentSettingTypes::LOCAL_NETWORK,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_LOOPBACK_NETWORK as u32,
    ContentSettingTypes::LOOPBACK_NETWORK,
  ),
  (
    PermissionType::CEF_PERMISSION_TYPE_SENSORS as u32,
    ContentSettingTypes::SENSORS,
  ),
];

wrap_permission_handler! {
  pub struct TauriCefPermissionHandler {}

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      // Chrome style displays the permission request UI and records the outcome as a
      // content setting. That content setting is what keeps `enumerateDevices()` from
      // returning a redacted device list, so let CEF handle the request.
      let Some(host) = alloy_style_host(browser) else {
        return 0;
      };

      // Alloy style has no permission UI and its default handling denies the request,
      // so grant camera and microphone capture here.
      let Some(callback) = callback else {
        return 0;
      };

      let allowed = requested_permissions & ALLOY_MEDIA_PERMISSIONS;
      if allowed == 0 {
        return 0;
      }

      let mut settings = Vec::with_capacity(2);
      if allowed & AUDIO_CAPTURE != 0 {
        settings.push(ContentSettingTypes::MEDIASTREAM_MIC);
      }
      if allowed & VIDEO_CAPTURE != 0 {
        settings.push(ContentSettingTypes::MEDIASTREAM_CAMERA);
      }
      allow_content_settings(&host, requesting_origin, &settings);

      callback.cont(allowed);
      1
    }

    fn on_show_permission_prompt(
      &self,
      browser: Option<&mut Browser>,
      _prompt_id: u64,
      requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      // Chrome style displays the permission prompt UI.
      let Some(host) = alloy_style_host(browser) else {
        return 0;
      };

      // Alloy style has no prompt UI, and its default handling is
      // `CEF_PERMISSION_RESULT_IGNORE`, which can leave the page's promise unresolved.
      // Accept instead, matching the behavior Alloy browsers had before permission
      // prompts were deferred to CEF.
      let Some(callback) = callback else {
        return 0;
      };

      let settings = PERMISSION_CONTENT_SETTINGS
        .iter()
        .filter(|(permission, _)| requested_permissions & permission != 0)
        .map(|(_, setting)| *setting)
        .collect::<Vec<_>>();
      allow_content_settings(&host, requesting_origin, &settings);

      callback.cont(PermissionRequestResult::ACCEPT);
      1
    }
  }
}

/// The host of `browser` when it uses the Alloy runtime style, which provides no
/// permission UI.
///
/// Returns `None` when the style cannot be determined, so that permission handling is
/// deferred to CEF, which is correct for the Chrome style Tauri webviews use by default.
fn alloy_style_host(browser: Option<&mut Browser>) -> Option<BrowserHost> {
  browser
    .and_then(|browser| browser.host())
    .filter(|host| host.runtime_style() == RuntimeStyle::ALLOY)
}

/// Records permissions granted to an Alloy style browser as content settings for
/// `requesting_origin`.
///
/// Granting through a CEF callback alone is invisible to Chromium's permission layer, so
/// `navigator.permissions.query()` keeps reporting `prompt` even though the feature
/// works. Writing the content setting is what Chrome style does when the user accepts
/// its prompt.
///
/// Does nothing when the origin is unknown, because `set_content_setting` with no URL
/// changes the default for every origin rather than for this one.
fn allow_content_settings(
  host: &BrowserHost,
  requesting_origin: Option<&CefString>,
  settings: &[ContentSettingTypes],
) {
  if requesting_origin.is_none() || settings.is_empty() {
    return;
  }

  let Some(context) = host.request_context() else {
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
