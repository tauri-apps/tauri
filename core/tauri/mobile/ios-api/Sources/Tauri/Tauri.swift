import SwiftRs
import Foundation
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
			if (!handle.loaded) {
				handle.instance.load(webview: webview)
			}
		}
	}

	func load<P: Plugin>(webview: WKWebView?, name: String, plugin: P) {
		let handle = PluginHandle(plugin: plugin)
		if let webview = webview {
			handle.instance.load(webview: webview)
			handle.loaded = true
		}
		plugins[name] = handle
	}

	func invoke(name: String, methodName: String, invoke: Invoke) {
		if let plugin = plugins[name] {
			let selectorWithThrows = Selector(("\(methodName):error:"))
			if plugin.instance.responds(to: selectorWithThrows) {
				var error: NSError? = nil
				withUnsafeMutablePointer(to: &error) {
					let methodIMP: IMP! = plugin.instance.method(for: selectorWithThrows)
					unsafeBitCast(methodIMP, to: (@convention(c)(Any?, Selector, Invoke, OpaquePointer) -> Void).self)(plugin, selectorWithThrows, invoke, OpaquePointer($0))
				}
				if let error = error {
					invoke.reject("\(error)")
					let _ = toRust(error) // TODO app is crashing without this memory leak (when an error is thrown)
				}
			} else {
				let selector = Selector(("\(methodName):"))
				if plugin.instance.responds(to: selector) {
					plugin.instance.perform(selector, with: invoke)
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
	public func copy(with zone: NSZone? = nil) -> Any {
		return self
	}
}

public func registerPlugin<P: Plugin>(webview: WKWebView?, name: String, plugin: P) {
	PluginManager.shared.load(
		webview: webview,
		name: name,
		plugin: plugin
	)
}

@_cdecl("on_webview_created")
func onWebviewCreated(webview: WKWebView, viewController: UIViewController) {
	PluginManager.shared.viewController = viewController
	PluginManager.shared.onWebviewCreated(webview)
}

@_cdecl("post_ipc_message")
func postIpcMessage(webview: WKWebView, name: UnsafePointer<SRString>, methodName: UnsafePointer<SRString>, data: NSDictionary, callback: UInt, error: UInt) {
	let invoke = Invoke(sendResponse: { (successResult: JsonValue?, errorResult: JsonValue?) -> Void in
		let (fn, payload) = errorResult == nil ? (callback, successResult) : (error, errorResult)
		var payloadJson: String
		do {
			try payloadJson = payload == nil ? "null" : payload!.jsonRepresentation() ?? "`Failed to serialize payload`"
		} catch {
			payloadJson = "`\(error)`"
		}
		webview.evaluateJavaScript("window['_\(fn)'](\(payloadJson))")
	}, data: JSTypes.coerceDictionaryToJSObject(data, formattingDatesAsStrings: true))
	PluginManager.shared.invoke(name: name.pointee.to_string(), methodName: methodName.pointee.to_string(), invoke: invoke)
}

@_cdecl("run_plugin_method")
func runPluginMethod(
	id: Int,
	name: UnsafePointer<SRString>,
	methodName: UnsafePointer<SRString>,
	data: NSDictionary,
	callback: @escaping @convention(c) (Int, Bool, UnsafePointer<CChar>?) -> Void
) {
	let invoke = Invoke(sendResponse: { (successResult: JsonValue?, errorResult: JsonValue?) -> Void in
		let (success, payload) = errorResult == nil ? (true, successResult) : (false, errorResult)
		var payloadJson: String = ""
		do {
			try payloadJson = payload == nil ? "null" : payload!.jsonRepresentation() ?? "`Failed to serialize payload`"
		} catch {
			payloadJson = "`\(error)`"
		}
		callback(id, success, payloadJson.cString(using: String.Encoding.utf8))
	}, data: JSTypes.coerceDictionaryToJSObject(data, formattingDatesAsStrings: true))
	PluginManager.shared.invoke(name: name.pointee.to_string(), methodName: methodName.pointee.to_string(), invoke: invoke)
}
