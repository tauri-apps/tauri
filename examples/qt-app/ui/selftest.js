// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Automated end-to-end checks. Runs automatically with QT_APP_AUTO=1 (or via the
// "Run self-test" button) and reports each result to Rust, which prints [PASS]/[FAIL].
window.runSelfTest = async function (list) {
  const { invoke } = window.__TAURI__.core
  const { listen, emit } = window.__TAURI__.event
  const { getCurrentWindow } = window.__TAURI__.window
  const { WebviewWindow, getAllWebviewWindows } = window.__TAURI__.webviewWindow
  let failures = 0
  const assert = (c, m) => {
    if (!c) throw new Error(m)
  }
  // Thrown by checks that can't run on this platform (reported as [SKIP], not a failure).
  class Skip extends Error {}
  const skip = (reason) => {
    throw new Skip(reason)
  }
  // `offscreen`, `wayland` or `xcb`: offscreen has no GPU (WebGL) and fakes window management;
  // Wayland doesn't let clients position windows.
  const platform = await invoke('qpa_platform')
  // QT_APP_ONLY=<regex> runs only the matching checks (a development aid).
  const only = await invoke('selftest_only').then((o) => o && new RegExp(o))
  // Checks for known bugs only run with QT_APP_BUGS=1 (some hang the app); otherwise they
  // are reported as [KNOWN-BUG] with the reason.
  const runBugs = await invoke('known_bugs')
  async function knownBug(name, reason, fn, timeout) {
    if (only && !only.test(name)) return
    if (runBugs) return check(`${name} (known bug: ${reason})`, fn, timeout)
    await invoke('report', {
      name,
      ok: false,
      detail: reason,
      bug: true
    }).catch(() => {})
  }
  async function check(name, fn, timeout = 8000) {
    if (only && !only.test(name)) return
    let ok = true,
      skipped = false,
      detail
    try {
      detail = String(
        await Promise.race([
          fn(),
          new Promise((_, rej) =>
            setTimeout(() => rej(new Error('timeout')), timeout)
          )
        ])
      )
    } catch (e) {
      if (e instanceof Skip) {
        skipped = true
        detail = `skipped on ${platform}: ${e.message}`
      } else {
        ok = false
        failures++
        detail = String((e && e.stack) || e)
      }
    }
    const li = document.createElement('li')
    li.className = skipped ? 'skip' : ok ? 'pass' : 'fail'
    li.textContent = `${skipped ? '–' : ok ? '✓' : '✗'} ${name}: ${detail}`
    list.appendChild(li)
    await invoke('report', { name, ok, detail, skipped }).catch(() => {})
  }

  await check('page origin', () => location.href)
  await check('engine', () => navigator.userAgent)
  await check('webview_version()', async () => {
    const v = await invoke('engine')
    assert(v.includes('Chromium'), v)
    return v
  })
  await check('invoke (sync command)', async () => {
    const r = await invoke('greet', { name: 'Qt' })
    assert(r.includes('Hello, Qt'), r)
    return r
  })
  await check('invoke (async command)', async () => {
    const r = await invoke('slow_add', { a: 2, b: 40 })
    assert(r === 42, r)
    return r
  })
  await check('invoke (error)', async () => {
    try {
      await invoke('fail')
    } catch (e) {
      assert(String(e).includes('expected failure'), e)
      return 'rejected with: ' + e
    }
    throw new Error('command did not fail')
  })
  await check('IPC routed to the right webview', async () => {
    const label = await invoke('whoami')
    assert(label === 'main', label)
    return label
  })
  await check(
    'event Rust -> JS',
    () =>
      new Promise(async (res) => {
        const un = await listen('from-rust', (e) => {
          un()
          res('received ' + JSON.stringify(e.payload))
        })
        await invoke('ping_event')
      })
  )
  await check('event JS -> Rust', async () => {
    await emit('from-js', { hello: 'rust' })
    return 'emitted'
  })
  await check('window API', async () => {
    const w = getCurrentWindow()
    const size = await w.innerSize()
    const scale = await w.scaleFactor()
    await w.setTitle('Tauri on QtWebEngine ✓')
    const title = await w.title()
    assert(title.endsWith('✓'), title)
    return `${size.width}x${size.height} @${scale}x, title="${title}", focused=${await w.isFocused()}`
  })
  await check('custom protocol + byte ranges', async () => {
    const url = window.__TAURI__.core.convertFileSrc('data', 'stream')
    const full = await (await fetch(url)).text()
    const r = await fetch(url, { headers: { Range: 'bytes=10-19' } })
    const part = await r.text()
    assert(r.status === 206, 'status ' + r.status)
    assert(
      part === full.slice(10, 20),
      `got "${part}" want "${full.slice(10, 20)}"`
    )
    return `${url}: ${full.length} bytes, range -> ${r.status} "${part}" (${r.headers.get('content-range')})`
  })
  await check('second window (with parent): create, IPC, close', async () => {
    const ready = new Promise(async (res) => {
      const un = await listen('child-ready', (e) => {
        un()
        res(e.payload)
      })
    })
    // `parent` used to fail on Linux without GTK windows; Qt windows are made transient instead.
    const w = new WebviewWindow('second', {
      url: 'index.html?child',
      title: 'Second',
      width: 420,
      height: 320,
      parent: 'main'
    })
    await new Promise((res, rej) => {
      w.once('tauri://created', res)
      w.once('tauri://error', (e) => rej(new Error(JSON.stringify(e.payload))))
    })
    const label = await ready
    assert(label === 'second', 'child reported ' + label)
    await w.close()
    await new Promise((r) => setTimeout(r, 400))
    const labels = (await getAllWebviewWindows()).map((w) => w.label)
    assert(!labels.includes('second'), 'still open: ' + labels)
    return `child identified as "${label}", remaining windows: ${labels}`
  })
  await check('window.open() popup connected to its opener', async () => {
    const reply = new Promise((res) =>
      window.addEventListener('message', (e) => {
        if (e.data && e.data.popup) res(e.data.popup)
      })
    )
    const popup = window.open(
      'index.html?popup',
      '_blank',
      'width=420,height=320'
    )
    assert(popup, 'window.open returned null')
    const label = await reply
    assert(label.startsWith('popup-'), label)
    await (await WebviewWindow.getByLabel(label)).close()
    return `popup "${label}" posted back through window.opener`
  })
  await check('history: can_go_back / go_back', async () => {
    history.pushState({}, '', '#step')
    assert(
      (await invoke('can_go_back')) === true,
      'can_go_back after pushState'
    )
    const popped = new Promise((res) =>
      window.addEventListener('popstate', res, { once: true })
    )
    await invoke('go_back')
    await popped
    assert(location.hash === '', 'hash after go_back: ' + location.hash)
    return 'pushState -> can_go_back=true -> go_back restored ' + location.href
  })
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
  // Registers a listener for `name` and returns `{ next }`, a promise of its next payload
  // (wrapped so that awaiting this function doesn't also wait for the event).
  async function nextEvent(name) {
    let resolve
    const next = new Promise((r) => (resolve = r))
    const un = await listen(name, (e) => {
      un()
      resolve(e.payload)
    })
    return { next }
  }
  await check('menubar built from the Tauri menu (Qt)', async () => {
    const tree = await invoke('menu_tree')
    for (const want of [
      'File',
      'New <Ctrl+N>',
      '[ ] Auto save <Ctrl+Shift+S>',
      'Close Window',
      'Edit',
      'Copy (Ctrl+C)',
      'Paste (Ctrl+V)',
      'Help',
      'About'
    ])
      assert(tree.includes(want), `missing "${want}" in:\n${tree}`)
    return tree
      .trim()
      .split('\n')
      .map((l) => l.trim())
      .join(' | ')
  })
  await check('menu click reaches on_menu_event', async () => {
    const got = (await nextEvent('menu')).next
    assert(await invoke('menu_trigger', { path: 'File/New' }), 'item not found')
    const id = await got
    assert(id === 'new', id)
    return 'File/New -> menu event "' + id + '"'
  })
  await check('check item toggles', async () => {
    const got = (await nextEvent('menu')).next
    assert(
      await invoke('menu_trigger', { path: 'File/Auto save' }),
      'item not found'
    )
    assert((await got) === 'autosave')
    await sleep(100)
    const tree = await invoke('menu_trigger', { path: 'File/Auto save' })
      .then(() => sleep(100))
      .then(() => invoke('menu_tree'))
    assert(
      tree.includes('[ ] Auto save'),
      'expected unchecked after second toggle:\n' + tree
    )
    return 'toggled on and off, menubar reflects state'
  })
  await check('set_text updates the live menubar', async () => {
    await invoke('rename_menu_item', { id: 'new', text: '&New file' })
    await sleep(100)
    const tree = await invoke('menu_tree')
    assert(tree.includes('New file <Ctrl+N>'), tree)
    return 'renamed to "New file"'
  })
  await check('swapping the window menu replaces the menubar', async () => {
    const { Menu } = window.__TAURI__.menu
    const swap = await Menu.new({
      items: [
        { text: 'Swapped', items: [{ id: 'swap-only', text: 'Only item' }] }
      ]
    })
    const prev = await swap.setAsWindowMenu()
    await sleep(100) // let Qt delete the old menubar
    let tree = await invoke('menu_tree')
    assert(
      tree.includes('Only item') && !tree.includes('Auto save'),
      'after swap:\n' + tree
    )
    await prev.setAsWindowMenu()
    await sleep(100)
    tree = await invoke('menu_tree')
    assert(
      tree.includes('Auto save') && !tree.includes('Only item'),
      'after restore:\n' + tree
    )
    return 'swapped to a new menu and back'
  })
  await check('context menu from JS (Menu.popup)', async () => {
    const { Menu } = window.__TAURI__.menu
    const menu = await Menu.new({
      items: [
        { id: 'ctx-reload', text: 'Reload' },
        { id: 'ctx-inspect', text: 'Inspect', accelerator: 'F12' }
      ]
    })
    await menu.popup()
    let popup = ''
    for (let i = 0; i < 40 && !popup; i++) {
      await sleep(50)
      popup = await invoke('take_popup')
    }
    assert(
      popup.includes('Reload') && popup.includes('Inspect (F12)'),
      'popup: ' + JSON.stringify(popup)
    )
    return popup.trim().split('\n').join(' | ')
  })
  await check('WebRTC data channel', async () => {
    const a = new RTCPeerConnection(),
      b = new RTCPeerConnection()
    a.onicecandidate = (e) => e.candidate && b.addIceCandidate(e.candidate)
    b.onicecandidate = (e) => e.candidate && a.addIceCandidate(e.candidate)
    const dc = a.createDataChannel('t')
    const got = new Promise(
      (res) =>
        (b.ondatachannel = (e) => (e.channel.onmessage = (m) => res(m.data)))
    )
    await a.setLocalDescription(await a.createOffer())
    await b.setRemoteDescription(a.localDescription)
    await b.setLocalDescription(await b.createAnswer())
    await a.setRemoteDescription(b.localDescription)
    dc.onopen = () => dc.send('hello over WebRTC')
    return await got
  })
  await check('codecs', () => {
    const v = document.createElement('video')
    return [
      'video/mp4; codecs="avc1.42E01E"',
      'video/webm; codecs="vp9"',
      'video/mp4; codecs="av01.0.05M.08"'
    ]
      .map(
        (t) =>
          `${t.split(';')[0]} ${t.match(/codecs="([^".]+)/)[1]}=${v.canPlayType(t) || 'no'}`
      )
      .join(', ')
  })
  await check('WebGL2', () => {
    const gl = document.createElement('canvas').getContext('webgl2')
    if (!gl && platform === 'offscreen') skip('no GPU context offscreen')
    assert(gl, 'no webgl2')
    const ext = gl.getExtension('WEBGL_debug_renderer_info')
    return ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : 'available'
  })
  await window.tauriApiChecks({
    check,
    knownBug,
    assert,
    skip,
    sleep,
    platform
  })
  await invoke('done', { failures })
}
