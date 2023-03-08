// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import app.tauri.annotation.PluginMethod
import java.lang.reflect.Method

class PluginMethodData(
  val method: Method, methodDecorator: PluginMethod
) {

  // The name of the method
  val name: String = method.name
}
