// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// The Tauri API allows you to interface with the backend layer.
///
/// This module exposes all other modules as an object where the key is the
/// module name, and the value is the module exports.
///
/// ```dart
/// import 'package:tauri_apps/api/app.dart';
/// import 'package:tauri_apps/api/dialog.dart';
/// import 'package:tauri_apps/api/event.dart';
/// import 'package:tauri_apps/api/fs.dart';
/// import 'package:tauri_apps/api/global_shortcut.dart';
/// ```
library tauri_apps;

export 'src/index.dart';
