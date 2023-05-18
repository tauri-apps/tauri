// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// ignore_for_file: constant_identifier_names

import 'dart:async';
import '../tauri.dart' show InvokeArgs, invoke;

enum TauriModule {
  App,
  Fs,
  Path,
  Os,
  Window,
  Shell,
  Event,
  Internal,
  Dialog,
  Cli,
  Notification,
  Http,
  GlobalShortcut,
  Process,
  Clipboard,
}

class TauriCommand {
  const TauriCommand({
    required final TauriModule tauriModule,
    required this.message,
  }) : __tauriModule = tauriModule;

  final TauriModule __tauriModule;
  final TauriCommandMessage message;

  InvokeArgs get toInvokeArgs => Map<String, dynamic>.from(
        message.toArgs,
      )..addAll(
          <String, dynamic>{
            '__tauriModule': __tauriModule.name,
          },
        );
}

Future<T> invokeTauriCommand<T>(final TauriCommand command) =>
    invoke<T>('tauri', command.toInvokeArgs);

/// Proxy to hold all possible values of a message as named parameters.
class TauriCommandMessage {
  const TauriCommandMessage({
    this.cmd,
    this.path,
    this.paths,
    this.options,
    this.contents,
    this.source,
    this.destination,
    this.oldPath,
    this.newPath,
    this.client,
    this.data,
    this.event,
    this.eventId,
    this.windowLabel,
    this.payload,
    this.handler,
    this.exitCode,
    this.shortcuts,
    this.shortcut,
    this.directory,
    this.ext,
    this.program,
    this.args,
    this.onEventFn,
    this.openWith,
    this.pid,
    this.buffer,
    this.message,
    this.title,
    this.type,
    this.buttonLabel,
    this.buttonLabels,
  });

  final dynamic cmd;
  final dynamic path;
  final dynamic paths;
  final dynamic options;
  final dynamic contents;
  final dynamic source;
  final dynamic destination;
  final dynamic oldPath;
  final dynamic newPath;
  final dynamic client;
  final dynamic data;
  final dynamic event;
  final dynamic eventId;
  final dynamic windowLabel;
  final dynamic payload;
  final dynamic handler;
  final dynamic exitCode;
  final dynamic shortcuts;
  final dynamic shortcut;
  final dynamic directory;
  final dynamic ext;
  final dynamic program;
  final dynamic args;
  final dynamic onEventFn;
  final dynamic openWith;
  final dynamic pid;
  final dynamic buffer;
  final dynamic message;
  final dynamic title;
  final dynamic type;
  final dynamic buttonLabel;
  final dynamic buttonLabels;

  InvokeArgs get toArgs => <String, dynamic>{
        'cmd': cmd,
        'path': path,
        'paths': paths,
        'options': options,
        'contents': contents,
        'source': source,
        'destination': destination,
        'oldPath': oldPath,
        'newPath': newPath,
        'client': client,
        'data': data,
        'event': event,
        'eventId': eventId,
        'windowLabel': windowLabel,
        'payload': payload,
        'handler': handler,
        'exitCode': exitCode,
        'shortcuts': shortcuts,
        'shortcut': shortcut,
        'directory': directory,
        'ext': ext,
        'program': program,
        'args': args,
        'onEventFn': onEventFn,
        'with': openWith,
        'pid': pid,
        'buffer': buffer,
        'message': message,
        'title': title,
        'type': type,
        'buttonLabel': buttonLabel,
        'buttonLabels': buttonLabels,
      }..removeWhere((final String key, final dynamic value) => value == null);
}

extension Splice<T> on List<T> {
  Iterable<T> splice(
    final int start,
    final int count, [
    final List<T>? insert,
  ]) {
    final List<T> result = <T>[
      ...getRange(start, start + count),
    ];

    replaceRange(
      start,
      start + count,
      insert ?? <T>[],
    );

    return result;
  }

  int unshift(final T element) {
    insert(0, element);
    return length;
  }
}
