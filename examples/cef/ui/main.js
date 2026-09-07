// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { invoke } = window.__TAURI__.core
const { listen } = window.__TAURI__.event
const { getAllWebviews } = window.__TAURI__.webview

/** How many entries a log keeps before the oldest are dropped. */
const LOG_LIMIT = 150

const $ = (id) => document.getElementById(id)

/** Appends one line to a log panel, keeping it scrolled to the newest entry. */
function log(id, tag, text, tone) {
  const panel = $(id)
  const entry = document.createElement('div')
  entry.className = 'entry'
  entry.dataset.text = `${tag} ${text}`

  const label = document.createElement('span')
  label.className = `tag ${tone ?? ''}`
  label.textContent = tag
  entry.append(label, document.createTextNode(text))

  const atBottom =
    panel.scrollTop + panel.clientHeight >= panel.scrollHeight - 4
  panel.append(entry)
  while (panel.childElementCount > LOG_LIMIT) panel.firstElementChild.remove()
  if (atBottom) panel.scrollTop = panel.scrollHeight
}

// --- runtime configuration ---------------------------------------------------

async function loadRuntimeInfo() {
  const info = await invoke('runtime_info')
  $('chrome-version').textContent = `Chromium ${info.chromeVersion}`
  $('cef-api-version').textContent = `CEF API ${info.cefApiVersion}`

  const table = $('configuration')
  for (const { name, value, hint } of info.configuration) {
    const row = table.insertRow()
    row.insertCell().textContent = name
    const cell = row.insertCell()
    cell.append(value)
    const note = document.createElement('span')
    note.className = 'hint'
    note.textContent = hint
    cell.append(note)
  }

  $('languages').textContent = navigator.languages.join(', ')
  // Chromium has no API that reports the autoplay policy, so the readback is
  // the behaviour itself: without the switch this context would start
  // `suspended`, because the page has had no user gesture.
  const context = new AudioContext()
  $('autoplay').textContent = context.state
  context.close()
  $('expose-gc').textContent =
    typeof window.gc === 'function'
      ? 'a function, so --expose-gc reached this renderer'
      : 'undefined'
}

// --- console messages --------------------------------------------------------

document.querySelectorAll('[data-console]').forEach((button) => {
  button.addEventListener('click', () => {
    const kind = button.dataset.console
    if (kind === 'throw') {
      // Reported by the renderer as an error even though no console API ran.
      setTimeout(() => {
        throw new Error('thrown from the example page')
      })
    } else if (kind === 'subresource') {
      // A blocked subresource is also console output, from the renderer itself.
      const image = new Image()
      image.src = 'https://example.invalid/missing.png'
    } else {
      console[kind](`console.${kind} from the example page`, { at: Date.now() })
    }
  })
})

// --- frame lifecycle ---------------------------------------------------------

document.querySelectorAll('[data-frame]').forEach((button) => {
  button.addEventListener('click', () => {
    const frame = $('child-frame')
    const target = button.dataset.frame
    if (target === 'remove') {
      frame.remove()
      return
    }
    if (!frame.isConnected) {
      // Recreate it so the destroyed/created halves of the lifecycle are visible.
      const replacement = document.createElement('iframe')
      replacement.id = 'child-frame'
      replacement.title = 'child frame'
      replacement.src = target
      $('frame-log').before(replacement)
      return
    }
    frame.src = target
  })
})

// --- Chrome accelerators -----------------------------------------------------

const COMMAND_GROUPS = [
  {
    id: 'windowAndTab',
    name: 'WindowAndTab',
    description:
      'Ctrl+N, Ctrl+Shift+N, Ctrl+T and the tab strip. The windows these open are real Chrome windows the app does not own.'
  },
  {
    id: 'document',
    name: 'Document',
    description:
      'Ctrl+P, Ctrl+S, Ctrl+U, Ctrl+O. The commonest one to want back — but Ctrl+O and Ctrl+S raise OS file dialogs the app never asked for.'
  },
  {
    id: 'browserChrome',
    name: 'BrowserChrome',
    description:
      'Ctrl+L and its neighbours: focus the omnibox, the toolbar, the bookmarks bar — none of which an app window has.'
  },
  {
    id: 'browserSurface',
    name: 'BrowserSurface',
    description:
      'Ctrl+H, Ctrl+J, Ctrl+D and friends. These load Chrome WebUI pages in place of the app UI, in the very webview the key was pressed in.'
  },
  {
    id: 'history',
    name: 'History',
    description:
      'Alt+Left and Alt+Right. The app sits on a second history entry, so going back from its first screen lands on a blank page.'
  }
]

for (const group of COMMAND_GROUPS) {
  const label = document.createElement('label')
  const input = document.createElement('input')
  input.type = 'checkbox'
  input.value = group.id
  const text = document.createElement('div')
  const name = document.createElement('b')
  name.textContent = group.name
  const description = document.createElement('span')
  description.textContent = group.description
  text.append(name, description)
  label.append(input, text)
  $('command-groups').append(label)
}

$('open-accelerators').addEventListener('click', async () => {
  const groups = [
    ...document.querySelectorAll('#command-groups input:checked')
  ].map((input) => input.value)
  try {
    const label = await invoke('open_accelerator_window', {
      options: {
        groups,
        runtimeStyle: $('runtime-style').value,
        zoomHotkeys: $('zoom-hotkeys').checked
      }
    })
    log(
      'popup-log',
      'window',
      `opened ${label} keeping [${groups.join(', ') || 'nothing'}]`
    )
    await refreshSnapshotTargets()
  } catch (error) {
    log('popup-log', 'error', String(error), 'error')
  }
})

// --- DevTools protocol -------------------------------------------------------

const CDP_PRESETS = [
  ['Browser.getVersion', {}],
  ['Page.getNavigationHistory', {}],
  ['Runtime.evaluate', { expression: 'navigator.userAgent' }],
  [
    'Emulation.setEmulatedMedia',
    { features: [{ name: 'prefers-color-scheme', value: 'dark' }] }
  ],
  ['Network.enable', {}]
]

for (const [method, params] of CDP_PRESETS) {
  const button = document.createElement('button')
  button.textContent = method
  button.addEventListener('click', () => {
    $('cdp-method').value = method
    $('cdp-params').value = JSON.stringify(params, null, 1)
  })
  $('cdp-presets').append(button)
}

$('cdp-send').addEventListener('click', async () => {
  let params
  try {
    params = JSON.parse($('cdp-params').value || '{}')
  } catch (error) {
    log('cdp-log', 'error', `invalid params JSON: ${error}`, 'error')
    return
  }
  const method = $('cdp-method').value.trim()
  try {
    const messageId = await invoke('send_devtools_message', { method, params })
    log('cdp-log', 'sent', `#${messageId} ${method}`, 'ok')
  } catch (error) {
    log('cdp-log', 'error', String(error), 'error')
  }
})

$('cdp-clear').addEventListener('click', () => {
  $('cdp-log').replaceChildren()
})

$('cdp-filter').addEventListener('input', () => {
  const needle = $('cdp-filter').value.toLowerCase()
  for (const entry of $('cdp-log').children) {
    entry.hidden =
      needle !== '' && !entry.dataset.text.toLowerCase().includes(needle)
  }
})

// --- native snapshot ---------------------------------------------------------

/**
 * Lists every webview that can be sampled — child webviews of a multiwebview
 * window included — so a dialog can be observed in one that is not this one.
 */
async function refreshSnapshotTargets() {
  const select = $('snapshot-target')
  const selected = select.value
  const webviews = await getAllWebviews()
  select.replaceChildren()
  for (const label of [
    '',
    ...webviews.map((webview) => webview.label).sort()
  ]) {
    const option = document.createElement('option')
    option.value = label
    option.textContent = label === '' ? 'this webview' : label
    select.append(option)
  }
  select.value = [...select.options].some((option) => option.value === selected)
    ? selected
    : ''
}

$('snapshot-target').addEventListener('focus', () => {
  refreshSnapshotTargets().catch(() => {})
})

$('snapshot').addEventListener('click', async () => {
  const label = $('snapshot-target').value
  try {
    // An absent label samples the calling webview.
    const snapshot = await invoke('native_snapshot', {
      options: { label: label || null }
    })
    $('snapshot-output').textContent = JSON.stringify(snapshot, null, 2)
  } catch (error) {
    $('snapshot-output').textContent = String(error)
  }
})

// --- child webviews ----------------------------------------------------------

$('open-children').addEventListener('click', async () => {
  try {
    const label = await invoke('open_child_webviews_window')
    log('popup-log', 'window', `opened ${label} with two child webviews`)
    await refreshSnapshotTargets()
  } catch (error) {
    log('popup-log', 'error', String(error), 'error')
  }
})

// --- popups ------------------------------------------------------------------

document.querySelectorAll('[data-popup]').forEach((button) => {
  button.addEventListener('click', () => {
    window.open(button.dataset.popup, '_blank', 'width=520,height=420')
  })
})

// --- permissions -------------------------------------------------------------

const PERMISSION_REQUESTS = {
  notifications: () => Notification.requestPermission(),
  geolocation: () =>
    new Promise((resolve, reject) =>
      navigator.geolocation.getCurrentPosition(
        (position) =>
          resolve(`${position.coords.latitude}, ${position.coords.longitude}`),
        reject
      )
    ),
  camera: () => navigator.mediaDevices.getUserMedia({ video: true }),
  display: () => navigator.mediaDevices.getDisplayMedia({ video: true }),
  'storage-access': () => document.requestStorageAccess()
}

document.querySelectorAll('[data-permission]').forEach((button) => {
  button.addEventListener('click', async () => {
    const kind = button.dataset.permission
    log('permission-log', 'page', `requested ${kind}`)
    try {
      const result = await PERMISSION_REQUESTS[kind]()
      if (result instanceof MediaStream) {
        for (const track of result.getTracks()) track.stop()
        log('permission-log', 'page', `${kind}: stream granted`, 'ok')
      } else {
        log('permission-log', 'page', `${kind}: ${result ?? 'resolved'}`, 'ok')
      }
    } catch (error) {
      log('permission-log', 'page', `${kind}: ${error}`, 'warning')
    }
  })
})

// --- everything the native observers reported --------------------------------

listen('cef://event', ({ payload }) => {
  switch (payload.kind) {
    case 'console':
      log(
        'console-log',
        payload.level,
        `${payload.message}  (${payload.window}${
          payload.source ? `, ${payload.source}:${payload.line}` : ''
        })`,
        payload.level
      )
      break
    case 'frame':
      log(
        'frame-log',
        payload.event,
        `${payload.isMain ? 'main' : 'child'} frame ${
          payload.frameId ? payload.frameId.slice(0, 12) : '(none)'
        }${payload.url ? ` → ${payload.url}` : ''}`
      )
      break
    case 'devTools':
      if (payload.event === 'event') {
        log(
          'cdp-log',
          'event',
          `${payload.method} ${truncate(payload.payload)}`
        )
      } else {
        log(
          'cdp-log',
          `#${payload.messageId}`,
          truncate(payload.payload),
          payload.success ? 'ok' : 'error'
        )
      }
      break
    case 'popup':
      log(
        'popup-log',
        'new window',
        `${payload.url}\n  opener: ${payload.openerSource ?? 'not reported'}\n  answered with: ${payload.label}`
      )
      refreshSnapshotTargets().catch(() => {})
      break
    case 'permission':
      log(
        'permission-log',
        payload.response.toLowerCase(),
        `${payload.permission} — ${payload.note}`,
        payload.response === 'Deny' ? 'error' : 'ok'
      )
      break
    case 'deepLink':
      log('deeplink-log', 'opened', payload.urls.join(', '), 'ok')
      break
  }
})

function truncate(text, limit = 400) {
  return text.length > limit ? `${text.slice(0, limit)}…` : text
}

loadRuntimeInfo().catch((error) => {
  document.querySelector('header p').textContent =
    `failed to load runtime info: ${error}`
})

// The list is also refreshed whenever the select is focused, so a window closed
// in the meantime does not stay on offer.
refreshSnapshotTargets().catch(() => {})
