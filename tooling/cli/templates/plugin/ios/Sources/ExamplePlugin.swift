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

@_cdecl("init_plugin_{{ plugin_name_snake_case }}")
func initPlugin() -> Plugin {
	return ExamplePlugin()
}
