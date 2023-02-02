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
		let method = "echo"
		if let plugin = plugins[name] as? NSObject {
			let selectorWithThrows = Selector(("\(method):error:"))
			if plugin.responds(to: selectorWithThrows) {
				var error: NSError? = nil
				withUnsafeMutablePointer(to: &error) {
					let methodIMP: IMP! = plugin.method(for: selectorWithThrows)
					unsafeBitCast(methodIMP, to: (@convention(c)(Any?, Selector, Invoke, OpaquePointer) -> Void).self)(plugin, selectorWithThrows, invoke, OpaquePointer($0))
				}
				if let error = error {
					invoke.reject("\(error)")
                    toRust(error) // TODO app is crashing without this memory leak
				}
			} else {
				let selector = Selector(("\(method):"))
				if plugin.responds(to: selector) {
					plugin.perform(selector, with: invoke)
				} else {
					invoke.reject("No method \(method) found for plugin \(name)")
				}
			}
		} else {
			invoke.reject("Plugin \(name) not initialized")
		}
	}
}

extension PluginManager: NSCopying {
	func copy(with zone: NSZone? = nil) -> Any {
		return self
	}
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
    let invoke = Invoke(sendResponse: { (success: NSDictionary?, error: NSDictionary?) -> Void in
		let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
		os_log("SENDING RESPONSE %{public}@ %{public}@ !!!!", log: log, type: .error, "\(success)", "\(error)")
	}, data: [:])
	PluginManager.shared.invoke(name: name.pointee.to_string(), invoke: invoke)
}
