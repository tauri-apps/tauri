// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names, use_super_parameters

/// The event system allows you to emit events to the backend and listen to
/// events from it.
///
/// This package is also accessible with `window.__TAURI__.event` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
library tauri_event;

import 'package:universal_html/html.dart';

import 'helpers/tauri.dart';
import 'tauri.dart';
import 'window.dart';

typedef EventCallback<T> = void Function(Event<T> event);
typedef UnlistenFn = VoidCallback;

class InvalidUnionTypeException<Invalid, Valid> implements Exception {
  const InvalidUnionTypeException();

  @override
  String toString() => 'Invalid union type requested.\n'
      'Tried to get $Invalid, but the union uses $Valid.';
}

class Union<L, R> {
  const Union.left(final L left)
      : _left = left,
        _right = null;

  const Union.right(final R right)
      : _right = right,
        _left = null;

  final L? _left;
  final R? _right;

  bool get isLeft => _left != null;
  bool get isRight => _right != null;

  L get left {
    if (_left == null) {
      throw InvalidUnionTypeException<L, R>();
    }
    return _left!;
  }

  R get right {
    if (_right == null) {
      throw InvalidUnionTypeException<R, L>();
    }
    return _right!;
  }

  dynamic get deref => isLeft ? left : right;
}

abstract class TauriEventInterface {
  String get stringValue;
}

class TauriEvent extends Union<String, TauriEventInterface>
    implements TauriEventInterface {
  const TauriEvent(final TauriEventInterface event) : super.right(event);
  const TauriEvent.fromString(final String event) : super.left(event);

  @override
  String toString() => isLeft ? left : right.stringValue;

  @override
  String get stringValue => toString();
}

class Event<T> {
  const Event({
    required this.event,
    required this.windowLabel,
    required this.id,
    required this.payload,
  });

  /// Event name.
  final EventName event;

  /// The label of the window that emitted this event.
  final String windowLabel;

  /// Event identifier used to unlisten.
  final int id;

  /// Event payload.
  final T payload;
}

class EventName {
  const EventName(this.name);

  final TauriEventInterface name;

  String get nameAsString => name.stringValue;
}

/// @since 1.1.0
enum TauriDefinedEvent implements TauriEventInterface {
  WINDOW_RESIZED('tauri://resize'),
  WINDOW_MOVED('tauri://move'),
  WINDOW_CLOSE_REQUESTED('tauri://close-requested'),
  WINDOW_CREATED('tauri://window-created'),
  WINDOW_DESTROYED('tauri://destroyed'),
  WINDOW_FOCUS('tauri://focus'),
  WINDOW_BLUR('tauri://blur'),
  WINDOW_SCALE_FACTOR_CHANGED('tauri://scale-change'),
  WINDOW_THEME_CHANGED('tauri://theme-changed'),
  WINDOW_FILE_DROP('tauri://file-drop'),
  WINDOW_FILE_DROP_HOVER('tauri://file-drop-hover'),
  WINDOW_FILE_DROP_CANCELLED('tauri://file-drop-cancelled'),
  MENU('tauri://menu'),
  CHECK_UPDATE('tauri://update'),
  UPDATE_AVAILABLE('tauri://update-available'),
  INSTALL_UPDATE('tauri://update-install'),
  STATUS_UPDATE('tauri://update-status'),
  DOWNLOAD_PROGRESS('tauri://update-download-progress');

  const TauriDefinedEvent(this.urlHandler);

  final String urlHandler;

  @override
  String get stringValue => urlHandler;
}

/// Unregister the event listener associated with the given name and id.
///
///   * [event]: The event name,
///   * [eventId]: Event identifier.
Future<void> _unlisten(final String event, final int eventId) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Event,
        message: TauriCommandMessage(
          cmd: 'unlisten',
          event: event,
          eventId: eventId,
        ),
      ),
    );

/// Listen to an event from the backend.
///
/// ```dart
/// import 'package:tauri_apps/api/event.dart' show listen;
///
/// (() async {
///   final UnlistenFn unlisten = await listen<String, String>(
///     event: EventName<String>('error'),
///     handler: (final Event<String, String> event) =>
///       print(
///         'Got error in window ${event.windowLabel}, '
///         'payload: ${event.payload}',
///       ),
///     );
/// })();
///
/// // you need to call unlisten if your handler goes out of scope e.g. the
/// // component is unmounted
/// unlisten();
/// ```
///
///   * [event]: Event name. Must include only alphanumeric characters, `-`,
///     `/`, `:` and `_`,
///   * [windowLabel]: The label of the window to which the event is sent,
///     if null the event will be sent to all windows,
///   * [handler]: Event handler callback.
///
/// @returns A future resolving to a function to unlisten to the event.
/// Note that removing the listener is required if your listener goes out of
/// scope e.g. the component is unmounted.
///
/// @since 1.0.0
Future<UnlistenFn> listen<T>({
  required final EventName event,
  required final EventCallback<T> handler,
  final String? windowLabel,
}) async =>
    invokeTauriCommand<int>(
      TauriCommand(
        tauriModule: TauriModule.Event,
        message: TauriCommandMessage(
          cmd: 'listen',
          event: event,
          windowLabel: windowLabel,
          handler: transformCallback<Event<T>>(callback: handler),
        ),
      ),
    ).then(
      (final int eventId) => () async => _unlisten(event.nameAsString, eventId),
    );

/// Listen to an one-off event from the backend.
///
/// ```dart
/// import 'package:tauri_apps/api/event' show once;
///
/// mixin LoadedPayload {
///   bool? loggedIn;
///   String? token;
/// }
///
/// (() async {
///   final unlisten = await once<LoadedPayload, String>(
///     event: EventName<String>('loaded'),
///     handler: (final Event<LoadedPayload, String> event) =>
///       print(
///         'App is loaded, loggedIn: ${event.payload.loggedIn}, '
///         'token: ${event.payload.token}.',
///       ),
///   );
/// })();
///
/// // you need to call unlisten if your handler goes out of scope e.g. the
/// // component is unmounted
/// unlisten();
/// ```
///
///   * [event] Event name. Must include only alphanumeric characters, `-`,
///   `/`, `:` and `_`,
///   * [windowLabel]: The label of the window to which the event is sent,
///     if null the event will be sent to all windows,
///   * [handler]: Event handler callback.
///
/// @returns A future resolving to a function to unlisten to the event.
/// Note that removing the listener is required if your listener goes out of
/// scope e.g. the component is unmounted.
///
/// @since 1.0.0
Future<UnlistenFn> once<T>({
  required final EventName event,
  required final EventCallback<T> handler,
  final String? windowLabel,
}) async =>
    listen<T>(
      event: event,
      windowLabel: windowLabel,
      handler: (final Event<T> eventData) {
        handler(eventData);
        _unlisten(event.nameAsString, eventData.id).onError(
          (final Object obj, final StackTrace stackTrace) {},
        );
      },
    );

/// Emits an event to the backend.
///
/// ```dart
/// import 'package:tauri_apps/api/event.dart' show emit;
///
/// (() async {
///   await emit(
///     event: EventName<String>('frontend-loaded'),
///     payload: <String, dynamic>{'loggedIn': true, 'token': 'authToken'});
/// })();
/// ```
///
///   * [event]: Event name,
///     Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
///   * [windowLabel]: The label of the window to which the event is sent,
///     if null the event will be sent to all windows,
///   * [payload]: Event payload.
Future<void> emit<T>({
  required final EventName event,
  final WindowLabel? windowLabel,
  final T? payload,
}) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Event,
        message: TauriCommandMessage(
          cmd: 'emit',
          event: event.nameAsString,
          windowLabel: windowLabel,
          payload: payload,
        ),
      ),
    );
