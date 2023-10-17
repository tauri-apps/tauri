// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  function toggleDevtoolsHotkey() {
    const osName = __TEMPLATE_os_name__

    const isHotkey =
      osName === 'macos'
        ? (event) => event.metaKey && event.altKey && event.key === 'I'
        : (event) => event.ctrlKey && event.shiftKey && event.key === 'I'

    document.addEventListener('keydown', (event) => {
      if (isHotkey(event)) {
        window.__TAURI_INVOKE__('plugin:window|internal_toggle_devtools')
      }
    })
  }

  if (
    document.readyState === 'complete' ||
    document.readyState === 'interactive'
  ) {
    toggleDevtoolsHotkey()
  } else {
    window.addEventListener('DOMContentLoaded', toggleDevtoolsHotkey, true)
  }
})()
