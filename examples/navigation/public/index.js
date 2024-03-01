// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const WebviewWindow = window.__TAURI__.webview.WebviewWindow

const routeSelect = document.querySelector('#route')
const link = document.querySelector('#link')

routeSelect.addEventListener('change', (event) => {
  link.href = event.target.value
})

document.querySelector('#go').addEventListener('click', () => {
  window.location.href = window.location.origin + '/' + routeSelect.value
})

document.querySelector('#open-window').addEventListener('click', () => {
  new WebviewWindow(Math.random().toString().replace('.', ''), {
    url: routeSelect.value
  })
})
