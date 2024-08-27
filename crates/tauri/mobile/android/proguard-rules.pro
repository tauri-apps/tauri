-keep class app.tauri.** {
  @app.tauri.JniMethod public <methods>;
  native <methods>;
}

-keep class app.tauri.plugin.JSArray {
  public <init>(...);
}

-keepclassmembers class org.json.JSONArray {
  public put(...);
}

-keep class app.tauri.plugin.JSObject {
  public <init>(...);
  public put(...);
}

-keep @app.tauri.annotation.TauriPlugin public class * {
  @app.tauri.annotation.Command public <methods>;
  @app.tauri.annotation.PermissionCallback <methods>;
  @app.tauri.annotation.ActivityCallback <methods>;
  @app.tauri.annotation.Permission <methods>;
  public <init>(...);
}

-keep @app.tauri.annotation.InvokeArg public class * {
  *;
}
