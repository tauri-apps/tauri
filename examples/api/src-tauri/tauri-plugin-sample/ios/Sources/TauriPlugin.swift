import SwiftRs
import MetalKit
import WebKit
import os.log
import Tauri

class TauriPlugin: NSObject, Plugin {
    public override init() {
        let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
        os_log("Plugin init", log: log, type: .error)
    }

    @objc func load(webview: WKWebView) { }

    @objc public func echo(_ invoke: Invoke) {
        invoke.resolve()
    }
}

@_cdecl("init_plugin")
func initPlugin(webview: WKWebView) {
    Tauri.registerPlugin(name: "sample", plugin: TauriPlugin(), webview: webview)
}
