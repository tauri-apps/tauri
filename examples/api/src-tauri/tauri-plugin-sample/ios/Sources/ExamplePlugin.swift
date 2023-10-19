// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import WebKit

class ExamplePlugin: Plugin {
  @objc public func ping(_ invoke: Invoke) throws {
    let onEvent = invoke.getChannel("onEvent")
    onEvent?.send(["kind": "ping"])

    let value = invoke.getString("value")
    invoke.resolve(["value": value as Any])
  }
}

@_cdecl("init_plugin_sample")
func initPlugin() -> Plugin {
  return ExamplePlugin()
}
