public class Channel {
  var callback: UInt64
  var handler: (JsonValue) -> Void

  public init(callback: UInt64, handler: @escaping (JsonValue) -> Void) {
    self.callback = callback
    self.handler = handler
  }

  public func send(_ data: JsonObject) {
    handler(.dictionary(data))
  }
}
