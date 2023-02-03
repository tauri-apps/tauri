import MetalKit
import WebKit
import Tauri

class ExamplePlugin: NSObject, Plugin {
	@objc func load(webview: WKWebView) {}

	@objc public func ping(_ invoke: Invoke) throws {
		let value = invoke.getString("value")
		invoke.resolve(.dictionary(["value": value as Any]))
	}
}

@_cdecl("init_plugin_{{ plugin_name_snake_case }}")
func initPlugin(webview: WKWebView?) {
	Tauri.registerPlugin(webview: webview, name: "{{plugin_name}}", plugin: ExamplePlugin())
}
