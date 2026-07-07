/// Permission types that can be requested by the webview.
///
/// See [`crate::WebViewBuilder::with_permission_handler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PermissionKind {
  /// Microphone access permission.
  Microphone,
  /// Camera access permission.
  Camera,
  /// Geolocation access permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION`.
  /// - **Linux**: Supported via `GeolocationPermissionRequest`.
  /// - **Android**: Supported via `WebChromeClient.onGeolocationPermissionsShowPrompt`.
  /// - **macOS / iOS**: Not yet supported by platform backends.
  Geolocation,
  /// Notifications permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS`.
  /// - **Linux**: Supported via `NotificationPermissionRequest`.
  /// - **macOS / Android / iOS**: Not yet supported by platform backends.
  Notifications,
  /// Clipboard read permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  ClipboardRead,
  /// Display capture permission (for getDisplayMedia).
  DisplayCapture,
  /// Midi access permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_MIDI_SYSTEM_EXCLUSIVE_MESSAGES`.
  /// - **Android**: Supported via `android.webkit.resource.MIDI_SYSEX`.
  /// - **macOS / Linux / iOS**: Not yet supported by platform backends.
  Midi,
  /// Sensors (accelerometer, gyroscope, etc.) access permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_OTHER_SENSORS`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  Sensors,
  /// Media key system access permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Android**: Supported via `android.webkit.resource.PROTECTED_MEDIA_ID`.
  /// - **Windows / macOS / Linux / iOS**: Not yet supported by platform backends.
  MediaKeySystemAccess,
  /// Local fonts access permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_LOCAL_FONTS`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  LocalFonts,
  /// Window management permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_WINDOW_MANAGEMENT`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  WindowManagement,
  /// Pointer lock permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux**: Supported via `PointerLockPermissionRequest`.
  /// - **Windows / macOS / Android / iOS**: Not yet supported by platform backends.
  PointerLock,
  /// Automatic downloads permission (multiple downloads without user interaction).
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_MULTIPLE_AUTOMATIC_DOWNLOADS`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  AutomaticDownloads,
  /// File system access permission (read/write via File System Access API).
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_FILE_READ_WRITE`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  FileSystemAccess,
  /// Media autoplay permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: Supported via `COREWEBVIEW2_PERMISSION_KIND_AUTOPLAY`.
  /// - **macOS / Linux / Android / iOS**: Not yet supported by platform backends.
  Autoplay,
  /// Other unrecognized permission type.
  Other,
}

impl std::fmt::Display for PermissionKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Microphone => write!(f, "microphone"),
      Self::Camera => write!(f, "camera"),
      Self::Geolocation => write!(f, "geolocation"),
      Self::Notifications => write!(f, "notifications"),
      Self::ClipboardRead => write!(f, "clipboard-read"),
      Self::DisplayCapture => write!(f, "display-capture"),
      Self::Midi => write!(f, "midi"),
      Self::Sensors => write!(f, "sensors"),
      Self::MediaKeySystemAccess => write!(f, "media-key-system-access"),
      Self::LocalFonts => write!(f, "local-fonts"),
      Self::WindowManagement => write!(f, "window-management"),
      Self::PointerLock => write!(f, "pointer-lock"),
      Self::AutomaticDownloads => write!(f, "automatic-downloads"),
      Self::FileSystemAccess => write!(f, "file-system-access"),
      Self::Autoplay => write!(f, "autoplay"),
      Self::Other => write!(f, "other"),
    }
  }
}

/// Response for permission requests.
///
/// See [`crate::WebViewBuilder::with_permission_handler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PermissionResponse {
  /// Grant the permission.
  ///
  /// ## Platform-specific
  ///
  /// - **Android**: Not supported for runtime permissions; the normal Android
  ///   permission flow is used instead.
  Allow,
  /// Deny the permission.
  Deny,
  /// Use the platform or browser default behavior.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / macOS / Android**: The default behavior is to continue the
  ///   platform or browser permission flow.
  /// - **Linux**: The default behavior is [`Self::Deny`]
  #[default]
  Default,
}

impl std::fmt::Display for PermissionResponse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Allow => write!(f, "allow"),
      Self::Deny => write!(f, "deny"),
      Self::Default => write!(f, "default"),
    }
  }
}
