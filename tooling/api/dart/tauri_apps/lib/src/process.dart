// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Perform operations on the current process.
///
/// This package is also accessible with `window.__TAURI__.process` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
library tauri_process;

import 'helpers/tauri.dart';

/// Exits immediately with the given `exitCode`.
///
/// ```dart
/// import 'package:tauri_apps/api/process.dart' show exit;
///
/// (() async {
///   await exit(1);
/// })();
/// ```
///
///  * [exitCode] The exit code to use.
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> exit([final int exitCode = 0]) async => invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Process,
        message: TauriCommandMessage(
          cmd: 'exit',
          exitCode: exitCode,
        ),
      ),
    );

/// Exits the current instance of the app then relaunches it.
///
/// ```dart
/// import 'package:tauri_apps/api/process.dart' show relaunch;
///
/// (() {
///   await relaunch();
/// })
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> relaunch() async => invokeTauriCommand<void>(
      const TauriCommand(
        tauriModule: TauriModule.Process,
        message: TauriCommandMessage(
          cmd: 'relaunch',
        ),
      ),
    );
