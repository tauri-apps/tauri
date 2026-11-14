// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The title bar and toolbar of one browser window. The Rust side owns the tabs; this page
// renders the `browser-tabs` snapshots it sends and calls commands for every action.
const { invoke } = window.__TAURI__.core
const appWindow = window.__TAURI__.window.getCurrentWindow()
const thisWebview = window.__TAURI__.webview.getCurrentWebview()

const $ = (id) => document.getElementById(id)
const tabsEl = $('tabs')
const address = $('address')

let state = { tabs: [], active: null, settings: {} }
let editing = false

const activeTab = () => state.tabs.find((t) => t.id === state.active)

// --- rendering ------------------------------------------------------------------------------

function applyTheme(theme) {
  if (theme === 'light' || theme === 'dark')
    document.documentElement.dataset.theme = theme
  else delete document.documentElement.dataset.theme
}

function favicon(tab) {
  const el = document.createElement('span')
  el.className = 'favicon'
  if (tab.loading) {
    el.classList.add('loading')
    return el
  }
  const letter = (tab.title || '?').trim().charAt(0).toUpperCase()
  if (tab.origin) {
    const img = new Image()
    img.src = `${tab.origin}/favicon.ico`
    img.alt = ''
    img.onerror = () => {
      img.remove()
      el.textContent = letter
    }
    el.append(img)
  } else {
    el.textContent = tab.internal ? '◆' : letter
  }
  return el
}

function renderTabs() {
  tabsEl.replaceChildren(
    ...state.tabs.map((tab, index) => {
      const el = document.createElement('div')
      el.className = 'tab' + (tab.id === state.active ? ' active' : '')
      el.setAttribute('role', 'tab')
      el.setAttribute('aria-selected', tab.id === state.active)
      el.title = tab.title
      el.draggable = true
      el.dataset.id = tab.id

      const title = document.createElement('span')
      title.className = 'title'
      title.textContent = tab.title

      const close = document.createElement('button')
      close.className = 'close'
      close.title = 'Close tab (Ctrl+W)'
      close.setAttribute('aria-label', 'Close tab')
      close.innerHTML =
        '<svg viewBox="0 0 8 8"><path d="M1 1l6 6M7 1L1 7"/></svg>'
      close.addEventListener('click', (e) => {
        e.stopPropagation()
        invoke('close_tab', { id: tab.id })
      })

      el.append(favicon(tab), title, close)
      el.addEventListener('mousedown', (e) => {
        if (e.button === 0 && tab.id !== state.active)
          invoke('activate_tab', { id: tab.id })
      })
      // middle click closes, like every browser
      el.addEventListener('auxclick', (e) => {
        if (e.button === 1) invoke('close_tab', { id: tab.id })
      })

      // drag to reorder
      el.addEventListener('dragstart', (e) => {
        e.dataTransfer.setData('text/x-tab', String(tab.id))
        e.dataTransfer.effectAllowed = 'move'
        el.classList.add('dragging')
      })
      el.addEventListener('dragend', () => el.classList.remove('dragging'))
      el.addEventListener('dragover', (e) => {
        if (e.dataTransfer.types.includes('text/x-tab')) e.preventDefault()
      })
      el.addEventListener('drop', (e) => {
        e.preventDefault()
        const id = Number(e.dataTransfer.getData('text/x-tab'))
        if (id && id !== tab.id) invoke('move_tab', { id, index })
      })
      return el
    })
  )
}

function renderToolbar() {
  const tab = activeTab()
  document.body.classList.toggle('loading', !!tab?.loading)
  $('back').disabled = !tab?.canGoBack
  $('forward').disabled = !tab?.canGoForward
  $('home').hidden = !state.settings.showHomeButton
  $('reload').title = tab?.loading ? 'Stop loading' : 'Reload (Ctrl+R)'
  if (!editing) address.value = tab?.url ?? ''
  const badge = $('site-badge')
  badge.className = tab?.url?.startsWith('https://') ? 'secure' : ''
  badge.title = tab?.url?.startsWith('https://') ? 'Connection is secure' : ''
  $('downloads').hidden = !state.downloads
  $('downloads-count').textContent = state.downloads
    ? String(state.downloads)
    : ''
  const zoom = $('zoom-badge')
  const pct = Math.round((tab?.zoom ?? 1) * 100)
  zoom.hidden = pct === Math.round((state.settings.defaultZoom ?? 1) * 100)
  zoom.textContent = `${pct}%`
  document.title = tab ? `${tab.title} - Qt Browser` : 'Qt Browser'
}

let firstRender = true
function render(next) {
  if (!next) return
  state = next
  // A new window opens on the new tab page: focus the address bar (the Rust side's request to
  // do so may have come before this page was ready).
  if (firstRender) {
    firstRender = false
    if (activeTab()?.url === '') setTimeout(() => address.focus())
  }
  applyTheme(state.settings.theme)
  renderTabs()
  renderToolbar()
}

// --- actions --------------------------------------------------------------------------------

const withActive = (fn) => {
  const tab = activeTab()
  if (tab) fn(tab.id)
}

$('new-tab').addEventListener('click', () => invoke('new_tab'))
$('back').addEventListener('click', () =>
  withActive((id) => invoke('go_back', { id }))
)
$('forward').addEventListener('click', () =>
  withActive((id) => invoke('go_forward', { id }))
)
$('reload').addEventListener('click', () =>
  withActive((id) => invoke(activeTab().loading ? 'stop' : 'reload', { id }))
)
$('home').addEventListener('click', () =>
  invoke('shortcut', { action: 'home' })
)
$('zoom-badge').addEventListener('click', () =>
  invoke('shortcut', { action: 'zoom-reset' })
)
$('downloads').addEventListener('click', () =>
  invoke('shortcut', { action: 'downloads' })
)
$('site-badge').addEventListener('click', () => {
  const r = $('site-badge').getBoundingClientRect()
  invoke('site_info_menu', { x: r.left, y: r.bottom + 6 })
})
$('menu').addEventListener('click', () => {
  const r = $('menu').getBoundingClientRect()
  invoke('open_menu', { x: r.right - 220, y: r.bottom })
})

$('minimize').addEventListener('click', () => appWindow.minimize())
$('maximize').addEventListener('click', () => appWindow.toggleMaximize())
$('close').addEventListener('click', () => appWindow.close())

// the address bar
address.addEventListener('focus', () => {
  editing = true
  address.select()
})
address.addEventListener('blur', () => {
  editing = false
  renderToolbar()
})
$('address-form').addEventListener('submit', (e) => {
  e.preventDefault()
  const input = address.value.trim()
  if (!input) return
  editing = false
  address.blur()
  withActive((id) => invoke('navigate', { id, input }))
})
address.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    editing = false
    renderToolbar()
    invoke('focus_content')
  }
})

// the same shortcuts as in pages (see SHORTCUT_SCRIPT in src/main.rs)
addEventListener(
  'keydown',
  (e) => {
    const ctrl = e.ctrlKey || e.metaKey
    const k = e.key.toLowerCase()
    let action = null
    if (ctrl && e.shiftKey && k === 't') action = 'reopen-tab'
    else if (ctrl && k === 'tab') action = e.shiftKey ? 'prev-tab' : 'next-tab'
    else if (ctrl && !e.shiftKey && !e.altKey) {
      action =
        {
          t: 'new-tab',
          n: 'new-window',
          w: 'close-tab',
          r: 'reload',
          ',': 'settings',
          '=': 'zoom-in',
          '+': 'zoom-in',
          '-': 'zoom-out',
          0: 'zoom-reset',
          o: 'open-file',
          s: 'save-page',
          p: 'print',
          j: 'downloads'
        }[k] ?? (k >= '1' && k <= '9' ? 'tab-' + k : null)
      if (k === 'l') {
        e.preventDefault()
        address.focus()
        return
      }
    } else if (k === 'f5') action = 'reload'
    else if (k === 'f11') action = 'fullscreen'
    else if (k === 'f12') action = 'devtools'
    if (action) {
      e.preventDefault()
      invoke('shortcut', { action })
    }
  },
  true
)

// window state
async function syncMaximized() {
  const max = await appWindow.isMaximized()
  $('maximize').title = max ? 'Restore' : 'Maximize'
  $('maximize').innerHTML = max
    ? '<svg viewBox="0 0 10 10"><rect x="1.5" y="3" width="5.5" height="5.5"/><path d="M3 3V1.5h5.5V7H7"/></svg>'
    : '<svg viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7"/></svg>'
}
appWindow.onResized(syncMaximized)
appWindow.onFocusChanged(({ payload }) =>
  document.body.classList.toggle('inactive', !payload)
)
window
  .matchMedia('(prefers-color-scheme: dark)')
  .addEventListener('change', renderToolbar)

// events from Rust
let toastTimer
thisWebview.listen('browser-tabs', ({ payload }) => render(payload))
thisWebview.listen('browser-focus-address', () => address.focus())
thisWebview.listen('browser-toast', ({ payload }) => {
  const toast = $('toast')
  toast.textContent = payload
  toast.hidden = false
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => (toast.hidden = true), 4000)
})

invoke('browser_state').then(render)
syncMaximized()
