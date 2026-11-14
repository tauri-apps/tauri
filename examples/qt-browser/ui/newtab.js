// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The new tab page: its search box behaves like the address bar.
const { invoke } = window.__TAURI__.core
const label = window.__TAURI__.webview.getCurrentWebview().label
const id = Number(label.replace('tab-', ''))

invoke('get_settings').then(({ settings }) => {
  if (settings.theme === 'light' || settings.theme === 'dark')
    document.documentElement.dataset.theme = settings.theme
})

document.getElementById('form').addEventListener('submit', (e) => {
  e.preventDefault()
  const input = document.getElementById('search').value.trim()
  if (input) invoke('navigate', { id, input })
})
