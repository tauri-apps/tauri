// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import WebKit

class PingArgs: Decodable {
  let value: String?
  let onEvent: Channel?
}

class ExamplePlugin: Plugin {
  @objc public func ping(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(PingArgs.self)
    try args.onEvent?.send(["kind": "ping"])
    invoke.resolve(["value": args.value ?? ""])
  }

  @objc public func jsValues(_ invoke: Invoke) throws {
    let array: JSArray = ["a", 1, true]
    let obj: JSObject = ["kind": "object", "nested": [1, 2] as JSArray]
    invoke.resolve(["array": array, "obj": obj])
  }
}

@_cdecl("init_plugin_sample")
func initPlugin() -> Plugin {
  return ExamplePlugin()
}
