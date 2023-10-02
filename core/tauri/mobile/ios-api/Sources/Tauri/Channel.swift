// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

public class Channel {
  public let id: UInt64
  let handler: (JsonValue) -> Void

  public init(id: UInt64, handler: @escaping (JsonValue) -> Void) {
    self.id = id
    self.handler = handler
  }

  public func send(_ data: JsonObject) {
    handler(.dictionary(data))
  }
}
