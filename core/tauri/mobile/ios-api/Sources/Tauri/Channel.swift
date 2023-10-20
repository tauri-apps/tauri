// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation

let CHANNEL_PREFIX = "__CHANNEL__:"
let channelDataKey = CodingUserInfoKey(rawValue: "sendChannelData")!

public class Channel: Decodable {
  public let id: UInt64
  let handler: (String) -> Void

  public required init(from decoder: Decoder) throws {
    let container = try decoder.singleValueContainer()
    let channelDef = try container.decode(String.self)

    let components = channelDef.components(separatedBy: CHANNEL_PREFIX)
    if components.count < 2 {
      throw DecodingError.dataCorruptedError(
        in: container,
        debugDescription: "Invalid channel definition from \(channelDef)"
      )

    }
    guard let channelId = UInt64(components[1]) else {
      throw DecodingError.dataCorruptedError(
        in: container,
        debugDescription: "Invalid channel ID from \(channelDef)"
      )
    }

    guard let handler = decoder.userInfo[channelDataKey] as? (String) -> Void else {
      throw DecodingError.dataCorruptedError(
        in: container,
        debugDescription: "missing userInfo for Channel handler. This is a Tauri issue"
      )
    }

    self.id = channelId
    self.handler = handler
  }

  func serialize(_ data: JsonValue) -> String {
    do {
      return try data.jsonRepresentation() ?? "\"Failed to serialize payload\""
    } catch {
      return "\"\(error)\""
    }
  }

  public func send(_ data: JsonObject) {
    send(.dictionary(data))
  }

  public func send(_ data: JsonValue) {
    handler(serialize(data))
  }

  public func resolve<T: Encodable>(_ data: T) throws {
    let json = try JSONEncoder().encode(data)
    handler(String(decoding: json, as: UTF8.self))
  }

}
