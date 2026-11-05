package com.tauri.api

// Hosts the webviews added to a window that already has one; it is started in the window's task,
// where the rules in res/xml/main_split_config.xml show it next to MainActivity.
class EmbeddedWebviewActivity : TauriActivity()
