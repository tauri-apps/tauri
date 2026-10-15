---
"tauri-runtime-cef": patch:feat
---

`data_store_identifier` is now supported on the CEF runtime: the identifier names a profile directory under the runtime's cache path, giving the same isolation `data_directory` does (`data_directory` wins when both are set). The runtime also logs a warning when a webview sets an attribute it cannot honour — `transparent`, `accept_first_mouse`, `browser_extensions_enabled` / `extensions_path`, a `background_throttling` policy or an overlay `scroll_bar_style` — naming the runtime-wide `Cef` API that does the same thing where one exists. The attributes' documentation on `tauri` and `tauri-utils` now describes each one's CEF behaviour.
