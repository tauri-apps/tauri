// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

class Channel(val id: Long, private val handler: (data: JSObject) -> Unit) {
  fun send(data: JSObject) {
    handler(data)
  }
}
