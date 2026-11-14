// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const OS_NAME = __TEMPLATE_os_name__

window.addEventListener('keydown', (event) => {
  if (OS_NAME === 'macos' ? event.metaKey : event.ctrlKey) {
    let value
    if (event.key === '-') {
      value = { delta: -0.2 }
    } else if (event.key === '=' || event.key === '+') {
      value = { delta: 0.2 }
    } else if (event.key === '0') {
      value = 1
    } else {
      return
    }
    window.__TAURI_INTERNALS__.invoke('plugin:webview|set_webview_zoom', {
      value
    })
  }
})

window.addEventListener('mousewheel', (event) => {
  if (event.ctrlKey) {
    event.preventDefault()
    window.__TAURI_INTERNALS__.invoke('plugin:webview|set_webview_zoom', {
      value: { delta: event.deltaY < 0 ? 0.2 : -0.2 }
    })
  }
})
