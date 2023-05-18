// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Register global shortcuts.
///
/// This package is also accessible with `window.__TAURI__.globalShortcut`
/// when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.globalShortcut`](https://tauri.app/v1/api/config/#allowlistconfig.globalshortcut)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "globalShortcut": {
///         "all": true // enable all global shortcut APIs
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_global_shortcuts;

import 'helpers/tauri.dart';
import 'tauri.dart';

typedef ShortcutHandler = void Function(String shortcut);

/// Register a global shortcut.
///
/// ```dart
/// import 'package:tauri_apps/api/global_shortcut.dart' show register;
///
/// register(
///   shortcut: 'CommandOrControl+Shift+C',
///   handler: () => print('Shortcut triggered'),
/// );
/// ```
///
///   * [shortcut]: Shortcut definition, modifiers and key separated by "+" e.g.
///     CmdOrControl+Q,
///   * [handler]: Shortcut handler callback - takes the triggered shortcut as
///     argument.
///
/// @since 1.0.0
Future<void> register({
  required final String shortcut,
  required final ShortcutHandler handler,
}) async =>
    invokeTauriCommand(
      TauriCommand(
        tauriModule: TauriModule.GlobalShortcut,
        message: TauriCommandMessage(
          cmd: 'register',
          shortcut: shortcut,
          handler: transformCallback<String>(callback: handler),
        ),
      ),
    );

/// Register a collection of global shortcuts.
///
/// ```dart
/// import 'package:tauri_apps/api/global_shortcut.dart' show registerAll;
///
/// registerAll(
///   <String>>['CommandOrControl+Shift+C', 'Ctrl+Alt+F12'],
///   (final String shortcut) => print('Shortcut ${shortcut} triggered.'),
/// );
/// ```
///
///   * shortcuts Array of shortcut definitions, modifiers and key separated by
///     "+" e.g. CmdOrControl+Q,
///   * handler Shortcut handler callback - takes the triggered shortcut as
///     argument.
///
/// @since 1.0.0
Future<void> registerAll({
  required final List<String> shortcuts,
  required final ShortcutHandler handler,
}) async =>
    invokeTauriCommand(
      TauriCommand(
        tauriModule: TauriModule.GlobalShortcut,
        message: TauriCommandMessage(
          cmd: 'registerAll',
          shortcuts: shortcuts,
          handler: transformCallback<String>(callback: handler),
        ),
      ),
    );

/// Determines whether the given shortcut is registered by this application or
/// not.
///
/// ```dart
/// import 'package:tauri_apps/api/global_shortcut.dart' show isRegistered;
/// final Future<bool> registered = isRegistered(
///     shortcut: 'CommandOrControl+P',
///   ).then(
///     (final bool isRegistered) => isRegistered,
///   );
/// ```
///
///   * shortcut Array of shortcut definitions, modifiers and key separated by
///     "+" e.g. CmdOrControl+Q.
///
/// @since 1.0.0
Future<bool> isRegistered({required final String shortcut}) async =>
    invokeTauriCommand(
      const TauriCommand(
        tauriModule: TauriModule.GlobalShortcut,
        message: TauriCommandMessage(
          cmd: 'isRegistered',
        ),
      ),
    );

/// Unregister a global shortcut.
///
/// ```dart
/// import 'package:tauri_apps/api/global_shortcut.dart' show unregister;
///
/// unregister(shortcut: 'CmdOrControl+Space');
/// ```
///
///   * shortcut shortcut definition, modifiers and key separated by "+" e.g.
///     CmdOrControl+Q.
///
/// @since 1.0.0
Future<void> unregister(final String shortcut) async => invokeTauriCommand(
      const TauriCommand(
        tauriModule: TauriModule.GlobalShortcut,
        message: TauriCommandMessage(
          cmd: 'unregister',
        ),
      ),
    );

/// Unregisters all shortcuts registered by the application.
///
/// ```dart
/// import 'package:tauri_apps/api/global_shortcut.dart' show unregisterAll;
///
/// unregisterAll();
/// ```
///
/// @since 1.0.0
Future<void> unregisterAll() async => invokeTauriCommand(
      const TauriCommand(
        tauriModule: TauriModule.GlobalShortcut,
        message: TauriCommandMessage(
          cmd: 'unregisterAll',
        ),
      ),
    );
