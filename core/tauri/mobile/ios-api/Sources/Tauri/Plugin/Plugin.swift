// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import WebKit
import os.log

open class Plugin: NSObject {
    public let manager: PluginManager = PluginManager.shared
    public var config: JSObject = [:]

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
}
