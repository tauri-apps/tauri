import SwiftRs
import MetalKit
import WebKit
import os.log
import Tauri

enum MyError: Error {
	case runtimeError(String)
}

class TauriPlugin: NSObject, Plugin {
	public override init() {
		let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
		os_log("Plugin init", log: log, type: .error)
	}

	@objc func load(webview: WKWebView) {
		let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
		os_log("Plugin load", log: log, type: .error)
	}

	@objc public func echo(_ invoke: Invoke) throws {
		// throw MyError.runtimeError("something wrong")
		invoke.resolve(.dictionary(["data": 0]))
	}
}

@_cdecl("init_plugin")
func initPlugin(webview: WKWebView?) {
	Tauri.registerPlugin(webview: webview, name: "sample", plugin: TauriPlugin())
}
