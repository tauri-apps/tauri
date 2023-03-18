// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit
import WebKit
import Tauri
import SwiftRs

class ExamplePlugin: Plugin {
	@objc public func ping(_ invoke: Invoke) throws {
		let value = invoke.getString("value")
		invoke.resolve(["value": value as Any])
	}
}

@_cdecl("init_plugin_sample")
func initPlugin(name: SRString, webview: WKWebView?) {
	Tauri.registerPlugin(webview: webview, name: name.toString(), plugin: ExamplePlugin())
}
