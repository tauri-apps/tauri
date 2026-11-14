// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// qtbrowser://settings: every change is saved right away (store plugin) and applied.
const { invoke } = window.__TAURI__.core
const $ = (id) => document.getElementById(id)
const saved = $('saved')
const fields = [
  'searchEngine',
  'homepage',
  'theme',
  'defaultZoom',
  'showHomeButton',
  'showMenuBar',
  'keepInTray',
  'notifications',
  'globalShortcut',
  'confirmClose',
  'userAgent'
]
let current = {}

function applyTheme(theme) {
  if (theme === 'light' || theme === 'dark')
    document.documentElement.dataset.theme = theme
  else delete document.documentElement.dataset.theme
}

function show(settings, autostart) {
  current = settings
  for (const f of fields) {
    const el = $(f)
    if (el.type === 'checkbox') el.checked = !!settings[f]
    else el.value = String(settings[f])
  }
  $('autostart').checked = autostart
  $('downloadDirLabel').textContent =
    settings.downloadDir || "The system's downloads folder"
  $('resetDownloadDir').hidden = !settings.downloadDir
  applyTheme(settings.theme)
}

function read() {
  const s = { ...current }
  for (const f of fields) {
    const el = $(f)
    s[f] =
      el.type === 'checkbox'
        ? el.checked
        : f === 'defaultZoom'
          ? Number(el.value)
          : el.value.trim()
  }
  return s
}

async function save(settings) {
  try {
    await invoke('save_settings', { settings })
    current = settings
    applyTheme(settings.theme)
    saved.textContent = 'Saved'
  } catch (e) {
    saved.textContent = 'Could not save: ' + e
  }
}

invoke('get_settings').then(({ settings, autostart }) =>
  show(settings, autostart)
)

$('form').addEventListener('change', (e) => {
  if (e.target.id === 'autostart') {
    invoke('set_autostart', { enabled: e.target.checked })
      .then(
        (on) => (
          (e.target.checked = on),
          (saved.textContent = on ? 'Opens at login' : "Won't open at login")
        )
      )
      .catch((err) => (saved.textContent = 'Autostart: ' + err))
    return
  }
  save(read())
})
$('form').addEventListener('submit', (e) => e.preventDefault())
$('changeDownloadDir').addEventListener('click', async () => {
  const dir = await invoke('pick_download_dir')
  if (dir) {
    await save({ ...read(), downloadDir: dir })
    show(current, $('autostart').checked)
  }
})
$('resetDownloadDir').addEventListener('click', async () => {
  await save({ ...read(), downloadDir: '' })
  show(current, $('autostart').checked)
})
$('clear').addEventListener('click', async () => {
  await invoke('clear_browsing_data')
  saved.textContent = 'Browsing data cleared'
})
