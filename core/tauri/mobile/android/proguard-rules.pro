-keep class app.tauri.** {
  @app.tauri.JniMethod public <methods>;
}

-keep class app.tauri.JSArray,app.tauri.JSObject {
  public <init>(...);
}
