import SwiftRs
import MetalKit
import WebKit
import os.log

class PluginManager {
    static var shared: PluginManager = PluginManager()
    var plugins: NSMutableDictionary = [:]

    func load(name: String, plugin: NSObject, webview: WKWebView) {
        plugin.perform(#selector(Plugin.load), with: webview)
        plugins[name] = plugin
    }

    func invoke(name: String, invoke: Invoke) {
        if let plugin = plugins[name] as? NSObject {
            plugin.perform(Selector(("echo:")), with: invoke)
        }
    }
}

extension PluginManager: NSCopying {

    func copy(with zone: NSZone? = nil) -> Any {
        return self
    }
}

func initInvoke() -> Invoke {
    return Invoke(sendResponse: { (success: NSDictionary?, error: NSDictionary?) -> Void in
        let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
        os_log("SENDING RESPONSE !!!!", log: log, type: .error)
    }, data: [:])
}

public func registerPlugin(name: String, plugin: NSObject, webview: WKWebView) {
    PluginManager.shared.load(
        name: name,
        plugin: plugin,
        webview: webview
    )
}

@_cdecl("invoke_plugin")
func invokePlugin(name: UnsafePointer<SRString>) {
    PluginManager.shared.invoke(name: name.pointee.to_string(), invoke: initInvoke())
}
