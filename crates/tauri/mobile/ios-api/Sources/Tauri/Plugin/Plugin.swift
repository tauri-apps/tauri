// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import WebKit
import os.log

struct RegisterListenerArgs: Decodable {
  let event: String
  let handler: Channel
}

struct RemoveListenerArgs: Decodable {
  let event: String
  let channelId: UInt64
}

open class Plugin: NSObject {
  public let manager: PluginManager = PluginManager.shared
  var config: String = "{}"
  private var listeners = [String: [Channel]]()

  internal func setConfig(_ config: String) {
    self.config = config
  }

  public func parseConfig<T: Decodable>(_ type: T.Type) throws -> T {
    let jsonData = self.config.data(using: .utf8)!
    let decoder = JSONDecoder()
    return try decoder.decode(type, from: jsonData)
  }

  @objc open func load(webview: WKWebView) {}

  @objc open func checkPermissions(_ invoke: Invoke) {
    invoke.resolve()
  }

  @objc open func requestPermissions(_ invoke: Invoke) {
    invoke.resolve()
  }

  public func trigger(_ event: String, data: JSObject) {
    if let eventListeners = listeners[event] {
      for channel in eventListeners {
        channel.send(data)
      }
    }
  }

  public func trigger<T: Encodable>(_ event: String, data: T) throws {
    if let eventListeners = listeners[event] {
      for channel in eventListeners {
        try channel.send(data)
      }
    }
  }

  @objc func registerListener(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(RegisterListenerArgs.self)

    if var eventListeners = listeners[args.event] {
      eventListeners.append(args.handler)
    } else {
      listeners[args.event] = [args.handler]
    }

    invoke.resolve()
  }

  @objc func removeListener(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(RemoveListenerArgs.self)

    if let eventListeners = listeners[args.event] {

      listeners[args.event] = eventListeners.filter { $0.id != args.channelId }
    }

    invoke.resolve()
  }
}
