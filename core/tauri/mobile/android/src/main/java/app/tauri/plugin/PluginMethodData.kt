// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import app.tauri.annotation.Command
import java.lang.reflect.Method

class CommandData(
  val method: Method, methodDecorator: Command
) {

  // The name of the method
  val name: String = method.name
}
