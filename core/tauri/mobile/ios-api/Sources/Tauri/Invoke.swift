import Foundation
import MetalKit

@objc public class Invoke: NSObject, JSValueContainer, BridgedJSValueContainer {
	public var dictionaryRepresentation: NSDictionary {
		return data as NSDictionary
	}

	public static var jsDateFormatter: ISO8601DateFormatter = {
		return ISO8601DateFormatter()
	}()

	var sendResponse: (JsonValue?, JsonValue?) -> Void
	public var data: JSObject

	public init(sendResponse: @escaping (JsonValue?, JsonValue?) -> Void, data: JSObject?) {
		self.sendResponse = sendResponse
		self.data = data ?? [:]
	}

	public func resolve(_ data: JsonValue? = nil) {
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
