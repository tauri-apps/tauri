// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Native system dialogs for opening and saving files.
///
/// This package is also accessible with `window.__TAURI__.dialog` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.dialog`](https://tauri.app/v1/api/config/#allowlistconfig.dialog)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "dialog": {
///         "all": true, // enable all dialog APIs
///         "ask": true, // enable dialog ask API
///         "confirm": true, // enable dialog confirm API
///         "message": true, // enable dialog message API
///         "open": true, // enable file open API
///         "save": true // enable file save API
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_dialog;

import './helpers/tauri.dart';
import 'event.dart';

/// Extension filters for the file dialog.
///
/// @since 1.0.0
class DialogFilter {
  const DialogFilter({
    required this.name,
    required this.extensions,
  });

  /// Filter name
  final String name;

  /// Extensions to filter, without a `.` prefix.
  ///
  /// ```dart
  /// extensions: ['svg', 'png']
  /// ```
  final List<String> extensions;
}

/// Options for the open dialog.
///
/// @since 1.0.0
class OpenDialogOptions {
  const OpenDialogOptions({
    this.title,
    this.filters,
    this.defaultPath,
    this.multiple,
    this.directory,
    this.recursive,
  });

  /// The title of the dialog window
  final String? title;

  /// The filters of the dialog
  final List<DialogFilter>? filters;

  /// Initial directory or file path
  final String? defaultPath;

  /// Whether the dialog allows multiple selection or not
  final bool? multiple;

  /// Whether the dialog is a directory selection or not
  final bool? directory;

  /// If `directory` is true, indicates that it will be read recursively later.
  /// Defines whether subdirectories will be allowed on the scope or not.
  final bool? recursive;
}

/// Options for the save dialog.
///
/// @since 1.0.0
class SaveDialogOptions {
  const SaveDialogOptions({
    this.title,
    this.filters,
    this.defaultPath,
  });

  /// The title of the dialog window
  final String? title;

  /// The filters of the dialog
  final List<DialogFilter>? filters;

  /// Initial directory or file path.
  /// If it's a directory path, the dialog class will change to that folder.
  /// const will();
  ///
  /// If it's not an existing directory, the file name will be set to the
  /// dialog's file name input and the dialog will be set to the parent folder.
  final String? defaultPath;
}

enum DialogType {
  info,
  warning,
  error;
}

/// @since 1.0.0
class MessageDialogOptions {
  const MessageDialogOptions({
    this.title,
    this.okLabel,
    this.type = DialogType.info,
  });

  /// The title of the dialog. Defaults to the app name
  final String? title;

  /// The type of the dialog. Defaults to `info`
  final DialogType type;

  /// The label of the confirm button
  final String? okLabel;
}

class ConfirmDialogOptions {
  const ConfirmDialogOptions({
    this.title,
    this.okLabel,
    this.cancelLabel,
    this.type = DialogType.info,
  });

  /// The title of the dialog. Defaults to the app name
  final String? title;

  /// The type of the dialog. Defaults to `info`
  final DialogType type;

  /// The label of the confirm button
  final String? okLabel;

  /// The label of the cancel button
  final String? cancelLabel;
}

const Union<String, ConfirmDialogOptions> defaultConfirmDialogOpts =
    Union<String, ConfirmDialogOptions>.right(ConfirmDialogOptions());

/// Open a file/directory selection dialog.
///
/// The selected paths are added to the filesystem and asset protocol allowlist
/// scopes.
///
/// When security is more important than the easy of use of this API,
/// prefer writing a dedicated command instead.
///
/// Note that the allowlist scope change is not persisted, so the values are
/// cleared when the application is restarted.
///
/// You can save it to the filesystem using
/// [tauri-plugin-persisted-scope](https://github.com/tauri-apps/tauri-plugin-persisted-scope).
///
/// ```dart
/// import 'package:tauri_apps/api/dialog.dart' show open;
/// // Open a selection dialog for image files
/// open(
///   options: OpenDialogOptions(
///     multiple: true,
///     filters: [
///       DialogFilter(name: 'Image', extensions: <String>['png', 'jpeg']),
///     ],
///   ),
/// ).then(
///   (final Union<String, List<String>>? selected) {
///     if (selected == null) {
///       // user cancelled the selection
///     } else if (selected.isRight) {
///       // user selected multiple files
///     } else {
///       // user selected a single file
///     }
///   },
/// );
/// ```
///
/// ```dart
/// import 'package:tauri_apps/api/dialog.dart' show openDialog;
/// import 'package:tauri_apps/api/path.dart' show appDir;
///
/// Future<void> open() async {
///   // Open a selection dialog for directories
///   final Union<String, List<String>>? selected = await openDialog(
///     options: OpenDialogOptions(
///       directory: true,
///       multiple: true,
///       defaultPath: await appDir(),
///     ),
///   );
///
///   if (selected == null) {
///     // user cancelled the selection
///   } else if (selected.isRight) {
///     // user selected multiple directories
///   } else {
///     // user selected a single directory
///   }
/// }
/// ```
///
/// @returns A future resolving to the selected path(s)
///
/// @since 1.0.0
Future<Union<String, List<String>>?> openDialog({
  final OpenDialogOptions options = const OpenDialogOptions(),
}) async {
  final dynamic res = invokeTauriCommand<dynamic>(
    TauriCommand(
      tauriModule: TauriModule.Dialog,
      message: TauriCommandMessage(
        cmd: 'openDialog',
        options: options,
      ),
    ),
  );

  if (res is String) {
    return Union<String, List<String>>.left(res);
  }

  if (res is List<String>) {
    return Union<String, List<String>>.right(res);
  }

  return null;
}

/// Open a file/directory save dialog.
///
/// The selected path is added to the filesystem and asset protocol allowlist
/// scopes.
///
/// When security is more important than the easy of use of this API,
/// prefer writing a dedicated command instead.
///
/// Note that the allowlist scope change is not persisted, so the values are
/// cleared when the application is restarted.
///
/// You can save it to the filesystem using
/// [tauri-plugin-persisted-scope](https://github.com/tauri-apps/tauri-plugin-persisted-scope).
///
/// ```dart
/// import 'package:tauri_apps/api/dialog.dart' show save;
///
/// save(
///   options: SaveDialogOptions(
///     filters: [
///       DialogFilter(name: 'Image', extensions: ['png', 'jpeg']),
///     ],
///   ),
/// ).then((final String? filePath) => print(filePath));
/// ```
///
/// @returns A future resolving to the selected path.
///
/// @since 1.0.0
Future<String?> save({
  final SaveDialogOptions options = const SaveDialogOptions(),
}) async =>
    invokeTauriCommand<String?>(
      TauriCommand(
        tauriModule: TauriModule.Dialog,
        message: TauriCommandMessage(
          cmd: 'saveDialog',
          options: options,
        ),
      ),
    );

/// Shows a message dialog with an `Ok` button.
///
/// ```dart
/// import 'package:tauri_apps/api/dialog.dart' show message;
///
/// message(
///   'Tauri is awesome',
///   Union<String, List<String>>.left('Tauri'),
/// );
/// message(
///   message: 'File not found',
///   options: MessageDialogOptions(
///     title: 'Tauri',
///     type: DialogType.error,
///   ),
/// );
/// ```
///
///   * [message]: The message to show.
///   * [options]: The dialog's options. If a string, it represents the dialog
///     title.
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> message({
  required final String message,
  final Union<String, MessageDialogOptions> options =
      const Union<String, MessageDialogOptions>.right(MessageDialogOptions()),
}) async {
  final MessageDialogOptions opts = options.isRight
      ? options.right
      : MessageDialogOptions(title: options.left);

  return invokeTauriCommand<void>(
    TauriCommand(
      tauriModule: TauriModule.Dialog,
      message: TauriCommandMessage(
        cmd: 'messageDialog',
        message: message,
        title: opts.title,
        type: opts.type,
        buttonLabel: opts.okLabel,
      ),
    ),
  );
}

/// Shows a question dialog with `Yes` and `No` buttons.
///
/// ```dart
/// import 'package:tauri_apps/api/dialog.dart' show ask;
///
/// ask(
///   message: 'Are you sure?',
///   options: ConfirmDialogOptions(title: 'Tauri'),
/// ).then((final bool answer) => print(answer));
/// ask(
///   message: 'This action cannot be reverted. Are you sure?',
///   options: ConfirmDialogOptions(
///     title: 'Tauri',
///     type: DialogType.warning,
///   ),
/// ).then((final bool answer) => print(answer));
/// ```
///
///   * [message]: The message to show.
///   * [options]: The dialog's options. If a string, it represents the dialog
///     title.
///
/// @returns A future resolving to a bool indicating whether `Yes` was clicked
/// or not.
///
/// @since 1.0.0
Future<bool> ask({
  required final String message,
  final Union<String, ConfirmDialogOptions> options = defaultConfirmDialogOpts,
}) async {
  final ConfirmDialogOptions opts = ConfirmDialogOptions(
    title: options.isRight ? options.right.title : options.left,
    type: options.isRight ? options.right.type : DialogType.info,
    okLabel: options.isRight ? options.right.okLabel ?? 'Yes' : 'Yes',
    cancelLabel: options.isRight ? options.right.cancelLabel ?? 'No' : 'No',
  );

  return invokeTauriCommand(
    TauriCommand(
      tauriModule: TauriModule.Dialog,
      message: TauriCommandMessage(
        cmd: 'askDialog',
        message: message,
        title: opts.title,
        type: opts.type,
        buttonLabels: <String>[opts.okLabel!, opts.cancelLabel!],
      ),
    ),
  );
}

/// Shows a question dialog with `Ok` and `Cancel` buttons.
///
/// ```dart
/// import 'package:tauri_apps/api/dialog.dart' show confirm;
///
/// confirm(
///   message: 'Are you sure?',
///   options: ConfirmDialogOptions(title: 'Tauri'),
/// ).then((final bool answer) => print(answer));
///
/// confirm(
///   message: 'This action cannot be reverted. Are you sure?',
///   options: ConfirmDialogOptions(title: 'Tauri', type: DialogOptions.warning),
/// ).then((final bool answer) => print(answer));
/// ```
///
///   * [message]: The message to show.
///   * [options]: The dialog's options. If a string, it represents the dialog
///     title.
///
/// @returns A future resolving to a bool indicating whether `Ok` was clicked or
/// not.
///
/// @since 1.0.0
Future<bool> confirm({
  required final String message,
  final Union<String, ConfirmDialogOptions> options = defaultConfirmDialogOpts,
}) async {
  final ConfirmDialogOptions opts = ConfirmDialogOptions(
    title: options.isRight ? options.right.title : options.left,
    type: options.isRight ? options.right.type : DialogType.info,
    okLabel: options.isRight ? options.right.okLabel ?? 'Yes' : 'Yes',
    cancelLabel:
        options.isRight ? options.right.cancelLabel ?? 'Cancel' : 'Cancel',
  );

  return invokeTauriCommand(
    TauriCommand(
      tauriModule: TauriModule.Dialog,
      message: TauriCommandMessage(
        cmd: 'confirmDialog',
        message: message,
        title: opts.title,
        type: opts.type,
        buttonLabels: <String>[opts.okLabel!, opts.cancelLabel!],
      ),
    ),
  );
}
