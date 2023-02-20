-keep class app.tauri.** {
  @app.tauri.JniMethod public <methods>;
}

-keep class app.tauri.JSArray,app.tauri.JSObject {
  public <init>(...);
}

-keep @app.tauri.annotation.TauriPlugin public class * {
  @app.tauri.annotation.PluginMethod public <methods>;
  @app.tauri.annotation.PermissionCallback <methods>;
  @app.tauri.annotation.ActivityCallback <methods>;
  @app.tauri.annotation.Permission <methods>;
  public <init>(...);
}
