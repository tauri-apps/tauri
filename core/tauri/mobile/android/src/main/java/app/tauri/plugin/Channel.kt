package app.tauri.plugin

class Channel(val id: Long, private val handler: (data: JSObject) -> Unit) {
  fun send(data: JSObject) {
    handler(data)
  }
}
