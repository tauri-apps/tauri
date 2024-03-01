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
