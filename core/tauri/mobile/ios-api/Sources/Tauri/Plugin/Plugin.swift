import WebKit
import os.log

open class Plugin: NSObject {
    public let manager: PluginManager = PluginManager.shared

    @objc open func load(webview: WKWebView) {}

    @objc open func checkPermissions(_ invoke: Invoke) {
        invoke.resolve()
    }

    @objc open func requestPermissions(_ invoke: Invoke) {
        invoke.resolve()
    }
}
