// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation

// declare our empty protocol, and conformance, for typing
public protocol JSValue {}
extension String: JSValue {}
extension Bool: JSValue {}
extension Int: JSValue {}
extension Float: JSValue {}
extension Double: JSValue {}
extension NSNumber: JSValue {}
extension NSNull: JSValue {}
extension Array: JSValue {}
extension Date: JSValue {}
extension Dictionary: JSValue where Key == String, Value == JSValue {}

// convenience aliases
public typealias JSObject = [String: JSValue]
public typealias JSArray = [JSValue]

extension Dictionary where Key == String, Value == JSValue {
  public func getValue(_ key: String) -> JSValue? {
    return self[key]
  }

  public func getString(_ key: String) -> String? {
    return self[key] as? String
  }

  public func getBool(_ key: String) -> Bool? {
    return self[key] as? Bool
  }

  public func getInt(_ key: String) -> Int? {
    return self[key] as? Int
  }

  public func getFloat(_ key: String) -> Float? {
    if let floatValue = self[key] as? Float {
      return floatValue
    } else if let doubleValue = self[key] as? Double {
      return Float(doubleValue)
    }
    return nil
  }

  public func getDouble(_ key: String) -> Double? {
    return self[key] as? Double
  }

  public func getArray(_ key: String) -> JSArray? {
    return self[key] as? JSArray
  }

  public func getObject(_ key: String) -> JSObject? {
    return self[key] as? JSObject
  }
}

/*
 Simply casting objects from foundation class clusters (such as __NSArrayM)
 doesn't work with the JSValue protocol and will always fail. So we need to
 recursively and explicitly convert each value in the dictionary.
 */
public enum JSTypes {}
extension JSTypes {
  public static func coerceDictionaryToJSObject(
    _ dictionary: NSDictionary?, formattingDatesAsStrings: Bool = false
  ) -> JSObject? {
    return coerceToJSValue(dictionary, formattingDates: formattingDatesAsStrings) as? JSObject
  }

  public static func coerceDictionaryToJSObject(
    _ dictionary: [AnyHashable: Any]?, formattingDatesAsStrings: Bool = false
  ) -> JSObject? {
    return coerceToJSValue(dictionary, formattingDates: formattingDatesAsStrings) as? JSObject
  }
}

private let dateStringFormatter = ISO8601DateFormatter()

// We need a large switch statement because we have a lot of types.
// swiftlint:disable:next cyclomatic_complexity
private func coerceToJSValue(_ value: Any?, formattingDates: Bool) -> JSValue? {
  guard let value = value else {
    return nil
  }
  switch value {
  case let stringValue as String:
    return stringValue
  case let numberValue as NSNumber:
    return numberValue
  case let boolValue as Bool:
    return boolValue
  case let intValue as Int:
    return intValue
  case let floatValue as Float:
    return floatValue
  case let doubleValue as Double:
    return doubleValue
  case let dateValue as Date:
    if formattingDates {
      return dateStringFormatter.string(from: dateValue)
    }
    return dateValue
  case let nullValue as NSNull:
    return nullValue
  case let arrayValue as NSArray:
    return arrayValue.compactMap { coerceToJSValue($0, formattingDates: formattingDates) }
  case let dictionaryValue as NSDictionary:
    let keys = dictionaryValue.allKeys.compactMap { $0 as? String }
    var result: JSObject = [:]
    for key in keys {
      result[key] = coerceToJSValue(dictionaryValue[key], formattingDates: formattingDates)
    }
    return result
  default:
    return nil
  }
}
