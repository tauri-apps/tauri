// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

internal class InvalidCommandException : Exception {
  constructor(s: String?) : super(s) {}
  constructor(t: Throwable?) : super(t) {}
  constructor(s: String?, t: Throwable?) : super(s, t) {}
}
