// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation

public typealias JsonObject = [String: Any?]

public enum JsonValue {
	case dictionary(JsonObject)

	enum SerializationError: Error {
		case invalidObject
	}

	public func jsonRepresentation(includingFields: JsonObject? = nil) throws -> String? {
		switch self {
		case .dictionary(var dictionary):
			if let fields = includingFields {
				dictionary.merge(fields) { (current, _) in current }
			}
			dictionary = prepare(dictionary: dictionary)
			guard JSONSerialization.isValidJSONObject(dictionary) else {
				throw SerializationError.invalidObject
			}
			let data = try JSONSerialization.data(withJSONObject: dictionary, options: [])
			return String(data: data, encoding: .utf8)
		}
	}

	private static let formatter = ISO8601DateFormatter()

	private func prepare(dictionary: JsonObject) -> JsonObject {
		return dictionary.mapValues { (value) -> Any in
			if let date = value as? Date {
				return JsonValue.formatter.string(from: date)
			} else if let aDictionary = value as? JsonObject {
				return prepare(dictionary: aDictionary)
			} else if let anArray = value as? [Any] {
				return prepare(array: anArray)
			}
			return value
		}
	}

	private func prepare(array: [Any]) -> [Any] {
		return array.map { (value) -> Any in
			if let date = value as? Date {
				return JsonValue.formatter.string(from: date)
			} else if let aDictionary = value as? JsonObject {
				return prepare(dictionary: aDictionary)
			} else if let anArray = value as? [Any] {
				return prepare(array: anArray)
			}
			return value
		}
	}
}
