// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
library tauri_mocks;

import 'dart:async';

import 'package:universal_html/js.dart' as js;
import 'window.dart';

class IPCMessage {
  const IPCMessage({
    required this.cmd,
    required this.callback,
    required this.error,
    required this.args,
  });

  final String cmd;
  final int callback;
  final int error;
  final Map<String, dynamic> args;
}

/// Intercepts all IPC requests with the given mock handler.
/// @since 1.0.0
void mockIPC(
  final FutureOr<dynamic> Function(String cmd, Map<String, dynamic> args) cb,
) {
  window.tauriIpc = (final dynamic message) async {
    final IPCMessage msg = message as IPCMessage;

    try {
      js.context.callMethod('_${msg.callback}', await cb(msg.cmd, msg.args));
    } on Exception catch (err) {
      js.context.callMethod('_${msg.error}', <dynamic>[err]);
    }
  };
}

/// @since 1.0.0
void mockWindows(
  final String current,
  final List<String> additionalWindows,
) {
  window.tauriMetadata = TauriMetadataProxy(
    windows: <String>[current, ...additionalWindows]
        .map((final String label) => WindowDef(label: label))
        .toList(),
    currentWindow: WindowDef(label: current),
  );
}

/// @since 1.0.0
void clearMocks() {
  window
    ..removeProperty(TauriWindowDefinition.tauriIpc)
    ..removeProperty(TauriWindowDefinition.tauriMetadata);
}
