// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Pure helpers for the browser permission handler.
//!
//! The CEF callbacks in `cef_impl.rs` wrap these helpers so the bit-mask
//! decisions stay unit-testable without a live CEF runtime.

use cef::sys::{cef_media_access_permission_types_t, cef_permission_request_types_t};

/// Mask of media-access bits the app is willing to forward to Chromium.
/// Covers both user-media (`getUserMedia`: mic + camera) and display-media
/// (`getDisplayMedia`: desktop audio + desktop video / screen share).
pub const ALLOWED_MEDIA_MASK: u32 =
  cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32
    | cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE as u32
    | cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DESKTOP_AUDIO_CAPTURE as u32
    | cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DESKTOP_VIDEO_CAPTURE as u32;

/// Mask of prompt-permission bits auto-accepted without user interaction.
/// Covers the set an embedded SaaS app (Slack, Meet, Discord, Notion, etc.)
/// realistically needs: mic/camera streams + their PTZ/captured-surface
/// siblings, clipboard, notifications, storage access, and the Chromium
/// Private Network Access family (loopback / local-network) that WebRTC
/// call flows like Slack Huddles use for STUN/TURN candidate gathering on
/// the host machine.
///
/// Still deliberately excluded: geolocation, midi-sysex,
/// protected-media-identifier, idle-detection, file-system-access,
/// window-management, AR/VR, hand-tracking — privacy- or fingerprint-
/// sensitive surfaces that should never be granted silently.
pub const ALLOWED_PROMPT_MASK: u32 =
  cef_permission_request_types_t::CEF_PERMISSION_TYPE_CAMERA_STREAM as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_CAMERA_PAN_TILT_ZOOM as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_MIC_STREAM as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_CAPTURED_SURFACE_CONTROL as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_CLIPBOARD as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_NOTIFICATIONS as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_STORAGE_ACCESS as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_TOP_LEVEL_STORAGE_ACCESS as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_LOOPBACK_NETWORK as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_LOCAL_NETWORK as u32
    | cef_permission_request_types_t::CEF_PERMISSION_TYPE_LOCAL_NETWORK_ACCESS as u32;

/// Returns the subset of `requested` permissions the embedder will forward to
/// Chromium for a `getUserMedia` / `getDisplayMedia` request. Zero means
/// "nothing requested is allowed — deny the whole callback".
pub fn allowed_media_permissions(requested: u32) -> u32 {
  requested & ALLOWED_MEDIA_MASK
}

/// Returns `true` iff every bit in `requested` is covered by
/// [`ALLOWED_PROMPT_MASK`]. A single out-of-list bit denies the whole prompt
/// because the CEF prompt callback is all-or-nothing.
pub fn should_accept_permission_prompt(requested: u32) -> bool {
  requested != 0 && (requested & !ALLOWED_PROMPT_MASK) == 0
}

/// Human-readable comma-joined names of the media bits set in `bits`.
/// Used only for grep-friendly debug logs.
pub fn format_media_bits(bits: u32) -> String {
  let mut out = Vec::new();
  if bits & cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32
    != 0
  {
    out.push("device-audio");
  }
  if bits & cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE as u32
    != 0
  {
    out.push("device-video");
  }
  if bits & cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DESKTOP_AUDIO_CAPTURE as u32
    != 0
  {
    out.push("desktop-audio");
  }
  if bits & cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DESKTOP_VIDEO_CAPTURE as u32
    != 0
  {
    out.push("desktop-video");
  }
  if out.is_empty() {
    "none".to_string()
  } else {
    out.join(",")
  }
}

/// Human-readable comma-joined names of the permission-prompt bits set in
/// `bits`. Covers only the types we care about surfacing; unknown bits render
/// as `0x<hex>` so unexpected values stay visible in logs.
pub fn format_permission_types(bits: u32) -> String {
  use cef_permission_request_types_t as T;
  const KNOWN: &[(u32, &str)] = &[
    (T::CEF_PERMISSION_TYPE_AR_SESSION as u32, "ar-session"),
    (
      T::CEF_PERMISSION_TYPE_CAMERA_PAN_TILT_ZOOM as u32,
      "camera-ptz",
    ),
    (T::CEF_PERMISSION_TYPE_CAMERA_STREAM as u32, "camera"),
    (
      T::CEF_PERMISSION_TYPE_CAPTURED_SURFACE_CONTROL as u32,
      "captured-surface",
    ),
    (T::CEF_PERMISSION_TYPE_CLIPBOARD as u32, "clipboard"),
    (
      T::CEF_PERMISSION_TYPE_TOP_LEVEL_STORAGE_ACCESS as u32,
      "top-storage",
    ),
    (T::CEF_PERMISSION_TYPE_DISK_QUOTA as u32, "disk-quota"),
    (T::CEF_PERMISSION_TYPE_LOCAL_FONTS as u32, "local-fonts"),
    (T::CEF_PERMISSION_TYPE_GEOLOCATION as u32, "geolocation"),
    (T::CEF_PERMISSION_TYPE_HAND_TRACKING as u32, "hand-tracking"),
    (T::CEF_PERMISSION_TYPE_IDENTITY_PROVIDER as u32, "identity"),
    (T::CEF_PERMISSION_TYPE_IDLE_DETECTION as u32, "idle"),
    (T::CEF_PERMISSION_TYPE_MIC_STREAM as u32, "mic"),
    (T::CEF_PERMISSION_TYPE_MIDI_SYSEX as u32, "midi-sysex"),
    (
      T::CEF_PERMISSION_TYPE_MULTIPLE_DOWNLOADS as u32,
      "multi-download",
    ),
    (T::CEF_PERMISSION_TYPE_NOTIFICATIONS as u32, "notifications"),
    (T::CEF_PERMISSION_TYPE_KEYBOARD_LOCK as u32, "kbd-lock"),
    (T::CEF_PERMISSION_TYPE_POINTER_LOCK as u32, "ptr-lock"),
    (
      T::CEF_PERMISSION_TYPE_PROTECTED_MEDIA_IDENTIFIER as u32,
      "protected-media",
    ),
    (
      T::CEF_PERMISSION_TYPE_REGISTER_PROTOCOL_HANDLER as u32,
      "protocol-handler",
    ),
    (T::CEF_PERMISSION_TYPE_STORAGE_ACCESS as u32, "storage"),
    (T::CEF_PERMISSION_TYPE_VR_SESSION as u32, "vr-session"),
    (
      T::CEF_PERMISSION_TYPE_WEB_APP_INSTALLATION as u32,
      "webapp-install",
    ),
    (
      T::CEF_PERMISSION_TYPE_WINDOW_MANAGEMENT as u32,
      "window-mgmt",
    ),
    (T::CEF_PERMISSION_TYPE_FILE_SYSTEM_ACCESS as u32, "fs"),
    (
      T::CEF_PERMISSION_TYPE_LOCAL_NETWORK_ACCESS as u32,
      "local-net-access",
    ),
    (T::CEF_PERMISSION_TYPE_LOCAL_NETWORK as u32, "local-net"),
    (T::CEF_PERMISSION_TYPE_LOOPBACK_NETWORK as u32, "loopback"),
  ];

  let mut remaining = bits;
  let mut out = Vec::new();
  for (bit, name) in KNOWN {
    if remaining & bit != 0 {
      out.push((*name).to_string());
      remaining &= !bit;
    }
  }
  if remaining != 0 {
    out.push(format!("0x{remaining:x}"));
  }
  if out.is_empty() {
    "none".to_string()
  } else {
    out.join(",")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const DEVICE_AUDIO: u32 =
    cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32;
  const DEVICE_VIDEO: u32 =
    cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE as u32;
  const DESKTOP_AUDIO: u32 =
    cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DESKTOP_AUDIO_CAPTURE as u32;
  const DESKTOP_VIDEO: u32 =
    cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DESKTOP_VIDEO_CAPTURE as u32;

  const MIC: u32 = cef_permission_request_types_t::CEF_PERMISSION_TYPE_MIC_STREAM as u32;
  const CAMERA: u32 = cef_permission_request_types_t::CEF_PERMISSION_TYPE_CAMERA_STREAM as u32;
  const NOTIFICATIONS: u32 =
    cef_permission_request_types_t::CEF_PERMISSION_TYPE_NOTIFICATIONS as u32;
  const GEOLOCATION: u32 = cef_permission_request_types_t::CEF_PERMISSION_TYPE_GEOLOCATION as u32;
  const MIDI_SYSEX: u32 = cef_permission_request_types_t::CEF_PERMISSION_TYPE_MIDI_SYSEX as u32;
  const PROTECTED_MEDIA: u32 =
    cef_permission_request_types_t::CEF_PERMISSION_TYPE_PROTECTED_MEDIA_IDENTIFIER as u32;

  #[test]
  fn media_mic_only_allowed() {
    assert_eq!(allowed_media_permissions(DEVICE_AUDIO), DEVICE_AUDIO);
  }

  #[test]
  fn media_camera_only_allowed() {
    assert_eq!(allowed_media_permissions(DEVICE_VIDEO), DEVICE_VIDEO);
  }

  #[test]
  fn media_desktop_audio_allowed() {
    assert_eq!(allowed_media_permissions(DESKTOP_AUDIO), DESKTOP_AUDIO);
  }

  #[test]
  fn media_desktop_video_allowed() {
    assert_eq!(allowed_media_permissions(DESKTOP_VIDEO), DESKTOP_VIDEO);
  }

  #[test]
  fn media_user_media_combination_allowed() {
    let requested = DEVICE_AUDIO | DEVICE_VIDEO;
    assert_eq!(allowed_media_permissions(requested), requested);
  }

  #[test]
  fn media_display_media_combination_allowed() {
    let requested = DESKTOP_AUDIO | DESKTOP_VIDEO;
    assert_eq!(allowed_media_permissions(requested), requested);
  }

  #[test]
  fn media_empty_request_denied() {
    assert_eq!(allowed_media_permissions(0), 0);
  }

  #[test]
  fn media_unknown_bit_stripped() {
    let unknown_bit: u32 = 1 << 30;
    assert_eq!(
      allowed_media_permissions(DEVICE_AUDIO | unknown_bit),
      DEVICE_AUDIO
    );
    assert_eq!(allowed_media_permissions(unknown_bit), 0);
  }

  #[test]
  fn prompt_mic_accepted() {
    assert!(should_accept_permission_prompt(MIC));
  }

  #[test]
  fn prompt_camera_accepted() {
    assert!(should_accept_permission_prompt(CAMERA));
  }

  #[test]
  fn prompt_notifications_accepted() {
    assert!(should_accept_permission_prompt(NOTIFICATIONS));
  }

  #[test]
  fn prompt_geolocation_denied() {
    assert!(!should_accept_permission_prompt(GEOLOCATION));
  }

  #[test]
  fn prompt_midi_sysex_denied() {
    assert!(!should_accept_permission_prompt(MIDI_SYSEX));
  }

  #[test]
  fn prompt_protected_media_denied() {
    assert!(!should_accept_permission_prompt(PROTECTED_MEDIA));
  }

  #[test]
  fn prompt_mixed_with_denied_is_denied() {
    assert!(!should_accept_permission_prompt(MIC | GEOLOCATION));
  }

  #[test]
  fn prompt_empty_denied() {
    assert!(!should_accept_permission_prompt(0));
  }

  #[test]
  fn format_media_bits_handles_known_and_empty() {
    assert_eq!(format_media_bits(0), "none");
    assert_eq!(format_media_bits(DEVICE_AUDIO), "device-audio");
    assert_eq!(
      format_media_bits(DEVICE_AUDIO | DESKTOP_VIDEO),
      "device-audio,desktop-video"
    );
  }

  #[test]
  fn format_permission_types_handles_known_and_unknown() {
    assert_eq!(format_permission_types(0), "none");
    assert_eq!(format_permission_types(MIC), "mic");
    assert_eq!(format_permission_types(MIC | CAMERA), "camera,mic");
    let unknown_bit: u32 = 1 << 30;
    let rendered = format_permission_types(MIC | unknown_bit);
    assert!(rendered.contains("mic"));
    assert!(rendered.contains("0x40000000"));
  }
}
