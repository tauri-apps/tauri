package app.tauri.plugin

@Retention(AnnotationRetention.RUNTIME)
annotation class PluginMethod(val returnType: String = "promise") { }
