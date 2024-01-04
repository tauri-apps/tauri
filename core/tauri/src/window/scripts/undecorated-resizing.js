// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  const osName = __TEMPLATE_os_name__
  if (osName !== 'macos') {
    document.addEventListener('mousemove', (e) => {
      window.__TAURI_INTERNALS__.invoke('plugin:window|on_mousemove', {
        x: e.clientX,
        y: e.clientY
      })
    })
    document.addEventListener('mousedown', (e) => {
      window.__TAURI_INTERNALS__.invoke('plugin:window|on_mousedown', {
        x: e.clientX,
        y: e.clientY
      })
    })
  }
})()
