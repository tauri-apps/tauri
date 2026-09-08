// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  function toggleDevtoolsHotkey() {
    const isMacos = __TEMPLATE_is_macos__

    const isHotkey = isMacos
      ? (event) => event.metaKey && event.altKey && event.code === 'KeyI'
      : (event) => event.ctrlKey && event.shiftKey && event.code === 'KeyI'

    document.addEventListener('keydown', (event) => {
      if (isHotkey(event)) {
        window.__TAURI_INTERNALS__.invoke(
          'plugin:webview|internal_toggle_devtools'
        )
      }
    })
  }

  if (
    document.readyState === 'complete'
    || document.readyState === 'interactive'
  ) {
    toggleDevtoolsHotkey()
  } else {
    window.addEventListener('DOMContentLoaded', toggleDevtoolsHotkey, true)
  }
})()
