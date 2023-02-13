-keep class app.tauri.** {
  @app.tauri.JniMethod public <methods>;
}

-keep class app.tauri.JSArray,app.tauri.JSObject {
  public <init>(...);
}

-keep @app.tauri.plugin.TauriPlugin public class * {
  @app.tauri.plugin.PluginMethod public <methods>;
  @app.tauri.PermissionCallback <methods>;
  @app.tauri.ActivityCallback <methods>;
  @app.tauri.Permission <methods>;
  public <init>(...);
}
