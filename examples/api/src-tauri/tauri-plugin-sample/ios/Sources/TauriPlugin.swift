import SwiftRs
import MetalKit
import WebKit
import os.log
import Tauri

class TauriPlugin: NSObject, Plugin {
    public init(webview: WKWebView) {
        let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
        os_log("Plugin load %{public}@ !!!!", log: log, type: .error, webview.url!.absoluteString)
    }

    public func echo(invoke: Invoke) {
        invoke.resolve()
    }
}

@_cdecl("init_plugin")
func initPlugin(webview: WKWebView, invoke: Invoke) -> TauriPlugin {
    let plugin = TauriPlugin(webview: webview)
    plugin.echo(invoke: invoke)
    return toRust(plugin)
}
