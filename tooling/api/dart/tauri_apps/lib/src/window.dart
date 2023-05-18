// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names, avoid_field_initializers_in_const_classes

/// Provides APIs to create windows, communicate with other windows and
/// manipulate the current window.
///
/// This package is also accessible with `window.__TAURI__.window` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.window`](https://tauri.app/v1/api/config/#allowlistconfig.window)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "window": {
///         "all": true, // enable all window APIs
///         "create": true, // enable window creation
///         "center": true,
///         "requestUserAttention": true,
///         "setResizable": true,
///         "setTitle": true,
///         "maximize": true,
///         "unmaximize": true,
///         "minimize": true,
///         "unminimize": true,
///         "show": true,
///         "hide": true,
///         "close": true,
///         "setDecorations": true,
///         "setAlwaysOnTop": true,
///         "setContentProtected": true,
///         "setSize": true,
///         "setMinSize": true,
///         "setMaxSize": true,
///         "setPosition": true,
///         "setFullscreen": true,
///         "setFocus": true,
///         "setIcon": true,
///         "setSkipTaskbar": true,
///         "setCursorGrab": true,
///         "setCursorVisible": true,
///         "setCursorIcon": true,
///         "setCursorPosition": true,
///         "setIgnoreCursorEvents": true,
///         "startDragging": true,
///         "print": true
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
///
/// ## Window events
///
/// Events can be listened to using `appWindow.listen`:
/// ```dart
/// import 'package:tauri_apps/api/window.dart' show appWindow;
///
/// appWindow.listen<void>(
///   event: EventName('my-window-event'),
///   handler: (final EventName event, final _) {},
/// );
/// ```
library tauri_window;

import 'dart:async';
import 'dart:typed_data';

import 'package:collection/collection.dart';
import 'package:universal_html/html.dart' as html;
import 'package:universal_html/js.dart' as js;

import './helpers/tauri.dart';
import 'event.dart' hide emit, listen, once;
import 'event.dart' as tauri_events show emit, listen, once;
import 'http.dart';

part 'helpers/window.dart';
part 'helpers/window_definitions.dart';
part 'helpers/window_events.dart';
part 'helpers/window_functions.dart';

/// A webview window handle allows emitting and listening to events from the
/// backend that are tied to the window.
///
/// @ignore
/// @since 1.0.0
class WebviewWindowHandle {
  const WebviewWindowHandle({required this.label})
      : listeners = const <EventName, EventCallbackList>{};

  /// The window label. It is a unique identifier for the window, can be used to
  /// reference it later.
  final WindowLabel label;

  /// Local event listeners.
  final EventCallbackRegistry listeners;

  /// Listen to an event emitted by the backend that is tied to the webview
  /// window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// Future<UnlistenFn> listenToWindowEvents() async =>
  ///   appWindow.listen<void>(
  ///     event: EventName('state-changed'),
  ///     handler: (final Event<void> event) => print(
  ///       'Got error: $payload.',
  ///     ),
  ///   );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// final Future<UnlistenFn> unlisten = listenToWindowEvents();
  /// unlisten.then((final UnlistenFn fn) => fn());
  /// ```
  ///
  ///   * [event]: Event name. Must include only alphanumeric characters, `-`,
  ///     `/`, `:` and `_`.
  ///   * [handler]: Event handler.
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  Future<UnlistenFn> listen<T>({
    required final EventName event,
    required final EventCallback<T> handler,
  }) async {
    if (_handleTauriEvent<T>(event: event, handler: handler)) {
      return _tauriEventHandler<T>(event: event, handler: handler);
    }

    return tauri_events.listen<T>(
      event: event,
      windowLabel: label,
      handler: handler,
    );
  }

  /// Listen to an one-off event emitted by the backend that is tied to the
  /// webview window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// Future<UnlistenFn> listenToWindowEvents() async =>
  ///   appWindow.once<void>(
  ///     event: EventName('initialized'),
  ///     handler: (final Event<void> event) => print(
  ///       'Window initialized!',
  ///     ),
  ///   );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// final Future<UnlistenFn> unlisten = listenToWindowEvents();
  /// unlisten.then((final UnlistenFn fn) => fn());
  /// ```
  ///
  ///   * [event]: Event name. Must include only alphanumeric characters, `-`,
  ///     `/`, `:` and `_`.
  ///   * [handler]: Event handler.
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  Future<UnlistenFn> once<T>({
    required final EventName event,
    required final EventCallback<T> handler,
  }) async {
    if (_handleTauriEvent<T>(event: event, handler: handler)) {
      return _tauriEventHandler<T>(event: event, handler: handler);
    }

    return tauri_events.once<T>(
      event: event,
      windowLabel: label,
      handler: handler,
    );
  }

  /// Emits an event to the backend, tied to the webview window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.emit<Map<String, dynamic>>(
  ///   event: EventName('window-loaded'),
  ///   payload: <String, dynamic>{
  ///     'loggedIn': true,
  ///     'token': 'authToken',
  ///   },
  /// ).then(() => print('Tauri is awesome!'));
  /// ```
  ///
  ///   * [event]: Event name. Must include only alphanumeric characters,
  ///     `-`, `/`, `:` and `_`.
  ///   * [payload]: Event payload.
  Future<void> emit<T>({
    required final EventName event,
    required final T payload,
  }) async {
    if (localTauriEvents.contains(event)) {
      final EventCallbackList handlers =
          listeners[event] ?? <DynamicEventCallback>[];
      for (final DynamicEventCallback handler in handlers) {
        handler(
          Event<dynamic>(
            event: event,
            id: -1,
            windowLabel: label,
            payload: payload,
          ),
        );
      }
      return;
    }

    return tauri_events.emit<T>(
      event: event,
      windowLabel: label,
      payload: payload,
    );
  }

  /// @ignore
  bool _handleTauriEvent<T>({
    required final EventName event,
    required final EventCallback<T> handler,
  }) {
    if (localTauriEvents.contains(event)) {
      final DynamicEventCallback dynamicHandler =
          handler as DynamicEventCallback;
      if (!listeners.containsKey(event)) {
        listeners[event] = <DynamicEventCallback>[dynamicHandler];
      } else {
        listeners[event]!.add(dynamicHandler);
      }
      return true;
    }
    return false;
  }

  Future<UnlistenFn> _tauriEventHandler<T>({
    required final EventName event,
    required final EventCallback<T> handler,
  }) =>
      Future<UnlistenFn>(
        () async {
          final EventCallbackList handlers =
              listeners[event] ?? <DynamicEventCallback>[];
          final EventCallbackList result = handlers
              .splice(
                handlers.indexOf(handler as DynamicEventCallback),
                1,
              )
              .toList();

          if (result.isEmpty) {
            throw StateError('Handler has not been registered.');
          }

          return tauri_events.listen<T>(
            event: event,
            windowLabel: label,
            handler: result.first,
          );
        },
      );
}

/// Manage the current window object.
///
/// @ignore
/// @since 1.0.0
class WindowManager extends WebviewWindowHandle {
  const WindowManager({required super.label}) : super();
  // Getters
  /// The scale factor that can be used to map physical pixels to logical
  /// pixels.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.scaleFactor().then((final double factor) => print(factor));
  /// ```
  ///
  /// @returns The window's monitor scale factor.
  Future<double> scaleFactor() async => invokeTauriCommand<double>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'scaleFactor'}
            },
          ),
        ),
      );

  /// The position of the top-left hand corner of the window's client area
  /// relative to the top-left hand corner of the desktop.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.innerPosition().then(
  ///   (final PhysicalPosition position) => print(position),
  /// );
  /// ```
  ///
  /// @returns The window's inner position.
  Future<PhysicalPosition> innerPosition() async =>
      invokeTauriCommand<Position>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'innerPosition'}
            },
          ),
        ),
      ).then((final Position p) => PhysicalPosition(x: p.x, y: p.y));

  /// The position of the top-left hand corner of the window relative to the
  /// top-left hand corner of the desktop.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.outerPosition().then(
  ///   (final PhysicalPosition position) => print(position),
  /// );
  /// ```
  ///
  /// @returns The window's outer position.
  Future<PhysicalPosition> outerPosition() async =>
      invokeTauriCommand<Position>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'outerPosition'}
            },
          ),
        ),
      ).then((final Position p) => PhysicalPosition(x: p.x, y: p.y));

  /// The physical size of the window's client area.
  /// The client area is the content of the window, excluding the title bar and
  /// borders.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.innerSize().then(
  ///   (final PhysicalSize size) => print(size),
  /// );
  /// ```
  ///
  /// @returns The window's inner size.
  Future<PhysicalSize> innerSize() async => invokeTauriCommand<Size>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'innerSize'}
            },
          ),
        ),
      ).then((final Size s) => PhysicalSize(width: s.width, height: s.height));

  /// The physical size of the entire window.
  /// These dimensions include the title bar and borders. If you don't want that
  /// (and you usually don't), use inner_size instead.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.outerSize().then(
  ///   (final PhysicalSize size) => print(size),
  /// );
  /// ```
  ///
  /// @returns The window's outer size.
  Future<PhysicalSize> outerSize() async => invokeTauriCommand<Size>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'outerSize'}
            },
          ),
        ),
      ).then((final Size s) => PhysicalSize(width: s.width, height: s.height));

  /// Gets the window's current fullscreen state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.isFullscreen().then(
  ///   (final bool fullscreen) => print(fullscreen),
  /// );
  /// ```
  ///
  /// @returns Whether the window is in fullscreen mode or not.
  Future<bool> isFullscreen() async => invokeTauriCommand<bool>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'isFullscreen'}
            },
          ),
        ),
      );

  /// Gets the window's current minimized state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.isMinimized().then(
  ///   (final bool minimized) => print(minimized),
  /// );
  /// ```
  ///
  /// @since 1.3.0
  Future<bool> isMinimized() async => invokeTauriCommand<bool>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'isMinimized'}
            },
          ),
        ),
      );

  /// Gets the window's current maximized state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.isMaximized().then(
  ///   (final bool maximized) => print(maximized),
  /// );
  /// ```
  ///
  /// @returns Whether the window is maximized or not.
  Future<bool> isMaximized() async => invokeTauriCommand<bool>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'isMaximized'}
            },
          ),
        ),
      );

  /// Gets the window's current decorated state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.isDecorated().then(
  ///   (final bool decorated) => print(decorated),
  /// );
  /// ```
  ///
  /// @returns Whether the window is decorated or not.
  Future<bool> isDecorated() async => invokeTauriCommand<bool>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'isDecorated'}
            },
          ),
        ),
      );

  /// Gets the window's current resizable state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.isResizable().then(
  ///   (final bool resizable) => print(resizable),
  /// );
  /// ```
  ///
  /// @returns Whether the window is resizable or not.
  Future<bool> isResizable() async => invokeTauriCommand<bool>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'isResizable'}
            },
          ),
        ),
      );

  /// Gets the window's current visible state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.isVisible().then(
  ///   (final bool visible) => print(visible),
  /// );
  /// ```
  ///
  /// @returns Whether the window is visible or not.
  Future<bool> isVisible() async => invokeTauriCommand<bool>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'isVisible'}
            },
          ),
        ),
      );

  /// Gets the window's current title.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.title().then(
  ///   (final String t) => print(t),
  /// );
  /// ```
  ///
  /// @since 1.3.0
  Future<String> title() async => invokeTauriCommand<String>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'title'}
            },
          ),
        ),
      );

  /// Gets the window's current theme.
  ///
  /// #### Platform-specific
  ///
  ///   - **macOS:** Theme was introduced on macOS 10.14. Returns `light` on
  ///     macOS 10.13 and below.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.theme().then(
  ///   (final Theme? t) => print(t),
  /// );
  /// ```
  ///
  /// @returns The window theme.
  Future<Theme?> theme() async => invokeTauriCommand<Theme?>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'theme'}
            },
          ),
        ),
      );

  // Setters
  /// Centers the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.center().then(
  ///   () => print('Tauri is awesome!'),
  /// );
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> center() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'center'}
            },
          ),
        ),
      );

  /// Requests user attention to the window, this has no effect if the
  /// application is already focused. How requesting for user attention
  /// manifests is platform dependent, see [UserAttentionType] for details.
  ///
  /// Providing `null` will unset the request for user attention. Unsetting the
  /// request for user attention might not be done automatically by the WM when
  /// the window receives input.
  ///
  /// #### Platform-specific
  ///
  ///   - **macOS:** `null` has no effect.
  ///   - **Linux:** Urgency levels have the same effect.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.requestUserAttention();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> requestUserAttention(
    final UserAttentionType? requestType,
  ) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'requestUserAttention',
                'payload': requestType
              }
            },
          ),
        ),
      );

  /// Updates the window resizable flag.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setResizable(resizable: false);
  /// ```
  ///
  ///   * [resizable]
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setResizable({required final bool resizable}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setResizable',
                'payload': resizable
              }
            },
          ),
        ),
      );

  /// Sets the window title.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setTitle('Tauri');
  /// ```
  ///
  ///   * [title]: The new title.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setTitle(final String title) async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'setTitle', 'payload': title}
            },
          ),
        ),
      );

  /// Maximizes the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.maximize();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> maximize() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'maximize'}
            },
          ),
        ),
      );

  /// Unmaximizes the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.unmaximize();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> unmaximize() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'unmaximize'}
            },
          ),
        ),
      );

  /// Toggles the window maximized state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.toggleMaximize();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> toggleMaximize() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'toggleMaximize'}
            },
          ),
        ),
      );

  /// Minimizes the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.minimize();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> minimize() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'minimize'}
            },
          ),
        ),
      );

  /// Unminimizes the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.unminimize();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> unminimize() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'unminimize'}
            },
          ),
        ),
      );

  /// Sets the window visibility to true.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.show();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> show() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'show'}
            },
          ),
        ),
      );

  /// Sets the window visibility to false.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.hide();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> hide() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'hide'}
            },
          ),
        ),
      );

  /// Closes the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.close();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> close() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'close'}
            },
          ),
        ),
      );

  /// Whether the window should have borders and bars.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.setDecorations(decorations: false);
  /// ```
  ///
  ///   * decorations Whether the window should have borders and bars.
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setDecorations({required final bool decorations}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setDecorations',
                'payload': decorations
              }
            },
          ),
        ),
      );

  /// Whether the window should always be on top of other windows.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.setAlwaysOnTop(alwaysOnTop: true);
  /// ```
  ///
  ///   * [alwaysOnTop]: Whether the window should always be on top of other
  ///     windows or not.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setAlwaysOnTop({required final bool alwaysOnTop}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setAlwaysOnTop',
                'payload': alwaysOnTop
              }
            },
          ),
        ),
      );

  /// Prevents the window contents from being captured by other apps.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.setContentProtected(protected: true);
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  ///
  /// @since 1.2.0
  Future<void> setContentProtected({required final bool protected}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setContentProtected',
                'payload': protected
              }
            },
          ),
        ),
      );

  /// Resizes the window with a new inner size.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow, LogicalSize;
  ///
  /// appWindow.setSize(size: LogicalSize(600, 500));
  /// ```
  ///
  ///   * [size]: The logical or physical inner size.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setSize<T extends SizedWindow>(final T size) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setSize',
                'payload': <dynamic, dynamic>{
                  'type': size.type,
                  'data': <dynamic, dynamic>{
                    'width': size.width,
                    'height': size.height
                  }
                }
              }
            },
          ),
        ),
      );

  /// Sets the window minimum inner size. If the `size` argument is not
  /// provided, the constraint is unset.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow, PhysicalSize;
  ///
  /// appWindow.setMinSize(PhysicalSize(600, 500));
  /// ```
  ///
  ///   * [size]: The logical or physical inner size, or `null` to unset the
  ///     constraint.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setMinSize<T extends SizedWindow>(final T size) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setMinSize',
                'payload': <dynamic, dynamic>{
                  'type': size.type,
                  'data': <dynamic, dynamic>{
                    'width': size.width,
                    'height': size.height
                  }
                }
              }
            },
          ),
        ),
      );

  /// Sets the window maximum inner size. If the `size` argument is undefined,
  /// the constraint is unset.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow, LogicalSize;
  ///
  /// appWindow.setMaxSize(LogicalSize(600, 500));
  /// ```
  ///
  ///   * [size]: The logical or physical inner size, or `null` to unset the
  ///     constraint.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setMaxSize<T extends SizedWindow>(final T size) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setMaxSize',
                'payload': <dynamic, dynamic>{
                  'type': size.type,
                  'data': <dynamic, dynamic>{
                    'width': size.width,
                    'height': size.height
                  }
                }
              }
            },
          ),
        ),
      );

  /// Sets the window outer position.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart'
  ///     show appWindow, LogicalPosition;
  ///
  /// appWindow.setPosition(LogicalPosition(600, 500));
  /// ```
  ///
  ///   * [position]: The new position, in logical or physical pixels.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setPosition<T extends PositionedWindow>(
    final T position,
  ) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setPosition',
                'payload': <dynamic, dynamic>{
                  'type': position.type,
                  'data': <dynamic, dynamic>{'x': position.x, 'y': position.y}
                }
              }
            },
          ),
        ),
      );

  /// Sets the window fullscreen state.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setFullscreen(fullscreen: true);
  /// ```
  ///
  ///   * [fullscreen]: Whether the window should go to fullscreen or not.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setFullscreen({required final bool fullscreen}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setFullscreen',
                'payload': fullscreen
              }
            },
          ),
        ),
      );

  /// Bring the window to front and focus.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.setFocus();
  /// ```
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setFocus() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'setFocus'}
            },
          ),
        ),
      );

  /// Sets the window icon.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.setIcon('/tauri/awesome.png');
  /// ```
  ///
  /// Note that you need the `icon-ico` or `icon-png` Cargo features to use this
  /// API.
  ///
  /// To enable it, change your Cargo.toml file:
  /// ```toml
  /// [dependencies]
  /// tauri = { version = "...", features = ["...", "icon-png"] }
  /// ```
  ///
  ///   * [icon]: Icon bytes or path to the icon file.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setIcon<T>(final T icon) async {
    dynamic icon_ = icon;
    if (icon is! Uint8List && icon is! String) {
      if (icon is Iterable<int>) {
        icon_ = Uint8List.fromList(icon.toList());
      }
      if (icon is ByteBuffer) {
        icon_ = Uint8List.view(icon);
      }

      throw InvalidTypeException<T>(icon, 'icon');
    }
    return invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Window,
        message: TauriCommandMessage(
          cmd: 'manage',
          data: <dynamic, dynamic>{
            'label': label,
            'cmd': <dynamic, dynamic>{
              'type': 'setIcon',
              'payload': <dynamic, dynamic>{
                // correctly serialize Uint8Arrays
                'icon': (icon is String) ? icon : List<int>.from(icon_),
              }
            }
          },
        ),
      ),
    );
  }

  /// Whether the window icon should be hidden from the taskbar or not.
  ///
  /// #### Platform-specific
  ///
  ///   - **macOS:** Unsupported.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// appWindow.setSkipTaskbar(skip: true);
  /// ```
  ///
  ///   * [skip]: true to hide window icon, false to show it.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setSkipTaskbar({required final bool skip}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setSkipTaskbar',
                'payload': skip
              }
            },
          ),
        ),
      );

  /// Grabs the cursor, preventing it from leaving the window.
  ///
  /// There's no guarantee that the cursor will be hidden. You should
  /// hide it by yourself if you want so.
  ///
  /// #### Platform-specific
  ///
  ///   - **Linux:** Unsupported.
  ///   - **macOS:** This locks the cursor in a fixed location, which looks
  ///     visually awkward.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setCursorGrab(grab: true);
  /// ```
  ///
  ///   * [grab]: `true` to grab the cursor icon, `false` to release it.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setCursorGrab({required final bool grab}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setCursorGrab',
                'payload': grab
              }
            },
          ),
        ),
      );

  /// Modifies the cursor's visibility.
  ///
  /// #### Platform-specific
  ///
  ///   - **Windows:** The cursor is only hidden within the confines of the
  ///     window.
  ///   - **macOS:** The cursor is hidden as long as the window has input focus,
  ///     even if the cursor is outside of the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setCursorVisible(visible: false);
  /// ```
  ///
  ///   * [visible]: If `false`, this will hide the cursor. If `true`, this will
  ///     show the cursor.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setCursorVisible({required final bool visible}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setCursorVisible',
                'payload': visible
              }
            },
          ),
        ),
      );

  /// Modifies the cursor icon of the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setCursorIcon(CursorIcon.help);
  /// ```
  ///
  ///   * icon The new cursor icon.
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setCursorIcon(final CursorIcon icon) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setCursorIcon',
                'payload': icon
              }
            },
          ),
        ),
      );

  /// Changes the position of the cursor in window coordinates.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart'
  ///   show appWindow, LogicalPosition;
  ///
  /// appWindow.setCursorPosition(LogicalPosition(600, 300));
  /// ```
  ///
  ///   * position The new cursor position.
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setCursorPosition<T extends PositionedWindow>(
    final T position,
  ) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setCursorPosition',
                'payload': <dynamic, dynamic>{
                  'type': position.type,
                  'data': <dynamic, dynamic>{'x': position.x, 'y': position.y}
                }
              }
            },
          ),
        ),
      );

  /// Changes the cursor events behavior.
  ///
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.setIgnoreCursorEvents(ignore: true);
  /// ```
  ///
  ///   * [ignore]: `true` to ignore the cursor events; `false` to process them
  ///     as usual.
  ///
  /// @returns A future indicating the success or failure of the operation.
  Future<void> setIgnoreCursorEvents({required final bool ignore}) async =>
      invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{
                'type': 'setIgnoreCursorEvents',
                'payload': ignore
              }
            },
          ),
        ),
      );

  /// Starts dragging the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// appWindow.startDragging();
  /// ```
  ///
  /// @return A future indicating the success or failure of the operation.
  Future<void> startDragging() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Window,
          message: TauriCommandMessage(
            cmd: 'manage',
            data: <dynamic, dynamic>{
              'label': label,
              'cmd': <dynamic, dynamic>{'type': 'startDragging'}
            },
          ),
        ),
      );

  // Listeners
  /// Listen to window resize.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onResized(
  ///   (final Event<PhysicalSize> event) {
  ///     print('Window resized: ${event.payload}');
  ///   }
  /// );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted.
  /// unlisten.then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onResized({
    required final EventCallback<PhysicalSize> handler,
  }) async =>
      listen<PhysicalSize>(
        event: const EventName(TauriDefinedEvent.WINDOW_RESIZED),
        handler: handler,
      );

  /// Listen to window move.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onMoved(
  ///   (final Event<PhysicalPosition> event) {
  ///     print('Window moved: ${event.payload}');
  ///   }
  /// );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted.
  /// unlisten.then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onMoved({
    required final EventCallback<PhysicalPosition> handler,
  }) async =>
      listen<PhysicalPosition>(
        event: const EventName(TauriDefinedEvent.WINDOW_MOVED),
        handler: handler,
      );

  /// Listen to window close requested. Emitted when the user requests to closes
  /// the window.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  /// import 'package:tauri_apps/api/dialog.dart' show confirm;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onCloseRequested(
  ///   handler: (final CloseRequestedEvent<TauriEvent> event) async {
  ///     final bool confirmed = await confirm('Are you sure?');
  ///     if (!confirmed) {
  ///       // user did not confirm closing the window; let's prevent it
  ///       event.preventDefault();
  ///     }
  ///   }
  /// );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// unlisten().then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onCloseRequested({
    required final CloseRequestedEventHandler handler,
  }) async =>
      listen<void>(
        event: const EventName(TauriDefinedEvent.WINDOW_CLOSE_REQUESTED),
        handler: (final Event<void> event) {
          final CloseRequestedEvent evt = CloseRequestedEvent(event: event);
          Future<void>(
            () async => handler(evt),
          ).then(
            (final _) {
              if (!evt.isPreventDefault) {
                close();
              }
            },
          );
        },
      );

  /// Listen to window focus change.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onFocusChanged(
  ///   handler: (final Event<bool> event) {
  ///     print('Focus changed, window is focused? ${event.payload}.');
  ///   }
  /// );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// unlisten().then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onFocusChanged({
    required final OnFocusedEventHandler handler,
  }) async {
    final UnlistenFn unlistenFocus = await listen<PhysicalPosition>(
      event: const EventName(TauriDefinedEvent.WINDOW_FOCUS),
      handler: (final Event<PhysicalPosition> event) => handler(
        Event<bool>(
          event: event.event,
          windowLabel: event.windowLabel,
          id: event.id,
          payload: true,
        ),
      ),
    );
    final UnlistenFn unlistenBlur = await listen<PhysicalPosition>(
      event: const EventName(TauriDefinedEvent.WINDOW_BLUR),
      handler: (final Event<PhysicalPosition> event) => handler(
        Event<bool>(
          event: event.event,
          windowLabel: event.windowLabel,
          id: event.id,
          payload: false,
        ),
      ),
    );
    return () {
      unlistenFocus();
      unlistenBlur();
    };
  }

  /// Listen to window scale change. Emitted when the window's scale factor has
  /// changed.
  ///
  /// The following user actions can cause DPI changes:
  ///   - Changing the display's resolution.
  ///   - Changing the display's scale factor (e.g. in Control Panel on
  ///     Windows).
  ///   - Moving the window to a display with a different scale factor.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onScaleChanged(
  ///   handler: (final Event<ScaleFactorChanged> event) {
  ///     print(
  ///       'Scale changed: ${event.payload.scaleFactor}, '
  ///       '${event.payload.size}',
  ///     );
  ///   }
  /// );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// unlisten().then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onScaleChanged({
    required final ScaleFactorChangedEventHandler handler,
  }) async =>
      listen<ScaleFactorChanged>(
        event: const EventName(TauriDefinedEvent.WINDOW_SCALE_FACTOR_CHANGED),
        handler: handler,
      );

  /// Listen to the window menu item click. The payload is the item id.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onMenuClicked(
  ///   handler: (final Event<String> event) {
  ///     print('Menu clicked: $payload.');
  ///   }
  /// );;
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// unlisten().then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onMenuClicked({
    required final MenuClickedEventHandler handler,
  }) async =>
      listen<String>(
        event: const EventName(TauriDefinedEvent.MENU),
        handler: handler,
      );

  /// Listen to a file drop event.
  ///
  /// The listener is triggered when the user hovers the selected files on the
  /// window, drops the files or cancels the operation.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onFileDropEvent(
  ///   handler: (final Event<FileDropEvent> event) {
  ///     if (event.payload.type == FileDropEventType.hover) {
  ///       print('User hovering: ${event.payload.paths}');
  ///     } else if (event.payload.type == FileDropEventType.drop) {
  ///       print('User dropped: ${event.payload.paths}');
  ///     } else {
  ///       print('File drop cancelled');
  ///     }
  ///   }
  /// );
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// unlisten().then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onFileDropEvent({
    required final EventCallback<FileDropEvent> handler,
  }) async {
    final UnlistenFn unlistenFileDrop = await listen<List<String>>(
      event: const EventName(TauriDefinedEvent.WINDOW_FILE_DROP),
      handler: (final Event<List<String>> event) => handler(
        Event<FileDropEvent>(
          event: event.event,
          windowLabel: event.windowLabel,
          id: event.id,
          payload: FileDropEvent(
            type: FileDropEventType.drop,
            paths: event.payload,
          ),
        ),
      ),
    );

    final UnlistenFn unlistenFileHover = await listen<List<String>>(
      event: const EventName(TauriDefinedEvent.WINDOW_FILE_DROP_HOVER),
      handler: (final Event<List<String>> event) => handler(
        Event<FileDropEvent>(
          event: event.event,
          windowLabel: event.windowLabel,
          id: event.id,
          payload: FileDropEvent(
            type: FileDropEventType.hover,
            paths: event.payload,
          ),
        ),
      ),
    );

    final UnlistenFn unlistenCancel = await listen<void>(
      event: const EventName(TauriDefinedEvent.WINDOW_FILE_DROP_CANCELLED),
      handler: (final Event<void> event) => handler(
        Event<FileDropEvent>(
          event: event.event,
          windowLabel: event.windowLabel,
          id: event.id,
          payload: const FileDropEvent(type: FileDropEventType.cancel),
        ),
      ),
    );

    return () {
      unlistenFileDrop();
      unlistenFileHover();
      unlistenCancel();
    };
  }

  /// Listen to the system theme change.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show appWindow;
  ///
  /// final Future<UnlistenFn> unlisten = appWindow.onThemeChanged(
  ///   handler: (final Event<Theme> event) {
  ///     print('New theme: ${event.payload}');
  ///   }
  /// );;
  ///
  /// // you need to call unlisten if your handler goes out of scope e.g. the
  /// // component is unmounted
  /// unlisten().then((final UnlistenFn fn) => fn());
  /// ```
  ///
  /// @returns A future resolving to a function to unlisten to the event.
  /// Note that removing the listener is required if your listener goes out of
  /// scope e.g. the component is unmounted.
  ///
  /// @since 1.0.2
  Future<UnlistenFn> onThemeChanged({
    required final ThemeChangedEventHandler handler,
  }) async =>
      listen<Theme>(
        event: const EventName(TauriDefinedEvent.WINDOW_THEME_CHANGED),
        handler: handler,
      );
}

/// Creates a new WebviewWindow.
///
/// ```dart
/// import 'package:tauri_apps/api/window.dart' show WebviewWindow;
///
/// final WebviewWindow webview = WebviewWindow(
///   label: 'my-label',
///   options: WindowOptions(
///     url: 'https://github.com/tauri-apps/tauri',
///   ),
/// );
///
/// webview.once<void>(
///   event: EventName('tauri://created'),
///   handler: () {// webview window successfully created},
/// );
///
/// webview.once<void>(
///   event: EventName('tauri://error'),
///   handler: () {// an error happened creating the webview window},
/// );
/// ```
///
///   * [label]: The unique webview window label. Must be alphanumeric:
///     `a-zA-Z-/:_`.
///   * [options]:
///   * [skip]:
/// @returns The WebviewWindow instance to communicate with the webview.
class WebviewWindow extends WindowManager {
  const WebviewWindow({
    required super.label,
    this.options = const WindowOptions(),
  }) : skip = false;

  const WebviewWindow._internal({
    required super.label,
    // ignore: unused_element
    this.options = const WindowOptions(),
  }) : skip = true;

  final WindowOptions options;
  final bool skip;

  /// Gets the WebviewWindow for the webview associated with the given label.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/window.dart' show WebviewWindow;
  ///
  /// final WebviewWindow mainWindow = WebviewWindow.getByLabel('main');
  /// ```
  ///
  ///   * label The webview window label.
  /// @returns The WebviewWindow instance to communicate with the webview or
  /// null if the webview doesn't exist.
  static WebviewWindow? getByLabel(final String label) {
    if (getAll().any((final WebviewWindow w) => w.label == label)) {
      return WebviewWindow._internal(label: label);
    }
    return null;
  }
}
