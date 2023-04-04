// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names

/// Invoke your custom commands.
///
/// This package is also accessible with `window.__TAURI__.tauri` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
library tauri_base;

import 'dart:async';
import 'dart:math';
import 'dart:typed_data';

import 'package:universal_html/js.dart' as js;

import 'window.dart';

extension Int on int {
  // Presented as hex because:
  // - It is more readable than the decimal equivalent (9,007,199,254,740,991)
  // - There are no int literals separators in dart (yet)
  // - There is no exponent operator which would optimal for readability
  //   (2**53 - 1)
  /// See: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER
  static const int MAX_SAFE_INTEGER = 0x001FFFFFFFFFFFFF;
}

extension Crypto on int {
  static int secureRandomNumber([final int length = 32]) {
    final Random random = Random.secure();
    final List<int> values = List<int>.generate(
      length,
      (final int i) => random.nextInt(Int.MAX_SAFE_INTEGER),
    );

    return values.first;
  }
}

/// Signature for the callback passed to [transformCallback].
typedef TransformCallback<T> = dynamic Function(T response);

/// Command arguments.
///
/// @since 1.0.0
typedef InvokeArgs = Map<String, dynamic>;

int uid() =>
    (window.crypto?.getRandomValues(
      Uint32List(1),
    ) as Uint32List?)
        ?.first ??
    Crypto.secureRandomNumber();

/// Transforms a callback function to a string identifier that can be passed to
/// the backend.
///
/// The backend uses the identifier to `eval()` the callback.
int transformCallback<T>({
  final TransformCallback<T>? callback,
  final bool once = false,
  // pass custom id, used in invoke because there is no hoisting in dart
  final int? id,
}) {
  final int identifier = id ?? uid();
  final String prop = '''_$identifier''';

  js.context[prop] = js.JsObject.jsify(
    <String, dynamic>{
      'value': (final T result) {
        if (once) {
          js.context.deleteProperty(prop);
        }
        return callback?.call(result);
      },
      'writable': false,
      'configurable': true,
    },
  );

  return identifier;
}

/// Sends a message to the backend.
/// ```dart
/// import 'package:tauri-apps/api/tauri.dart' show invoke;
/// (() async {
///   await invoke(
///     'login',
///     <String, dynamic>{
///       'user': 'tauri',
///       'password': 'poiwe3h4r5ip3yrhtew9ty',
///     },
///   );
/// })();
/// ```
/// * [cmd]: The command name.
/// * [args]: The optional arguments to pass to the command.
/// * @return A promise resolving or rejecting to the backend response.
///
/// @since 1.0.0
Future<T> invoke<T>(
  final String cmd, [
  final InvokeArgs args = const <String, dynamic>{},
]) async =>
    Future<T>(
      () async {
        final Completer<T> promise = Completer<T>();
        final int error = uid();
        final int callback = uid();

        transformCallback<T>(
          callback: (final T e) {
            promise.complete(e);
            js.context.deleteProperty('''_$error''');
          },
          once: true,
          id: callback,
        );
        transformCallback<Object>(
          callback: (final Object e) {
            promise.completeError(e);
            js.context.deleteProperty('''_$callback''');
          },
          once: true,
          id: error,
        );

        window.tauriIpc(
          <String, dynamic>{
            'cmd': cmd,
            'callback': callback,
            'error': error,
            ...args,
          },
        );

        return promise.future;
      },
    );

/// Convert a device file path to an URL that can be loaded by the webview.
///
/// Note that `asset:` and `https://asset.localhost` must be added to
/// [`tauri.security.csp`](https://tauri.app/v1/api/config/#securityconfig.csp)
/// in `tauri.conf.json`.
///
/// Example CSP value: `"csp": "default-src 'self';
/// img-src 'self' asset: https://asset.localhost"` to use the asset protocol on
/// image sources.
///
/// Additionally, `asset` must be added to
/// [`tauri.allowlist.protocol`](https://tauri.app/v1/api/config/#allowlistconfig.protocol)
/// in `tauri.conf.json` and its access scope must be defined on the
/// `assetScope` array on the same `protocol` object.
///
/// ```dart
/// import 'package:tauri-apps/api/path.dart' show appDataDir, join;
/// import 'package:tauri-apps/api/tauri.dart' show convertFileSrc, window;
///
/// (() async {
///   final appDataDirPath = await appDataDir();
///   final filePath = await join(appDataDirPath, 'assets/video.mp4');
///   final assetUrl = convertFileSrc(filePath);
///
///   final video = window.document.getElementById('my-video');
///   final source = window.document.createElement('source');
///   source.type = 'video/mp4';
///   source.src = assetUrl;
///   video.appendChild(source);
///   video.load();
/// })();
/// ```
///
/// @since 1.0.0
String convertFileSrc(
  final String filePath, [
  final String protocol = 'asset',
]) {
  final String path = Uri.encodeComponent(filePath);

  return window.navigator.userAgent.contains('Windows')
      ? '''https://$protocol.localhost/$path'''
      : '''$protocol://localhost/$path''';
}
