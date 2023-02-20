// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import UIKit

@objc public class Invoke: NSObject, JSValueContainer, BridgedJSValueContainer {
	public var dictionaryRepresentation: NSDictionary {
		return data as NSDictionary
	}

	public static var jsDateFormatter: ISO8601DateFormatter = {
		return ISO8601DateFormatter()
	}()

  public var command: String
	public var data: JSObject
	var sendResponse: (JsonValue?, JsonValue?) -> Void

	public init(command: String, sendResponse: @escaping (JsonValue?, JsonValue?) -> Void, data: JSObject?) {
    self.command = command
		self.data = data ?? [:]
		self.sendResponse = sendResponse
	}

	public func resolve() {
		sendResponse(nil, nil)
	}

	public func resolve(_ data: JsonObject) {
		resolve(.dictionary(data))
	}

	public func resolve(_ data: JsonValue) {
		sendResponse(data, nil)
	}

	public func reject(_ message: String, _ code: String? = nil, _ error: Error? = nil, _ data: JsonValue? = nil) {
		let payload: NSMutableDictionary = ["message": message, "code": code ?? "", "error": error ?? ""]
		if let data = data {
			switch data {
			case .dictionary(let dict):
				for entry in dict {
					payload[entry.key] = entry.value
				}
			}
		}
		sendResponse(nil, .dictionary(payload as! JsonObject))
	}

	public func unimplemented() {
		unimplemented("not implemented")
	}

	public func unimplemented(_ message: String) {
		sendResponse(nil, .dictionary(["message": message]))
	}

	public func unavailable() {
		unavailable("not available")
	}

	public func unavailable(_ message: String) {
		sendResponse(nil, .dictionary(["message": message]))
	}
}
