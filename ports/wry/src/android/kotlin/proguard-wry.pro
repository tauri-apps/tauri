# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class {{package-unescaped}}.* {
  native <methods>;
}

-keep class {{package-unescaped}}.WryActivity {
  public <init>(...);

  void setWebView({{package-unescaped}}.RustWebView);
  java.lang.Class getAppClass(...);
  int getId();
  java.lang.String getVersion();
  int startActivity(...);
}

-keep class {{package-unescaped}}.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class {{package-unescaped}}.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class {{package-unescaped}}.RustWebChromeClient,{{package-unescaped}}.RustWebViewClient {
  public <init>(...);
}
