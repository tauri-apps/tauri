// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Send toast notifications (brief auto-expiring OS window element) to your
/// user.
/// Can also be used with the Notification Web API.
///
/// This package is also accessible with `window.__TAURI__.notification` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.notification`](https://tauri.app/v1/api/config/#allowlistconfig.notification)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "notification": {
///         "all": true // enable all notification APIs
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_notification;

import 'package:universal_html/html.dart' as html;

import './helpers/tauri.dart';

/// Options to send a notification.
///
/// @since 1.0.0
class Options {
  const Options({required this.title, this.body, this.icon});

  /// Notification title.
  final String title;

  /// Optional notification body.
  final String? body;

  /// Optional notification icon.
  final String? icon;
}

/// Possible permission values.
enum Permission {
  granted,
  denied,
  defaultPermission;

  String get permissionAsString {
    switch (this) {
      case Permission.granted:
        return 'granted';
      case Permission.denied:
        return 'denied';
      case Permission.defaultPermission:
        return 'default';
    }
  }

  static Permission of(final String data) {
    if (data == Permission.granted.permissionAsString) {
      return Permission.granted;
    }
    if (data == Permission.denied.permissionAsString) {
      return Permission.denied;
    }

    return Permission.defaultPermission;
  }
}

/// Checks if the permission to send notifications is granted.
///
/// ```dart
/// import 'package:tauri_apps/api/notification.dart' show isPermissionGranted;
///
/// final Future<bool> permissionGranted = isPermissionGranted();
/// ```
///
/// @since 1.0.0
Future<bool> isPermissionGranted() async {
  if (html.Notification.permission !=
      Permission.defaultPermission.permissionAsString) {
    return Future<bool>.value(
      html.Notification.permission == Permission.granted.permissionAsString,
    );
  }

  return invokeTauriCommand(
    const TauriCommand(
      tauriModule: TauriModule.Notification,
      message: TauriCommandMessage(
        cmd: 'isNotificationPermissionGranted',
      ),
    ),
  );
}

/// Requests the permission to send notifications.
///
/// ```dart
/// import 'package:tauri_apps/api/notification.dart'
///       show isPermissionGranted, requestPermission;
///
/// final Future<void> f = isPermissionGranted()
///   .then(
///     (final bool permissionGranted) {
///       if (!permissionGranted) {
///         requestPermission().then(
///           (final Permission permission)
///               => permissionGranted = permission == Permission.granted,
///         );
///       }
///     },
///   );
/// ```
///
/// @returns A future resolving to whether the user granted the permission or
/// not.
///
/// @since 1.0.0
Future<Permission> requestPermission() async =>
    Permission.of(await html.Notification.requestPermission());

/// Sends a notification to the user.
///
/// ```dart
/// import 'package:tauri_apps/api/notification.dart'
///     show isPermissionGranted, requestPermission, sendNotification;
///
/// final Future<void> f = isPermissionGranted()
///   .then(
///     (final bool permissionGranted) {
///       if (!permissionGranted) {
///         requestPermission()
///           .then(
///             (final permission) =>
///                 permissionGranted = permission == Permission.granted,
///             )
///           .then(
///             (final bool granted) {
///               sendNotification<String>('Tauri is awesome!');
///               sendNotification<Options>(
///                 Options(
///                   title: 'TAURI',
///                   body: 'Tauri is awesome!',
///                 ),
///               );
///             },
///           );
///       }
///     },
///   );
/// ```
///
/// @since 1.0.0
void sendNotification<T>(final T options) {
  if (options is String) {
    html.Notification(options);
  } else if (options is Options) {
    html.Notification(
      options.title,
      body: options.body,
      icon: options.icon,
    );
  } else {
    throw UnsupportedError(
      'options must be either a String or Options object, '
      'but ${options.runtimeType} was provided.',
    );
  }
}
