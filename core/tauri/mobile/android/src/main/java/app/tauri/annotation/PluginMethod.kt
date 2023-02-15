package app.tauri.annotation

@Retention(AnnotationRetention.RUNTIME)
annotation class PluginMethod(val returnType: String = "promise") { }
