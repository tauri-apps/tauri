package {{reverse-domain app.domain}}.{{snake-case app.name}}

internal class InvalidPluginMethodException : Exception {
  constructor(s: String?) : super(s) {}
  constructor(t: Throwable?) : super(t) {}
  constructor(s: String?, t: Throwable?) : super(s, t) {}
}
