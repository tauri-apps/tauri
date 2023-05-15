// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import UIKit

let CHANNEL_PREFIX = "__CHANNEL__:"

@objc public class Invoke: NSObject, JSValueContainer, BridgedJSValueContainer {
	public var dictionaryRepresentation: NSDictionary {
		return data as NSDictionary
	}

	public static var jsDateFormatter: ISO8601DateFormatter = {
		return ISO8601DateFormatter()
	}()

  public var command: String
  var callback: UInt64
  var error: UInt64
	public var data: JSObject
	var sendResponse: (UInt64, JsonValue?) -> Void

	public init(command: String, callback: UInt64, error: UInt64, sendResponse: @escaping (UInt64, JsonValue?) -> Void, data: JSObject?) {
    self.command = command
    self.callback = callback
    self.error = error
		self.data = data ?? [:]
		self.sendResponse = sendResponse
	}

	public func resolve() {
		sendResponse(callback, nil)
	}

	public func resolve(_ data: JsonObject) {
		resolve(.dictionary(data))
	}

	public func resolve(_ data: JsonValue) {
		sendResponse(callback, data)
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
		sendResponse(self.error, .dictionary(payload as! JsonObject))
	}

	public func unimplemented() {
		unimplemented("not implemented")
	}

	public func unimplemented(_ message: String) {
		sendResponse(error, .dictionary(["message": message]))
	}

	public func unavailable() {
		unavailable("not available")
	}

	public func unavailable(_ message: String) {
		sendResponse(error, .dictionary(["message": message]))
	}

  public func getChannel(_ key: String) -> Channel? {
    let channelDef = getString(key, "")
    guard let callback = UInt64(channelDef.components(separatedBy: CHANNEL_PREFIX)[1]) else {
      return nil
    }
    return Channel(callback: callback, handler: { (res: JsonValue) -> Void in
      self.sendResponse(callback, res)
    })
  }
}
