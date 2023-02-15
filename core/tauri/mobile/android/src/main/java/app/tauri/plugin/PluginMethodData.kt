package app.tauri.plugin

import app.tauri.annotation.PluginMethod
import java.lang.reflect.Method

class PluginMethodData(
  val method: Method, methodDecorator: PluginMethod
) {

  // The name of the method
  val name: String = method.name

  // The return type of the method (see PluginMethod for constants)
  val returnType: String = methodDecorator.returnType
}
