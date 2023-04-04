// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Get application metadata.
///
/// This package is also accessible with `window.__TAURI__.app` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.app`](https://tauri.app/v1/api/config/#allowlistconfig.app)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "app": {
///         "all": true, // enable all app APIs
///         "show": true,
///         "hide": true
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_app;

import './helpers/tauri.dart';

/// Gets the application version.
///
/// ```dart
/// import getVersion 'package:tauri_apps/api/app.dart';
///
/// (() async {
///   final String appVersion = await getVersion();
/// })();
/// ```
///
/// @since 1.0.0
Future<String> getVersion() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.App,
        message: TauriCommandMessage(
          cmd: 'getAppVersion',
        ),
      ),
    );

/// Gets the application name.
///
/// ```dart
/// import getName 'package:tauri_apps/api/app.dart';
///
/// (() async {
///   final String appName = await getName();
/// })();
/// ```
///
/// @since 1.0.0
Future<String> getName() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.App,
        message: TauriCommandMessage(
          cmd: 'getAppName',
        ),
      ),
    );

/// Gets the Tauri version.
///
/// ```dart
/// import getTauriVersion 'package:tauri_apps/api/app.dart';
///
/// (() async {
///   final String tauriVersion = await getTauriVersion();
/// })();
/// ```
///
/// @since 1.0.0
Future<String> getTauriVersion() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.App,
        message: TauriCommandMessage(
          cmd: 'getTauriVersion',
        ),
      ),
    );

/// Shows the application on macOS. This function does not automatically focus
/// any specific app window.
///
/// ```dart
/// import show 'package:tauri_apps/api/app.dart';
///
/// (() async {
///   await show();
/// })();
/// ```
///
/// @since 1.2.0
Future<void> show() async => invokeTauriCommand<void>(
      const TauriCommand(
        tauriModule: TauriModule.App,
        message: TauriCommandMessage(
          cmd: 'show',
        ),
      ),
    );

/// Hides the application on macOS.
///
/// ```dart
/// import 'package:tauri_apps/api/app.dart';
///
/// (() async {
///   await hide();
/// })();
/// ```
///
/// @since 1.2.0
Future<void> hide() async => invokeTauriCommand<void>(
      const TauriCommand(
        tauriModule: TauriModule.App,
        message: TauriCommandMessage(
          cmd: 'hide',
        ),
      ),
    );
