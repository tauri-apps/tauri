// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names, only_throw_errors

/// Customize the auto updater flow.
///
/// This package is also accessible with `window.__TAURI__.updater` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
library tauri_updater;

import 'dart:async';

import './event.dart';

/// Signature for the callback passed as a hanlder of the [onUpdaterEvent]
/// listener.
typedef UpdateEventHandlerCallback = void Function(UpdateStatusResult? status);

/// @since 1.0.0
enum UpdateStatus {
  PENDING,
  ERROR,
  DONE,
  UPTODATE,
}

/// @since 1.0.0
class UpdateStatusResult {
  const UpdateStatusResult({required this.status, this.error});

  final String? error;
  final UpdateStatus status;
}

/// @since 1.0.0
class UpdateManifest {
  const UpdateManifest({
    required this.version,
    required this.date,
    required this.body,
  });

  final String version;
  final String date;
  final String body;
}

/// @since 1.0.0
class UpdateResult {
  const UpdateResult({required this.shouldUpdate, this.manifest});

  final UpdateManifest? manifest;
  final bool shouldUpdate;
}

/// Listen to an updater event.
///
/// ```dart
/// import 'package:tauri_apps/api/updater.dart' show onUpdaterEvent;
///
/// final Future<UnlistenFn> unlisten = onUpdaterEvent(
///   handler: (final UpdateStatusResult? result) => print(
///     'Updater event ${result?.error} ${result?.status}.',
///   ),
/// );
///
/// // you need to call unlisten if your handler goes out of scope e.g. the
/// // component is unmounted
/// final Future<void> f = unlisten.then((final UnlistenFn fn) => fn());
///
/// ```
///
/// @returns A future resolving to a function to unlisten to the event.
/// Note that removing the listener is required if your listener goes out of
/// scope e.g. the component is unmounted.
///
/// @since 1.0.2
Future<UnlistenFn> onUpdaterEvent({
  required final UpdateEventHandlerCallback handler,
}) async =>
    listen<UpdateStatusResult>(
      event: const EventName(TauriDefinedEvent.STATUS_UPDATE),
      handler: (final Event<UpdateStatusResult>? data) =>
          handler(data?.payload),
    );

/// Install the update if there's one available.
///
/// ```dart
/// import 'package:tauri_apps/api/updater.dart'
///     show checkUpdate, installUpdate;
///
/// final Future<void> f = checkUpdate().then(
///   (final UpdateResult update) async {
///     if (update.shouldUpdate) {
///       print(
///         'Installing update ${update.manifest?.version}, '
///         '${update.manifest?.date}, ${update.manifest?.body}.',
///       );
///       await installUpdate();
///     }
///   },
/// );
/// ```
///
/// @return A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> installUpdate() async {
  final Completer<void> promise = Completer<void>();
  UnlistenFn? unlistenerFn;

  void cleanListener() {
    unlistenerFn?.call();
    unlistenerFn = null;
  }

  return Future<void>(() async {
    void onStatusChange(final UpdateStatusResult? statusResult) {
      if (statusResult?.error != null && statusResult!.error!.isNotEmpty) {
        cleanListener();
        promise.completeError(
          (final Object err, final StackTrace trace) => statusResult.error,
        );
        return;
      }

      // install complete
      if (statusResult?.status == UpdateStatus.DONE) {
        cleanListener();
        promise.complete();
      }
    }

    // listen status change
    await onUpdaterEvent(handler: onStatusChange)
        .then((final UnlistenFn fn) => unlistenerFn = fn)
        .onError(
      (final Object e, final StackTrace trace) {
        cleanListener();
        // dispatch the error to our checkUpdate
        throw e;
      },
    );

    // start the process we dont require much security as it's
    // handled by rust
    await emit<TauriDefinedEvent>(
      event: const EventName(TauriDefinedEvent.INSTALL_UPDATE),
    ).onError(
      (final Object e, final StackTrace trace) {
        cleanListener();
        // dispatch the error to our checkUpdate
        throw e;
      },
    );

    return promise.future;
  });
}

/// Checks if an update is available.
///
/// ```dart
/// import 'package:tauri_apps/api/updater.dart' show checkUpdate;
///
/// final Future<UpdateResult> update = checkUpdate();
///
/// // now run installUpdate() if needed
/// ```
///
/// @return Future resolving to the update status.
///
/// @since 1.0.0
Future<UpdateResult> checkUpdate() async {
  final Completer<UpdateResult> promise = Completer<UpdateResult>();
  UnlistenFn? unlistenerFn;

  void cleanListener() {
    unlistenerFn?.call();
    unlistenerFn = null;
  }

  return Future<UpdateResult>(
    () async {
      void onUpdateAvailable(final UpdateManifest manifest) {
        cleanListener();
        promise.complete(UpdateResult(shouldUpdate: true, manifest: manifest));
      }

      void onStatusChange(final UpdateStatusResult? statusResult) {
        if (statusResult?.error != null && statusResult!.error!.isNotEmpty) {
          cleanListener();
          promise.completeError(
            (final Object err, final StackTrace trace) => statusResult.error,
          );
          return;
        }

        if (statusResult?.status == UpdateStatus.UPTODATE) {
          cleanListener();
          promise.complete(const UpdateResult(shouldUpdate: false));
        }
      }

      // wait to receive the latest update
      await once<UpdateManifest>(
        event: const EventName(TauriDefinedEvent.UPDATE_AVAILABLE),
        handler: (final Event<UpdateManifest> data) =>
            onUpdateAvailable(data.payload),
      ).onError(
        (final Object e, final StackTrace trace) {
          cleanListener();
          // dispatch the error to our checkUpdate
          throw e;
        },
      );

      // listen status change
      await onUpdaterEvent(handler: onStatusChange)
          .then((final UnlistenFn fn) => unlistenerFn = fn)
          .onError((final Object e, final StackTrace trace) {
        cleanListener();
        // dispatch the error to our checkUpdate
        throw e;
      });

      // start the process
      await emit<TauriDefinedEvent>(
        event: const EventName(TauriDefinedEvent.CHECK_UPDATE),
      ).onError(
        (final Object e, final StackTrace trace) {
          cleanListener();
          // dispatch the error to our checkUpdate
          throw e;
        },
      );

      return promise.future;
    },
  );
}
