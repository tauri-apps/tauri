// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::webview::{PermissionKind, PermissionResponse};

pub(super) fn to_wry_permission_kind(kind: wry::PermissionKind) -> PermissionKind {
  match kind {
    wry::PermissionKind::Microphone => PermissionKind::Microphone,
    wry::PermissionKind::Camera => PermissionKind::Camera,
    wry::PermissionKind::Geolocation => PermissionKind::Geolocation,
    wry::PermissionKind::Notifications => PermissionKind::Notifications,
    wry::PermissionKind::ClipboardRead => PermissionKind::ClipboardRead,
    wry::PermissionKind::DisplayCapture => PermissionKind::DisplayCapture,
    wry::PermissionKind::Midi => PermissionKind::Midi,
    wry::PermissionKind::Sensors => PermissionKind::Sensors,
    wry::PermissionKind::MediaKeySystemAccess => PermissionKind::MediaKeySystemAccess,
    wry::PermissionKind::LocalFonts => PermissionKind::LocalFonts,
    wry::PermissionKind::WindowManagement => PermissionKind::WindowManagement,
    wry::PermissionKind::PointerLock => PermissionKind::PointerLock,
    wry::PermissionKind::AutomaticDownloads => PermissionKind::AutomaticDownloads,
    wry::PermissionKind::FileSystemAccess => PermissionKind::FileSystemAccess,
    wry::PermissionKind::Autoplay => PermissionKind::Autoplay,
    wry::PermissionKind::Other => PermissionKind::Other,
    _ => PermissionKind::Other,
  }
}

pub(super) fn to_wry_permission_response(response: PermissionResponse) -> wry::PermissionResponse {
  match response {
    PermissionResponse::Allow => wry::PermissionResponse::Allow,
    PermissionResponse::Deny => wry::PermissionResponse::Deny,
    PermissionResponse::Default => wry::PermissionResponse::Default,
  }
}
