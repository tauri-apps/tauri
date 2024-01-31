// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  const osName = __TEMPLATE_os_name__
  if (osName !== 'macos' && osName !== 'ios' && osName !== 'android') {
    document.addEventListener('mousemove', (e) => {
      window.__TAURI_INTERNALS__.invoke('plugin:window|internal_on_mousemove', {
        x: e.clientX,
        y: e.clientY
      })
    })
    document.addEventListener('mousedown', (e) => {
      window.__TAURI_INTERNALS__.invoke('plugin:window|internal_on_mousedown', {
        x: e.clientX,
        y: e.clientY
      })
    })
  }
})()
