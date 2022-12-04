package {{reverse-domain app.domain}}.{{snake-case app.name}}

import java.lang.reflect.Method

class PluginHandle(val instance: Plugin) {
  private val pluginMethods: HashMap<String, PluginMethodData> = HashMap()
  init {
    indexMethods()
  }

  @Throws(
    InvalidPluginMethodException::class,
    IllegalAccessException::class
  )
  fun invoke(methodName: String, call: PluginCall) {
    val methodMeta = pluginMethods[methodName]
      ?: throw InvalidPluginMethodException("No method " + methodName + " found for plugin " + instance.javaClass.name)
    methodMeta.method.invoke(instance, call)
  }

  private fun indexMethods() {
    val methods: Array<Method> = instance.javaClass.methods
    for (methodReflect in methods) {
      val method: PluginMethod = methodReflect.getAnnotation(PluginMethod::class.java) ?: continue
      val methodMeta = PluginMethodData(methodReflect, method)
      pluginMethods.put(methodReflect.name, methodMeta)
    }
  }
}