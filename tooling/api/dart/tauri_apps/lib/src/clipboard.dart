// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Read and write to the system clipboard.
///
/// This package is also accessible with `window.__TAURI__.clipboard` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.clipboard`](https://tauri.app/v1/api/config/#allowlistconfig.clipboard)
/// in `tauri.conf.json`:
///
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "clipboard": {
///         "all": true, // enable all Clipboard APIs
///         "writeText": true,
///         "readText": true
///       }
///     }
///   }
/// }
/// ```
///
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_clipboard;

import 'dart:async';
import 'helpers/tauri.dart'
    show TauriCommand, TauriCommandMessage, TauriModule, invokeTauriCommand;

/// Writes plain text to the clipboard.
///
/// ```dart
/// import 'package:tauri_apps/api/clipboard.dart' show writeText, readText;
///
/// (() async {
///   await writeText('Tauri is awesome!');
///   assert(await readText() == 'Tauri is awesome!', 'Not so awesome???');
/// })();
/// ```
///
/// @since 1.0.0.
Future<void> writeText(final String text) => invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Clipboard,
        message: TauriCommandMessage(cmd: 'writeText', data: text),
      ),
    );

/// Gets the clipboard content as plain text.
///
/// ```dart
/// import 'package:tauri_apps/api/clipboard.dart' show readText;
///
/// (() async {
///   const String? clipboardText = await readText();
/// })();
/// ```
///
/// @since 1.0.0.
Future<String?> readText() => invokeTauriCommand<String?>(
      const TauriCommand(
        tauriModule: TauriModule.Clipboard,
        message: TauriCommandMessage(
          cmd: 'readText',
          // if data is not set, `serde` will ignore the custom deserializer
          // that is set when the API is not allowlisted
          // ignore: avoid_redundant_argument_values
          data: null,
        ),
      ),
    );
