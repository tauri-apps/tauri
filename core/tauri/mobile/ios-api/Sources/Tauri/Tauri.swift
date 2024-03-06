// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import SwiftRs
import UIKit
import WebKit
import os.log

class PluginHandle {
  var instance: Plugin
  var loaded = false

  init(plugin: Plugin) {
    instance = plugin
  }
}

public class PluginManager {
  static let shared: PluginManager = PluginManager()
  public var viewController: UIViewController?
  var plugins: [String: PluginHandle] = [:]
  var ipcDispatchQueue = DispatchQueue(label: "ipc")
  public var isSimEnvironment: Bool {
    #if targetEnvironment(simulator)
      return true
    #else
      return false
    #endif
  }

  public func assetUrl(fromLocalURL url: URL?) -> URL? {
    guard let inputURL = url else {
      return nil
    }

    return URL(string: "asset://localhost")!.appendingPathComponent(inputURL.path)
  }

  func onWebviewCreated(_ webview: WKWebView) {
    for (_, handle) in plugins {
      if !handle.loaded {
        handle.instance.load(webview: webview)
      }
    }
  }

  func load<P: Plugin>(name: String, plugin: P, config: String, webview: WKWebView?) {
    plugin.setConfig(config)
    let handle = PluginHandle(plugin: plugin)
    if let webview = webview {
      handle.instance.load(webview: webview)
      handle.loaded = true
    }
    plugins[name] = handle
  }

  func invoke(name: String, invoke: Invoke) {
    if let plugin = plugins[name] {
      ipcDispatchQueue.async {
        let selectorWithThrows = Selector(("\(invoke.command):error:"))
        if plugin.instance.responds(to: selectorWithThrows) {
          var error: NSError? = nil
          withUnsafeMutablePointer(to: &error) {
            let methodIMP: IMP! = plugin.instance.method(for: selectorWithThrows)
            unsafeBitCast(
              methodIMP, to: (@convention(c) (Any?, Selector, Invoke, OpaquePointer) -> Void).self)(
                plugin.instance, selectorWithThrows, invoke, OpaquePointer($0))
          }
          if let error = error {
            invoke.reject("\(error)")
            // TODO: app crashes without this leak
            let _ = Unmanaged.passRetained(error)
          }
        } else {
          let selector = Selector(("\(invoke.command):"))
          if plugin.instance.responds(to: selector) {
            plugin.instance.perform(selector, with: invoke)
          } else {
            invoke.reject("No command \(invoke.command) found for plugin \(name)")
          }
        }
      }
    } else {
      invoke.reject("Plugin \(name) not initialized")
    }
  }
}

extension PluginManager: NSCopying {
  public func copy(with zone: NSZone? = nil) -> Any {
    return self
  }
}

@_cdecl("register_plugin")
func registerPlugin(name: SRString, plugin: NSObject, config: SRString, webview: WKWebView?) {
  PluginManager.shared.load(
    name: name.toString(),
    plugin: plugin as! Plugin,
    config: config.toString(),
    webview: webview
  )
}

@_cdecl("on_webview_created")
func onWebviewCreated(webview: WKWebView, viewController: UIViewController) {
  PluginManager.shared.viewController = viewController
  PluginManager.shared.onWebviewCreated(webview)
}

@_cdecl("run_plugin_command")
func runCommand(
  id: Int,
  name: SRString,
  command: SRString,
  data: SRString,
  callback: @escaping @convention(c) (Int, Bool, UnsafePointer<CChar>) -> Void,
  sendChannelData: @escaping @convention(c) (UInt64, UnsafePointer<CChar>) -> Void
) {
  let callbackId: UInt64 = 0
  let errorId: UInt64 = 1
  let invoke = Invoke(
    command: command.toString(), callback: callbackId, error: errorId,
    sendResponse: { (fn: UInt64, payload: String?) -> Void in
      let success = fn == callbackId
      callback(id, success, payload ?? "null")
    },
    sendChannelData: { (id: UInt64, payload: String) -> Void in
      sendChannelData(id, payload)
    }, data: data.toString())
  PluginManager.shared.invoke(name: name.toString(), invoke: invoke)
}
