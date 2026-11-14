// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Interactive showcase: every control calls a Tauri JS API or a command in src/main.rs and logs
// the outcome. Rust-side hooks (window events, navigation, downloads, ...) log here too.
const T = window.__TAURI__
const { invoke, convertFileSrc } = T.core
const { listen, emit } = T.event
const {
  getCurrentWindow,
  getAllWindows,
  availableMonitors,
  cursorPosition,
  UserAttentionType
} = T.window
const { LogicalSize, LogicalPosition } = T.dpi
const { getCurrentWebview, getAllWebviews } = T.webview
const { WebviewWindow } = T.webviewWindow
const { Menu } = T.menu

const $ = (id) => document.getElementById(id)
const win = getCurrentWindow()
const webview = getCurrentWebview()

// --- event log ----------------------------------------------------------------------------------

function log(source, message, isError = false) {
  if ($('quiet')?.checked && /: (Resized|Moved)\(/.test(message)) return
  const li = document.createElement('li')
  if (isError) li.className = 'err'
  const src = document.createElement('span')
  src.className = 'src'
  src.textContent = source
  li.append(
    src,
    typeof message === 'string' ? message : JSON.stringify(message)
  )
  $('log').prepend(li)
}

/** Run an action from a button, logging its result or error. */
function act(source, fn) {
  return async () => {
    try {
      const r = await fn()
      if (r !== undefined)
        log(source, typeof r === 'string' ? r : JSON.stringify(r))
    } catch (e) {
      log(source, String(e), true)
    }
  }
}

function on(id, source, fn) {
  $(id).addEventListener('click', act(source, fn))
}

// --- secondary pages (child webviews, second windows from the self-test, popups) ---------------

const query = location.search
const isChildWebview = webview.label.startsWith('child-')

if (query.includes('child') || query.includes('popup') || isChildWebview) {
  $('mini').hidden = false
  $('mini-title').textContent = query.includes('popup')
    ? 'Popup window'
    : isChildWebview
      ? 'Child webview'
      : 'Second window'
  $('mini-info').textContent =
    `label: ${webview.label} — ${navigator.userAgent.match(/Chrome\/[\d.]+/)}`
  // `Webview.close()` only removes the webview (its window stays, empty); popups and second
  // windows close their window, child webviews close just themselves.
  $('mini-close').onclick = () =>
    isChildWebview ? webview.close() : win.close()
  $('mini-devtools').onclick = () => invoke('devtools', { action: 'toggle' })
  // Used by the self-test.
  if (query.includes('child')) {
    // `probe` events targeted at this webview are answered with what the page sees (the
    // self-test checks event targeting, zoom and IPC after reparenting with it).
    webview
      .listen('probe', async (e) =>
        emit('probe-reply', {
          label: await invoke('whoami'),
          payload: e.payload,
          dpr: window.devicePixelRatio,
          width: window.innerWidth,
          height: window.innerHeight
        })
      )
      .then(() => invoke('whoami'))
      .then((label) => emit('child-ready', label))
  }
  if (query.includes('popup') && window.opener)
    invoke('whoami').then((label) =>
      window.opener.postMessage({ popup: label }, '*')
    )
} else {
  $('app').hidden = false
  setup().catch((e) => log('setup', String(e), true))
}

async function setup() {
  await listen('showcase-log', (e) => log(e.payload.source, e.payload.message))
  $('clear-log').onclick = () => ($('log').textContent = '')

  // --- about ------------------------------------------------------------------------------------
  const engine = await invoke('engine')
  $('engine').textContent = `${engine} · window "${win.label}"`
  const facts = {
    App: `${await T.app.getName()} ${await T.app.getVersion()} (${await T.app.getIdentifier()})`,
    Tauri: await T.app.getTauriVersion(),
    'Webview engine': engine,
    'User agent': navigator.userAgent,
    Origin: location.origin
  }
  for (const [k, v] of Object.entries(facts)) {
    const dt = document.createElement('dt')
    dt.textContent = k
    const dd = document.createElement('dd')
    dd.textContent = v
    $('facts').append(dt, dd)
  }
  on('monitors', 'monitors', async () =>
    (await availableMonitors())
      .map(
        (m) => `${m.name} ${m.size.width}×${m.size.height} @${m.scaleFactor}x`
      )
      .join(' | ')
  )
  on('cursor', 'cursor', async () => {
    const p = await cursorPosition()
    return `${Math.round(p.x)}, ${Math.round(p.y)}`
  })

  // --- commands & events ------------------------------------------------------------------------
  on('greet', 'invoke', () => invoke('greet', { name: $('greet-name').value }))
  on(
    'add',
    'invoke',
    async () => `slow_add(2, 40) = ${await invoke('slow_add', { a: 2, b: 40 })}`
  )
  on('error', 'invoke', () => invoke('fail'))
  on('emit-rust', 'event', async () => {
    await emit('from-js', { hello: 'rust', at: Date.now() })
    return 'emitted "from-js" (Rust prints it to stdout)'
  })
  await listen('from-rust', (e) =>
    log('event', `from Rust: ${JSON.stringify(e.payload)}`)
  )
  on('ping', 'event', () => invoke('ping_event'))
  on(
    'eval-btn',
    'eval',
    async () =>
      `result: ${await invoke('eval_js', { code: $('eval-code').value })}`
  )

  // --- window -----------------------------------------------------------------------------------
  $('title').value = await win.title()
  on('set-title', 'window', async () => {
    await win.setTitle($('title').value)
    return `title = ${$('title').value}`
  })
  document.querySelectorAll('[data-win]').forEach((b) =>
    b.addEventListener(
      'click',
      act('window', () => win[b.dataset.win]())
    )
  )
  on('fullscreen', 'window', async () =>
    win.setFullscreen(!(await win.isFullscreen()))
  )
  on('hide-show', 'window', async () => {
    await win.hide()
    setTimeout(() => win.show().then(() => win.setFocus()), 2000)
  })
  on('attention', 'window', () =>
    win.requestUserAttention(UserAttentionType.Critical)
  )
  const toggle = (id, fn) =>
    $(id).addEventListener(
      'change',
      act('window', () => fn($(id).checked))
    )
  toggle('on-top', (v) => win.setAlwaysOnTop(v))
  toggle('decorations', (v) => win.setDecorations(v))
  toggle('resizable', (v) => win.setResizable(v))
  toggle('prevent-close', (v) => invoke('set_prevent_close', { prevent: v }))
  toggle('cursor-visible', (v) => win.setCursorVisible(v))
  document.querySelectorAll('[data-size]').forEach((b) =>
    b.addEventListener(
      'click',
      act('window', () => {
        const [w, h] = b.dataset.size.split('x').map(Number)
        return win.setSize(new LogicalSize(w, h))
      })
    )
  )
  on('min-size', 'window', () => win.setMinSize(new LogicalSize(600, 400)))
  on('position', 'window', () => win.setPosition(new LogicalPosition(100, 100)))
  $('cursor-icon').addEventListener(
    'change',
    act('window', () => win.setCursorIcon($('cursor-icon').value))
  )
  $('theme').addEventListener(
    'change',
    act('theme', async () => {
      await invoke('set_theme', { theme: $('theme').value })
      return `app theme: ${$('theme').value} (also try View → Theme)`
    })
  )
  $('progress').addEventListener(
    'change',
    act('window', () => {
      const progress = Number($('progress').value)
      return win.setProgressBar({
        status: progress ? 'normal' : 'none',
        progress
      })
    })
  )
  $('badge').addEventListener(
    'change',
    act('window', () =>
      win.setBadgeCount(Number($('badge').value) || undefined)
    )
  )

  // --- webview & devtools -----------------------------------------------------------------------
  const devtools = (action) => async () =>
    `DevTools open: ${await invoke('devtools', { action })}`
  on('devtools-toggle', 'devtools', devtools('toggle'))
  on('dt-open', 'devtools', devtools('open'))
  on('dt-close', 'devtools', devtools('close'))
  $('zoom').addEventListener(
    'change',
    act('webview', () => {
      const factor = Number($('zoom').value) / 100
      return invoke('zoom', { factor }).then(() => `zoom ${factor}`)
    })
  )
  $('bg').addEventListener(
    'change',
    act('webview', () => {
      const hex = $('bg').value
      const rgb = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16))
      return webview
        .setBackgroundColor([...rgb, 255])
        .then(() => `background ${hex} (visible while loading)`)
    })
  )
  const showHistory = (s) =>
    ($('history-state').textContent =
      `canGoBack=${s.canGoBack} canGoForward=${s.canGoForward} ${new URL(s.url).hash || ''}`)
  on('push', 'history', async () => {
    history.pushState({}, '', `#step-${Date.now() % 1000}`)
    showHistory(await invoke('history', { action: 'state' }))
  })
  document.querySelectorAll('[data-history]').forEach((b) =>
    b.addEventListener(
      'click',
      act('history', async () => {
        const s = await invoke('history', { action: b.dataset.history })
        showHistory(s)
        return `${b.dataset.history}: ${s.url}`
      })
    )
  )
  on('navigate', 'webview', () =>
    invoke('navigate', { url: $('nav-url').value })
  )
  on('print', 'webview', () => invoke('print_page'))
  on('clear-data', 'webview', async () => {
    await invoke('clear_browsing_data')
    return 'cache, cookies and visited links cleared'
  })
  on('set-cookie', 'cookies', async () => {
    const value = `v${Date.now() % 10000}`
    await invoke('set_cookie', { name: 'showcase', value })
    return `set showcase=${value}`
  })
  on('list-cookies', 'cookies', async () => {
    const list = await invoke('cookies')
    return list.length ? list.join(' | ') : 'no cookies'
  })

  // --- windows & webviews -----------------------------------------------------------------------
  on(
    'new-window',
    'windows',
    async () => `opened ${await invoke('open_window', { withParent: false })}`
  )
  on(
    'new-window-parent',
    'windows',
    async () =>
      `opened ${await invoke('open_window', { withParent: true })} (transient for main)`
  )
  on('window-open', 'windows', () => {
    const popup = window.open(
      'index.html?popup',
      '_blank',
      'width=480,height=360'
    )
    return popup
      ? 'window.open() returned a window'
      : 'window.open() returned null'
  })
  on(
    'add-child',
    'webviews',
    async () =>
      `added ${await invoke('add_child_webview', { url: $('child-url').value })} (follows the window size)`
  )
  on(
    'close-children',
    'webviews',
    async () =>
      `closed ${await invoke('close_child_webviews')} child webview(s)`
  )
  on('list-windows', 'windows', async () => {
    const windows = (await getAllWindows()).map((w) => w.label)
    const webviews = (await getAllWebviews()).map((w) => w.label)
    return `windows: ${windows.join(', ')} · webviews: ${webviews.join(', ')}`
  })

  // --- menus ------------------------------------------------------------------------------------
  await listen('menu', (e) => {
    if (e.payload === 'devtools') log('devtools', 'toggled from the View menu')
  })
  $('context-zone').addEventListener('contextmenu', async (e) => {
    e.preventDefault()
    try {
      const action = (id) => log('context-menu', `clicked "${id}"`)
      const menu = await Menu.new({
        items: [
          {
            id: 'ctx-hello',
            text: 'Say hello',
            accelerator: 'CmdOrCtrl+H',
            action
          },
          { id: 'ctx-check', text: 'A check item', checked: true, action },
          { item: 'Separator' },
          { item: 'Copy' },
          { item: 'Paste' },
          { item: 'Separator' },
          {
            text: 'Submenu',
            items: [{ id: 'ctx-sub', text: 'Nested item', action }]
          },
          {
            id: 'ctx-devtools',
            text: 'Inspect',
            action: () => invoke('devtools', { action: 'open' })
          }
        ]
      })
      await menu.popup()
    } catch (err) {
      log('context-menu', String(err), true)
    }
  })
  on('rename-btn', 'menu', async () => {
    await invoke('rename_menu_item', { id: 'new', text: $('rename').value })
    return `File → New is now "${$('rename').value.replace('&', '')}"`
  })
  on('menu-tree', 'menu', async () =>
    (await invoke('menu_tree'))
      .trim()
      .split('\n')
      .map((l) => l.trim())
      .join(' | ')
  )

  // --- media ------------------------------------------------------------------------------------
  let stream = null
  const play = (s) => {
    stream?.getTracks().forEach((t) => t.stop())
    stream = s
    $('video').srcObject = s
  }
  on('cam', 'media', async () => {
    play(
      await navigator.mediaDevices.getUserMedia({ video: true, audio: true })
    )
    return (
      'camera + microphone: '
      + stream
        .getTracks()
        .map((t) => t.label || t.kind)
        .join(', ')
    )
  })
  on('screen', 'media', async () => {
    play(
      await navigator.mediaDevices.getDisplayMedia({ video: true, audio: true })
    )
    return (
      'screen: '
      + stream
        .getVideoTracks()
        .map((t) => t.label)
        .join(', ')
    )
  })
  on('stop-media', 'media', () => play(null))
  on('video-fs', 'media', () => $('video').requestFullscreen())
  on('webrtc', 'webrtc', async () => {
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
    dc.onopen = () => dc.send('hello over a WebRTC data channel')
    return await got
  })
  on('codecs', 'media', () => {
    const v = document.createElement('video')
    return [
      'video/mp4; codecs="avc1.42E01E"',
      'video/webm; codecs="vp9"',
      'video/mp4; codecs="av01.0.05M.08"',
      'audio/mp4; codecs="mp4a.40.2"',
      'audio/webm; codecs="opus"'
    ]
      .map(
        (t) => `${t.match(/codecs="([^".]+)/)[1]}=${v.canPlayType(t) || 'no'}`
      )
      .join(', ')
  })
  on('webgl', 'media', () => {
    const gl = document.createElement('canvas').getContext('webgl2')
    if (!gl) throw new Error('no WebGL2')
    const ext = gl.getExtension('WEBGL_debug_renderer_info')
    return ext
      ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)
      : 'WebGL2 available'
  })

  // --- files & protocols ------------------------------------------------------------------------
  on('download', 'download', () => {
    const a = document.createElement('a')
    a.href = URL.createObjectURL(
      new Blob([`Downloaded from Tauri on QtWebEngine at ${new Date()}\n`], {
        type: 'text/plain'
      })
    )
    a.download = 'tauri-qt-showcase.txt'
    a.click()
    return 'download started (see the download events)'
  })
  on('stream', 'protocol', async () => {
    const url = convertFileSrc('data', 'stream')
    const r = await fetch(url, { headers: { Range: 'bytes=10-19' } })
    return `${url} → ${r.status} ${r.headers.get('content-range')} "${await r.text()}"`
  })
  on('crash', 'renderer', async () => {
    // chrome://kill terminates the renderer at once (chrome://crash leaves a hung process that
    // QtWebEngine only reports at exit).
    await invoke('navigate', { url: 'chrome://kill' })
    return 'renderer killed: on_web_content_process_terminate logs it and reloads the page'
  })

  // --- self-test --------------------------------------------------------------------------------
  const runSelfTest = async () => {
    $('selftest-section').hidden = false
    $('results').textContent = ''
    $('selftest-section').scrollIntoView({ behavior: 'smooth' })
    await window.runSelfTest($('results'))
  }
  $('run-selftest').onclick = () =>
    runSelfTest().catch((e) => log('self-test', String(e), true))
  const mode = win.label === 'main' ? await invoke('auto_mode') : null
  if (mode === 'smoke') return smokeTest()
  // QT_APP_AUTO=crash: kill the renderer once; after on_web_content_process_terminate reloads
  // the page, finish.
  if (mode === 'crash') {
    if (sessionStorage.getItem('crashed')) {
      let failures = 0
      const report = async (name, ok, detail) => {
        if (!ok) failures++
        await invoke('report', {
          name: `crash: ${name}`,
          ok,
          detail: String(detail)
        })
      }
      const n = await invoke('terminations')
      await report(
        'on_web_content_process_terminate fired once',
        n === 1,
        `${n} termination(s)`
      )
      await report(
        'the hook reloaded the page in the same webview',
        webview.label === 'main' && location.protocol === 'tauri:',
        location.href
      )
      const greeting = await invoke('greet', { name: 'after crash' }).catch(
        (e) => 'error: ' + e
      )
      await report(
        'IPC works in the new renderer',
        greeting.includes('Hello, after crash'),
        greeting
      )
      return invoke('done', { failures })
    }
    sessionStorage.setItem('crashed', '1')
    return invoke('navigate', { url: 'chrome://kill' })
  }
  if (mode) await runSelfTest()

  log(
    'ready',
    'Showcase loaded — try the buttons, the menubar and right-click the context zone'
  )
}

// QT_APP_AUTO=smoke: click through the showcase controls and check the event log for each
// expected result (a development aid; results go to stdout through the `report` command).
async function smokeTest() {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
  const platform = await invoke('qpa_platform')
  // Show resize/move events in the log: some steps wait for them.
  $('quiet').checked = false
  // `expect` is a regex for the new log lines, or a predicate on them. `skip(lines)` returns a
  // reason when the step can't work on this platform (reported as [SKIP]).
  async function step(name, run, expect, timeout = 6000, skip) {
    const before = $('log').children.length
    let ok = false,
      detail = '',
      skipped = false
    if (typeof skip === 'string') {
      await invoke('report', {
        name: `showcase: ${name}`,
        ok: true,
        detail: skip,
        skipped: true
      })
      return true
    }
    try {
      await run()
      for (let t = 0; t < timeout && !ok; t += 100) {
        await sleep(100)
        const fresh = [...$('log').children]
          .slice(0, $('log').children.length - before)
          .map((li) => li.innerText)
          .join(' || ')
        detail = fresh
        ok = typeof expect === 'function' ? expect(fresh) : expect.test(fresh)
      }
      const reason = !ok && skip && skip(detail)
      if (reason) {
        skipped = true
        detail = `skipped on ${platform}: ${reason}`
      }
    } catch (e) {
      detail = String(e)
    }
    await invoke('report', {
      name: `showcase: ${name}`,
      ok,
      detail: detail.slice(0, 300),
      skipped
    })
    return ok || skipped
  }
  // For actions that log nothing on success: no error within the first poll.
  const noError = (lines) => !/Error|error/.test(lines)
  const closeWindows = (prefix) => async () => {
    for (const w of await getAllWindows())
      if (w.label.startsWith(prefix)) await w.close()
  }
  const onWayland =
    platform === 'wayland'
      ? 'Wayland clients cannot position windows'
      : undefined
  const click = (id) => () => $(id).click()
  const change = (id, value) => () => {
    const el = $(id)
    if (el.type === 'checkbox') el.checked = value
    else el.value = value
    el.dispatchEvent(new Event('change'))
  }
  let failures = 0
  const steps = [
    ['about: monitors', click('monitors'), /monitors.*@/],
    ['about: cursor position', click('cursor'), /cursor\s*-?\d+, -?\d+/],
    ['commands: greet', click('greet'), /Hello, Qt/],
    ['commands: error', click('error'), /expected failure/],
    ['events: Rust -> JS', click('ping'), /from Rust: "pong"/],
    ['eval from Rust', click('eval-btn'), /result: \[/],
    ['devtools: open', click('dt-open'), /DevTools open: true/],
    ['devtools: close', click('dt-close'), /DevTools open: false/],
    ['webview: zoom 150%', change('zoom', 150), /zoom 1\.5/],
    ['webview: zoom 100%', change('zoom', 100), /zoom 1\b/],
    [
      'history: pushState + back',
      async () => {
        $('push').click()
        await sleep(300)
        document.querySelector('[data-history=back]').click()
      },
      /back: tauri:/
    ],
    ['cookies: set', click('set-cookie'), /set showcase=/],
    [
      'cookies: list',
      async () => {
        await sleep(500)
        $('list-cookies').click()
      },
      /showcase=v/
    ],
    [
      'navigation: blocked link',
      click('blocked'),
      /navigation.*blocked https:\/\/blocked\.example\.com/
    ],
    ['download', click('download'), /finished .*success: true/],
    ['custom protocol range', click('stream'), /206 bytes 10-19\/1000/],
    ['child webview: add', click('add-child'), /added child-\d/],
    [
      'windows: list',
      async () => {
        await sleep(800)
        $('list-windows').click()
      },
      /webviews:.*child-\d/
    ],
    ['child webview: close', click('close-children'), /closed 1 child/],
    ['window: new with parent', click('new-window-parent'), /opened window-\d/],
    [
      'window: close it',
      async () => {
        await sleep(1000)
        const w = await WebviewWindow.getByLabel('window-0')
        await w.close()
      },
      /window-0: (close requested|Destroyed)/
    ],
    [
      'window: title',
      async () => {
        $('title').value = 'Smoke ✓'
        $('set-title').click()
      },
      /title = Smoke/
    ],
    ['window: always on top', change('on-top', true), /^$|window/, 800],
    [
      'window: prevent close',
      async () => {
        change('prevent-close', true)()
        await sleep(200)
        await win.close()
      },
      /main: close requested \(prevented\)/
    ],
    [
      'window: allow close again',
      change('prevent-close', false),
      /^$|window/,
      300
    ],
    ['theme: dark', change('theme', 'dark'), /app theme: dark/],
    ['theme: system', change('theme', 'system'), /app theme: system/],
    ['menu: rename item', click('rename-btn'), /File → New is now/],
    ['menu: tree', click('menu-tree'), /View \| Toggle DevTools <F12>/],
    [
      'menu: View/Toggle DevTools',
      () => invoke('menu_trigger', { path: 'View/Toggle DevTools' }),
      /toggled from the View menu/
    ],
    [
      'menu: View/Zoom In',
      () => invoke('menu_trigger', { path: 'View/Zoom In' }),
      /clicked "zoom-in"/
    ],
    [
      'menu: View/Actual Size',
      () => invoke('menu_trigger', { path: 'View/Actual Size' }),
      /clicked "zoom-reset"/
    ],
    ['media: WebRTC', click('webrtc'), /hello over a WebRTC/],
    ['media: codecs', click('codecs'), /avc1=probably/],
    [
      'media: WebGL',
      click('webgl'),
      /media(ANGLE|WebGL2 available)/,
      3000,
      (lines) =>
        platform === 'offscreen'
        && /no WebGL2/.test(lines)
        && 'no GPU context offscreen'
    ],
    [
      'window: size 800x600',
      () => document.querySelector('[data-size="800x600"]').click(),
      /main: Resized\(PhysicalSize \{ width: \d+, height: \d+ \}\)/
    ],
    [
      'window: size 1100x820',
      () => document.querySelector('[data-size="1100x820"]').click(),
      /main: Resized/
    ],
    ['window: fullscreen on', click('fullscreen'), /main: Resized/],
    [
      'window: fullscreen off',
      async () => {
        await sleep(300)
        $('fullscreen').click()
      },
      /main: Resized/
    ],
    [
      'window: toggle maximize',
      () => document.querySelector('[data-win="toggleMaximize"]').click(),
      /main: Resized/
    ],
    [
      'window: toggle maximize back',
      async () => {
        await sleep(300)
        document.querySelector('[data-win="toggleMaximize"]').click()
      },
      /main: Resized/
    ],
    ['window: position', click('position'), /main: Moved/, 3000, onWayland],
    [
      'window: center',
      () => document.querySelector('[data-win="center"]').click(),
      noError,
      300
    ],
    ['window: min size', click('min-size'), noError, 300],
    ['window: decorations off', change('decorations', false), noError, 300],
    ['window: decorations on', change('decorations', true), noError, 300],
    ['window: resizable off', change('resizable', false), noError, 300],
    ['window: resizable on', change('resizable', true), noError, 300],
    ['window: always on top off', change('on-top', false), noError, 300],
    ['window: cursor icon', change('cursor-icon', 'pointer'), noError, 300],
    ['window: request attention', click('attention'), noError, 300],
    ['window: progress bar', change('progress', 50), noError, 300],
    ['window: badge', change('badge', 3), noError, 300],
    [
      'webview: background color',
      change('bg', '#336699'),
      /background #336699/
    ],
    [
      'history: forward',
      async () => {
        $('push').click()
        await sleep(300)
        document.querySelector('[data-history=back]').click()
        await sleep(300)
        document.querySelector('[data-history=forward]').click()
      },
      /forward: tauri:.*#step/
    ],
    [
      'windows: window.open() popup',
      click('window-open'),
      /new-window.*-> popup-\d/
    ],
    [
      'windows: close the popup',
      async () => {
        await sleep(800)
        await closeWindows('popup-')()
      },
      /popup-\d: Destroyed/
    ],
    ['windows: new window', click('new-window'), /opened window-\d/],
    [
      'windows: close it',
      async () => {
        await sleep(800)
        await closeWindows('window-')()
      },
      /window-\d: Destroyed/
    ]
  ]
  for (const [name, run, expect, timeout, skip] of steps)
    if (!(await step(name, run, expect, timeout, skip))) failures++
  // Close the DevTools window the View menu opened.
  await invoke('devtools', { action: 'close' })
  await invoke('done', { failures })
}
