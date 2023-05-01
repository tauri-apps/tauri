// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import WebKit
import os.log

open class Plugin: NSObject {
  public let manager: PluginManager = PluginManager.shared
  public var config: JSObject = [:]
  private var listeners = [String: [Channel]]()

  internal func setConfig(_ config: JSObject) {
    self.config = config
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

  @objc func registerListener(_ invoke: Invoke) {
    guard let event = invoke.getString("event") else {
      invoke.reject("`event` not provided")
      return
    }
    guard let channel = invoke.getChannel("handler") else {
      invoke.reject("`handler` not provided")
      return
    }

    if var eventListeners = listeners[event] {
      eventListeners.append(channel)
    } else {
      listeners[event] = [channel]
    }

    invoke.resolve()
  }

  @objc func removeListener(_ invoke: Invoke) {
    guard let event = invoke.getString("event") else {
      invoke.reject("`event` not provided")
      return
    }

    if let eventListeners = listeners[event] {
      guard let channelId = invoke.getInt("channelId") else {
        invoke.reject("`channelId` not provided")
        return
      }

      listeners[event] = eventListeners.filter { $0.id != channelId }
    }

    invoke.resolve()
  }
}
