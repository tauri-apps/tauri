import Foundation
import MetalKit

@objc public class Invoke: NSObject, JSValueContainer, BridgedJSValueContainer {
    public var jsObjectRepresentation: JSObject {
        return data as? JSObject ?? [:]
    }

    public var dictionaryRepresentation: NSDictionary {
        return data as NSDictionary
    }

    public static var jsDateFormatter: ISO8601DateFormatter = {
        return ISO8601DateFormatter()
    }()

    var sendResponse: (NSDictionary?, NSDictionary?) -> Void
    var data: NSDictionary

    public init(sendResponse: @escaping (NSDictionary?, NSDictionary?) -> Void, data: NSDictionary) {
      self.sendResponse = sendResponse
      self.data = data
    }

    public func resolve() {
        sendResponse(nil, nil)
    }

    public func resolve(_ data: NSDictionary = [:]) {
        sendResponse(data, nil)
    }

    public func reject(_ message: String, _ code: String? = nil, _ error: Error? = nil, _ data: NSDictionary? = nil) {
        let payload: NSMutableDictionary = ["message": message, "code": code ?? "", "error": error ?? ""]
        if let data = data {
            for entry in data {
                payload[entry.key] = entry.value
            }
        }
        sendResponse(nil, payload)
    }

    public func unimplemented() {
        unimplemented("not implemented")
    }

    public func unimplemented(_ message: String) {
        sendResponse(nil, ["message": message])
    }

    public func unavailable() {
        unavailable("not available")
    }

    public func unavailable(_ message: String) {
        sendResponse(nil, ["message": message])
    }
}
