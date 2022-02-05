;(function () {
  if (window.location.origin.startsWith(__TEMPLATE_origin__)) {
    __RAW_freeze_prototype__

    __RAW_pattern_script__

    __RAW_ipc_script__
    ;(function () {
      __RAW_bundle_script__
    })()

    __RAW_listen_function__

    __RAW_core_script__

    __RAW_event_initialization_script__

    if (window.ipc) {
      window.__TAURI_INVOKE__('__initialized', { url: window.location.href })
    } else {
      window.addEventListener('DOMContentLoaded', function () {
        window.__TAURI_INVOKE__('__initialized', { url: window.location.href })
      })
    }

    __RAW_plugin_initialization_script__
  }
})()
