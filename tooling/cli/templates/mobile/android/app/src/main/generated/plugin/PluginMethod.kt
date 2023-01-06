package {{reverse-domain app.domain}}.{{snake-case app.name}}

@Retention(AnnotationRetention.RUNTIME)
annotation class PluginMethod(val returnType: String = "promise") {
  companion object {
    var RETURN_PROMISE = "promise"
    var RETURN_CALLBACK = "callback"
    var RETURN_NONE = "none"
  }
}
