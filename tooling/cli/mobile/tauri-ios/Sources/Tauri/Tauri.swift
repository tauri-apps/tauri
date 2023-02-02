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

	func invoke(name: String, methodName: String, invoke: Invoke) {
		if let plugin = plugins[name] as? NSObject {
			let selectorWithThrows = Selector(("\(methodName):error:"))
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
				let selector = Selector(("\(methodName):"))
				if plugin.responds(to: selector) {
					plugin.perform(selector, with: invoke)
				} else {
					invoke.reject("No method \(methodName) found for plugin \(name)")
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
func invokePlugin(webview: WKWebView, name: UnsafePointer<SRString>, methodName: UnsafePointer<SRString>, data: NSDictionary, callback: UInt, error: UInt) {
	let invoke = Invoke(sendResponse: { (successResult: JsonValue?, errorResult: JsonValue?) -> Void in
		let (fn, payload) = errorResult == nil ? (callback, successResult) : (error, errorResult)
		var payloadJson: String
		do {
			try payloadJson = payload == nil ? "null" : payload!.jsonRepresentation() ?? "`Failed to serialize payload`"
		} catch {
			payloadJson = "`\(error)`"
		}
		webview.evaluateJavaScript("window['_\(fn)'](\(payloadJson))")
	}, data: data)
	PluginManager.shared.invoke(name: name.pointee.to_string(), methodName: methodName.pointee.to_string(), invoke: invoke)
}
