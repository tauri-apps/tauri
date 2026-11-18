// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import UIKit

@objc public class Invoke: NSObject {
  public let command: String
  let callback: UInt64
  let error: UInt64
  let data: String
  let sendResponse: (UInt64, String?) -> Void
  let sendChannelData: (UInt64, String) -> Void
  /// The view controller hosting the webview that originated this call.
  /// Nil for non-contextual calls or after the view controller is deallocated.
  public private(set) weak var viewController: UIViewController?
  /// Whether the call was made on behalf of a specific webview.
  /// A contextual call must not fall back to another view controller when `viewController` is unusable.
  public let isContextual: Bool

  public convenience init(
    command: String, callback: UInt64, error: UInt64,
    sendResponse: @escaping (UInt64, String?) -> Void,
    sendChannelData: @escaping (UInt64, String) -> Void, data: String
  ) {
    self.init(
      command: command, callback: callback, error: error, sendResponse: sendResponse,
      sendChannelData: sendChannelData, data: data, viewController: nil, isContextual: false)
  }

  init(
    command: String, callback: UInt64, error: UInt64,
    sendResponse: @escaping (UInt64, String?) -> Void,
    sendChannelData: @escaping (UInt64, String) -> Void, data: String,
    viewController: UIViewController?, isContextual: Bool
  ) {
    self.command = command
    self.callback = callback
    self.error = error
    self.data = data
    self.sendResponse = sendResponse
    self.sendChannelData = sendChannelData
    self.viewController = viewController
    self.isContextual = isContextual
  }

  /// Returns the view controller to present from. Call on the main thread before presenting.
  /// Contextual calls return the top-most controller presented over the origin (UIKit refuses to
  /// present from a controller that is already presenting), or nil if the origin is unavailable,
  /// being dismissed or no longer in a window.
  /// Non-contextual calls use the plugin manager's view controller.
  public func presentingViewController() -> UIViewController? {
    guard isContextual else {
      return PluginManager.shared.viewController
    }
    guard let viewController = viewController,
      !viewController.isBeingDismissed
    else {
      return nil
    }
    var controller = viewController
    while let presented = controller.presentedViewController, !presented.isBeingDismissed {
      controller = presented
    }
    // A full-screen modal detaches the origin's view, so check the controller we present from.
    guard controller.viewIfLoaded?.window != nil else {
      return nil
    }
    return controller
  }

  public func getRawArgs() -> String {
    return self.data
  }

  public func getArgs() throws -> JSObject {
    let jsonData = self.data.data(using: .utf8)!
    let data = try JSONSerialization.jsonObject(with: jsonData, options: [])
    return JSTypes.coerceDictionaryToJSObject(
      (data as! NSDictionary), formattingDatesAsStrings: true)!
  }

  public func parseArgs<T: Decodable>(_ type: T.Type) throws -> T {
    let jsonData = self.data.data(using: .utf8)!
    let decoder = JSONDecoder()
    decoder.userInfo[channelDataKey] = sendChannelData
    return try decoder.decode(type, from: jsonData)
  }

  func serialize(_ data: JsonValue) -> String {
    do {
      return try data.jsonRepresentation() ?? "\"Failed to serialize payload\""
    } catch {
      return "\"\(error)\""
    }
  }

  public func resolve() {
    sendResponse(callback, nil)
  }

  public func resolve(_ data: JsonObject) {
    resolve(.dictionary(data))
  }

  public func resolve(_ data: JsonValue) {
    sendResponse(callback, serialize(data))
  }

  public func resolve<T: Encodable>(_ data: T) {
    do {
      let json = try JSONEncoder().encode(data)
      sendResponse(callback, String(decoding: json, as: UTF8.self))
    } catch {
      sendResponse(self.error, "\"\(error)\"")
    }
  }

  public func reject(
    _ message: String, code: String? = nil, error: Error? = nil, data: JsonValue? = nil
  ) {
    let payload: NSMutableDictionary = [
      "message": message
    ]

    if let code = code {
      payload["code"] = code
    }

    if let error = error {
      payload["error"] = error
    }

    if let data = data {
      switch data {
      case .dictionary(let dict):
        for entry in dict {
          payload[entry.key] = entry.value
        }
      }
    }

    sendResponse(self.error, serialize(.dictionary(payload as! JsonObject)))
  }

  public func unimplemented() {
    unimplemented("not implemented")
  }

  public func unimplemented(_ message: String) {
    reject(message)
  }

  public func unavailable() {
    unavailable("not available")
  }

  public func unavailable(_ message: String) {
    reject(message)
  }
}
