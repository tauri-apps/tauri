// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// qtbrowser://about: the app, os and process plugins' JavaScript APIs, plus `about_info`.
const T = window.__TAURI__
const set = (id, text) => (document.getElementById(id).textContent = text)

T.core.invoke('get_settings').then(({ settings }) => {
  if (settings.theme === 'light' || settings.theme === 'dark')
    document.documentElement.dataset.theme = settings.theme
})
;(async () => {
  set('version', `${await T.app.getName()} ${await T.app.getVersion()}`)
  const info = await T.core.invoke('about_info')
  set('tauri', `${info.tauri} (app API: ${await T.app.getTauriVersion()})`)
  set('webview', info.webview)
  set('config', info.paths.config)
  set('data', info.paths.data)
  set('logs', info.paths.logs)
  set('downloads', info.paths.downloads)
  set('shortcut', info.globalShortcut.keys)
  set(
    'shortcutState',
    info.globalShortcut.registered
      ? 'Registered'
      : 'Not registered (turned off, or unsupported here, e.g. Wayland)'
  )
  if (T.os) {
    set('os', `${T.os.platform()} ${T.os.version()} (${T.os.family()})`)
    set('arch', T.os.arch())
    set('locale', (await T.os.locale()) || 'unknown')
    set('hostname', (await T.os.hostname()) || 'unknown')
  }
})()

document
  .getElementById('restart')
  .addEventListener('click', () => T.process.relaunch())
