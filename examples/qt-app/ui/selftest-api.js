// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Self-test, part 2: the Tauri JS API end to end (app, path, monitors, windows, webviews,
// events, IPC, custom protocols, tray). Called by `runSelfTest` in selftest.js with its helpers.
window.tauriApiChecks = async function ({
  check,
  knownBug,
  assert,
  skip,
  sleep,
  platform
}) {
  const T = window.__TAURI__
  const { invoke, Channel, convertFileSrc } = T.core
  const { listen, once, emit, emitTo } = T.event
  const { getCurrentWindow, getAllWindows, Window } = T.window
  const { LogicalSize, LogicalPosition, PhysicalSize, PhysicalPosition } = T.dpi
  const { Webview, getCurrentWebview, getAllWebviews } = T.webview
  const { WebviewWindow } = T.webviewWindow
  const main = getCurrentWindow()
  const offscreen = platform === 'offscreen'
  const wayland = platform === 'wayland'

  /** Poll `fn` until it returns a truthy value (returned) or `ms` elapse (throws `what`). */
  async function waitFor(what, fn, ms = 4000) {
    const end = Date.now() + ms
    let last
    while (Date.now() < end) {
      last = await fn()
      if (last) return last
      await sleep(50)
    }
    throw new Error(
      `timed out waiting for ${what} (last: ${JSON.stringify(last)})`
    )
  }
  /** Collect the payloads of `event` (any target) until the returned `stop()` is called. */
  async function collect(event) {
    const got = []
    const un = await listen(event, (e) => got.push(e.payload))
    return { got, stop: un }
  }
  /** Resolve when `obj` reports `tauri://created`, reject on `tauri://error`. */
  const created = (obj) =>
    new Promise((res, rej) => {
      obj.once('tauri://created', res)
      obj.once('tauri://error', (e) =>
        rej(new Error(JSON.stringify(e.payload)))
      )
    })
  /** Wait for the page with webview label `label` (an `index.html?child` page) to be ready. */
  async function childReady(label) {
    let un
    const ready = new Promise(async (res) => {
      un = await listen('child-ready', (e) => e.payload === label && res())
    })
    return { ready: ready.then(() => un()) }
  }
  /** Open an `index.html?child` window and wait until its page is up. */
  async function openWindow(label, options = {}) {
    const { ready } = await childReady(label)
    const w = new WebviewWindow(label, {
      url: 'index.html?child',
      title: label,
      width: 420,
      height: 320,
      ...options
    })
    await created(w)
    await ready
    return w
  }
  /** Ask the `?child` page in webview `label` for its view of the world (see app.js). */
  async function probe(label, payload = null) {
    const replies = await collect('probe-reply')
    try {
      await emitTo({ kind: 'Webview', label }, 'probe', payload)
      return await waitFor(`probe reply from ${label}`, () =>
        replies.got.find((r) => r.label === label)
      )
    } finally {
      replies.stop()
    }
  }
  const labels = async () => (await getAllWindows()).map((w) => w.label)
  const windowFlags = async (w) => ({
    title: await w.title(),
    resizable: await w.isResizable(),
    decorated: await w.isDecorated(),
    onTop: await w.isAlwaysOnTop(),
    minimizable: await w.isMinimizable(),
    maximizable: await w.isMaximizable(),
    closable: await w.isClosable(),
    visible: await w.isVisible()
  })
  // label -> label of the window hosting it.
  const webviewHosts = () => invoke('webview_hosts')
  const near = (a, b, tol = 2) => Math.abs(a - b) <= tol

  await check(
    'webview: getAllWebviews reports each webview with its window',
    async () => {
      const all = await T.webview.getAllWebviews()
      const main = all.find((w) => w.label === 'main')
      assert(
        main && main.window && main.window.label === 'main',
        JSON.stringify(all.map((w) => [w.label, w.window && w.window.label]))
      )
      return all.map((w) => `${w.label}@${w.window.label}`).join(', ')
    }
  )

  // --- app / path / monitors ----------------------------------------------------------------
  await check('app: name, version, identifier, tauri version', async () => {
    const [name, version, id, tauri] = await Promise.all([
      T.app.getName(),
      T.app.getVersion(),
      T.app.getIdentifier(),
      T.app.getTauriVersion()
    ])
    assert(
      name === 'qt-app' && version === '0.1.0' && id === 'dev.example.qtapp',
      `${name} ${version} ${id}`
    )
    assert(tauri.startsWith('3.'), 'tauri ' + tauri)
    return `${name} ${version} (${id}), tauri ${tauri}`
  })
  await check('app: defaultWindowIcon', async () => {
    const icon = await T.app.defaultWindowIcon()
    assert(icon, 'no default icon')
    const { width, height } = await icon.size()
    const rgba = await icon.rgba()
    assert(
      width > 0 && rgba.length === width * height * 4,
      `${width}x${height}, ${rgba.length} bytes`
    )
    return `${width}x${height}`
  })
  await check('path: core path API', async () => {
    const data = await T.path.appDataDir()
    assert(data.endsWith('dev.example.qtapp'), data)
    const file = await T.path.join(data, 'sub', 'file.txt')
    assert((await T.path.basename(file)) === 'file.txt', file)
    assert((await T.path.extname(file)) === 'txt', 'extname')
    assert(
      (await T.path.dirname(file)) === (await T.path.join(data, 'sub')),
      'dirname'
    )
    assert(
      (await T.path.resolve(data, 'sub', '..', 'x'))
        === (await T.path.join(data, 'x')),
      'resolve'
    )
    return file
  })
  await check(
    'monitors: primary, available, current, from point, cursor',
    async () => {
      const primary = await T.window.primaryMonitor()
      const all = await T.window.availableMonitors()
      const current = await T.window.currentMonitor()
      assert(
        primary && all.length >= 1 && current,
        `primary ${!!primary}, ${all.length} monitors, current ${!!current}`
      )
      assert(
        primary.size.width > 0 && primary.scaleFactor > 0,
        JSON.stringify(primary)
      )
      const p = primary.position
      const at = await T.window.monitorFromPoint(p.x + 1, p.y + 1)
      assert(
        at && at.name === primary.name,
        `monitorFromPoint -> ${at && at.name}`
      )
      const cursor = await T.window.cursorPosition()
      assert(
        Number.isFinite(cursor.x) && Number.isFinite(cursor.y),
        JSON.stringify(cursor)
      )
      return `${all.length} monitor(s), primary ${primary.name} ${primary.size.width}x${primary.size.height} @${primary.scaleFactor}x`
    }
  )

  // --- windows ------------------------------------------------------------------------------
  await check('windows: getAllWindows / getByLabel / getCurrent', async () => {
    assert(main.label === 'main', main.label)
    assert((await labels()).includes('main'), 'main missing')
    const w = await Window.getByLabel('main')
    assert(w && w.label === 'main', 'getByLabel')
    assert((await Window.getByLabel('nope')) === null, 'getByLabel(nope)')
    return (await labels()).join(', ')
  })
  await check('windows: builder options are applied', async () => {
    const w = await openWindow('opts', {
      title: 'Builder ✓',
      resizable: false,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      minWidth: 200,
      minHeight: 150,
      maxWidth: 900,
      maxHeight: 700,
      focus: false
    })
    const got = await windowFlags(w)
    const want = {
      title: 'Builder ✓',
      resizable: false,
      decorated: false,
      onTop: true,
      minimizable: true,
      maximizable: true,
      closable: true,
      visible: true
    }
    assert(JSON.stringify(got) === JSON.stringify(want), JSON.stringify(got))
    return JSON.stringify(got)
  })
  await check(
    'windows: builder closable / minimizable / maximizable: false',
    async () => {
      const w = await openWindow('opts2', {
        minimizable: false,
        maximizable: false,
        closable: false
      })
      const got = await windowFlags(w)
      await w.destroy()
      const want = {
        title: 'opts2',
        resizable: true,
        decorated: true,
        onTop: false,
        minimizable: false,
        maximizable: false,
        closable: false,
        visible: true
      }
      assert(JSON.stringify(got) === JSON.stringify(want), JSON.stringify(got))
      return JSON.stringify(got)
    }
  )
  await check(
    'windows: decorations:false survives closable:false',
    async () => {
      const w = await openWindow('opts3', {
        decorations: false,
        closable: false
      })
      const decorated = await w.isDecorated()
      await w.destroy()
      assert(!decorated, 'isDecorated() is true')
      return 'still frameless'
    }
  )
  await check(
    'windows: destroy() skips close-requested and emits destroyed',
    async () => {
      const w = await WebviewWindow.getByLabel('opts')
      let closeRequested = false
      const unClose = await w.onCloseRequested((e) => {
        closeRequested = true
        e.preventDefault()
      })
      const destroyed = new Promise((res) => w.once('tauri://destroyed', res))
      await w.destroy()
      await destroyed
      unClose()
      assert(!closeRequested, 'close-requested fired')
      await waitFor(
        'opts to disappear',
        async () => !(await labels()).includes('opts')
      )
      return 'destroyed without a close request'
    }
  )
  await check('windows: hidden window, show / hide / isVisible', async () => {
    const w = new WebviewWindow('hidden', {
      url: 'index.html?child',
      visible: false,
      width: 300,
      height: 200
    })
    await created(w)
    assert(!(await w.isVisible()), 'visible after visible:false')
    await w.show()
    await waitFor('visible', () => w.isVisible())
    await w.hide()
    await waitFor('hidden', async () => !(await w.isVisible()))
    await w.destroy()
    return 'visible:false -> show -> hide'
  })

  // A window for the window-API checks below.
  let wt
  await check('windows: open a window for the API checks', async () => {
    wt = await openWindow('wintest', { width: 420, height: 320 })
    return (await labels()).join(', ')
  })
  if (wt) {
    const scale = await wt.scaleFactor()
    await check('window: setTitle / title', async () => {
      await wt.setTitle('Title ✓ ünïcödé')
      assert((await wt.title()) === 'Title ✓ ünïcödé', await wt.title())
      return await wt.title()
    })
    await check(
      'window: setSize / innerSize / outerSize + onResized',
      async () => {
        const sizes = []
        const un = await wt.onResized((e) => sizes.push(e.payload))
        await wt.setSize(new LogicalSize(500, 360))
        const want = {
          width: Math.round(500 * scale),
          height: Math.round(360 * scale)
        }
        const inner = await waitFor('inner size', async () => {
          const s = await wt.innerSize()
          return s.width === want.width && s.height === want.height && s
        })
        await waitFor('resized event', () =>
          sizes.some((s) => s.width === want.width && s.height === want.height)
        )
        un()
        const outer = await wt.outerSize()
        assert(
          outer.width >= inner.width && outer.height >= inner.height,
          `outer ${JSON.stringify(outer)}`
        )
        await wt.setSize(
          new PhysicalSize(Math.round(440 * scale), Math.round(330 * scale))
        )
        await waitFor(
          'physical size',
          async () => (await wt.innerSize()).width === Math.round(440 * scale)
        )
        return `inner ${inner.width}x${inner.height}, outer ${outer.width}x${outer.height}, events ${sizes.length}`
      }
    )
    await check('window: min/max size constrain setSize', async () => {
      await wt.setMinSize(new LogicalSize(460, 340))
      await wt.setSize(new LogicalSize(200, 100))
      await sleep(200)
      let s = await wt.innerSize()
      assert(
        s.width >= Math.round(460 * scale) - 1
          && s.height >= Math.round(340 * scale) - 1,
        'min: ' + JSON.stringify(s)
      )
      await wt.setMinSize(null)
      await wt.setMaxSize(new LogicalSize(480, 350))
      await wt.setSize(new LogicalSize(900, 900))
      await sleep(200)
      s = await wt.innerSize()
      assert(
        s.width <= Math.round(480 * scale) + 1
          && s.height <= Math.round(350 * scale) + 1,
        'max: ' + JSON.stringify(s)
      )
      await wt.setMaxSize(null)
      await wt.setSizeConstraints({
        minWidth: 300,
        minHeight: 200,
        maxWidth: 1000,
        maxHeight: 800
      })
      await wt.setSizeConstraints(undefined)
      await wt.setSize(new LogicalSize(440, 330))
      await waitFor(
        'unconstrained size',
        async () => (await wt.innerSize()).width === Math.round(440 * scale)
      )
      return 'clamped to min and max, constraints cleared'
    })
    await check(
      'window: setPosition / outerPosition / innerPosition + onMoved',
      async () => {
        if (wayland) skip('Wayland clients cannot position windows')
        const moves = []
        const un = await wt.onMoved((e) => moves.push(e.payload))
        await wt.setPosition(new LogicalPosition(120, 90))
        const want = { x: Math.round(120 * scale), y: Math.round(90 * scale) }
        const pos = await waitFor('position', async () => {
          const p = await wt.outerPosition()
          return near(p.x, want.x) && near(p.y, want.y) && p
        })
        await waitFor('moved event', () => moves.length > 0)
        un()
        const inner = await wt.innerPosition()
        assert(
          inner.x >= pos.x && inner.y >= pos.y,
          'inner ' + JSON.stringify(inner)
        )
        await wt.setPosition(new PhysicalPosition(want.x + 10, want.y + 10))
        await waitFor('physical position', async () =>
          near((await wt.outerPosition()).x, want.x + 10)
        )
        return `outer ${pos.x},${pos.y} inner ${inner.x},${inner.y}, ${moves.length} move event(s)`
      }
    )
    await check('window: center', async () => {
      if (wayland) skip('Wayland clients cannot position windows')
      await wt.center()
      const m = await T.window.monitorFromPoint(
        (await wt.outerPosition()).x + 1,
        (await wt.outerPosition()).y + 1
      )
      const size = await wt.outerSize()
      const want = {
        x: m.position.x + (m.size.width - size.width) / 2,
        y: m.position.y + (m.size.height - size.height) / 2
      }
      const pos = await waitFor('centered', async () => {
        const p = await wt.outerPosition()
        return near(p.x, want.x, 40) && near(p.y, want.y, 40) && p
      })
      return `${pos.x},${pos.y} on ${m.size.width}x${m.size.height}`
    })
    await check('window: boolean properties round-trip', async () => {
      const props = [
        ['Resizable', 'isResizable'],
        ['Decorations', 'isDecorated'],
        ['AlwaysOnTop', 'isAlwaysOnTop'],
        ['Closable', 'isClosable'],
        ['Minimizable', 'isMinimizable'],
        ['Maximizable', 'isMaximizable'],
        ['Enabled', 'isEnabled']
      ]
      const out = []
      for (const [set, get] of props) {
        const initial = await wt[get]()
        await wt['set' + set](!initial)
        const flipped = await waitFor(
          `${get} = ${!initial}`,
          async () => (await wt[get]()) === !initial || null,
          2000
        ).catch((e) => e)
        await wt['set' + set](initial)
        assert(flipped === true, `${get}: ${flipped}`)
        assert((await wt[get]()) === initial, `${get} not restored`)
        out.push(`${get}=${initial}`)
      }
      return out.join(' ')
    })
    await check('window: maximize / unmaximize / toggleMaximize', async () => {
      const before = await wt.innerSize()
      await wt.maximize()
      await waitFor('maximized', () => wt.isMaximized())
      const max = await waitFor('bigger', async () => {
        const s = await wt.innerSize()
        return s.width > before.width && s
      })
      await wt.unmaximize()
      await waitFor('unmaximized', async () => !(await wt.isMaximized()))
      await wt.toggleMaximize()
      await waitFor('maximized by toggle', () => wt.isMaximized())
      await wt.toggleMaximize()
      await waitFor('restored by toggle', async () => !(await wt.isMaximized()))
      return `${before.width}x${before.height} -> ${max.width}x${max.height} -> restored`
    })
    await check('window: minimize / unminimize / isMinimized', async () => {
      await wt.minimize()
      const minimized = await waitFor(
        'minimized',
        () => wt.isMinimized(),
        2000
      ).catch(() => false)
      if (!minimized && wayland)
        skip('the compositor does not report minimized state')
      assert(minimized, 'isMinimized false after minimize')
      await wt.unminimize()
      await waitFor('unminimized', async () => !(await wt.isMinimized()))
      return 'minimized and restored'
    })
    await check('window: setFullscreen / isFullscreen', async () => {
      await wt.setFullscreen(true)
      await waitFor('fullscreen', () => wt.isFullscreen())
      const m = await T.window.monitorFromPoint(
        (await wt.outerPosition()).x + 1,
        (await wt.outerPosition()).y + 1
      )
      const s = await waitFor('monitor-sized', async () => {
        const s = await wt.innerSize()
        return s.width === m.size.width && s
      })
      await wt.setFullscreen(false)
      await waitFor('windowed', async () => !(await wt.isFullscreen()))
      return `fullscreen ${s.width}x${s.height}`
    })
    await check('window: setFocus / isFocused + onFocusChanged', async () => {
      await main.setFocus()
      const mainFocused = await waitFor(
        'main focused',
        () => main.isFocused(),
        3000
      ).catch(() => false)
      if (!mainFocused && wayland)
        skip('the compositor refused to activate the window')
      assert(mainFocused, 'main not focused')
      const focus = []
      const un = await wt.onFocusChanged((e) => focus.push(e.payload))
      try {
        await wt.setFocus()
        const ok = await waitFor(
          'wintest focused',
          () => wt.isFocused(),
          3000
        ).catch(() => false)
        if (!ok && wayland)
          skip('the compositor refused to activate the window')
        assert(ok, 'not focused after setFocus')
        assert(!(await main.isFocused()), 'main still focused')
        await waitFor('focus event', () => focus.includes(true))
        await main.setFocus()
        await waitFor('main focused again', () => main.isFocused())
        await waitFor('blur event', () => focus.includes(false))
      } finally {
        un()
      }
      return `events: ${focus.join(', ')}`
    })
    await check(
      'window: theme / setTheme / app setTheme are accepted',
      async () => {
        for (const t of ['dark', 'light', null]) await wt.setTheme(t)
        for (const t of ['dark', null]) await T.app.setTheme(t)
        const theme = await wt.theme()
        assert(theme === 'light' || theme === 'dark', theme)
        return `theme: ${theme}`
      }
    )
    // The theme is app-wide on Linux; theme() reports what was set even when the Qt platform
    // theme ignores the request (offscreen, some Wayland desktops).
    await check('window: theme() reflects setTheme', async () => {
      await wt.setTheme('dark')
      try {
        await waitFor('dark', async () => (await wt.theme()) === 'dark', 2000)
        await T.app.setTheme('dark')
        await waitFor(
          'app dark',
          async () => (await main.theme()) === 'dark',
          2000
        )
      } finally {
        await wt.setTheme(null)
        await T.app.setTheme(null)
      }
      return 'dark'
    })
    await check('window: other setters are accepted', async () => {
      const icon = await T.app.defaultWindowIcon()
      const calls = [
        () => wt.setIcon(icon),
        () => wt.setBackgroundColor([30, 30, 30, 255]),
        () => wt.setProgressBar({ status: 'normal', progress: 40 }),
        () => wt.setProgressBar({ status: 'none' }),
        () => wt.setBadgeCount(3),
        () => wt.setBadgeCount(undefined),
        () => wt.requestUserAttention(T.window.UserAttentionType.Informational),
        () => wt.requestUserAttention(null),
        () => wt.setCursorIcon('pointer'),
        () => wt.setCursorIcon('default'),
        () => wt.setCursorVisible(false),
        () => wt.setCursorVisible(true),
        () => wt.setIgnoreCursorEvents(true),
        () => wt.setIgnoreCursorEvents(false),
        () => wt.setSkipTaskbar(true),
        () => wt.setSkipTaskbar(false),
        () => wt.setAlwaysOnBottom(true),
        () => wt.setAlwaysOnBottom(false),
        () => wt.setFocusable(false),
        () => wt.setFocusable(true),
        () => wt.setContentProtected(true),
        () => wt.setContentProtected(false),
        () => wt.setShadow(true),
        () => wt.setVisibleOnAllWorkspaces(false)
      ]
      for (const call of calls) await call()
      assert(await wt.isVisible(), 'window hidden by a setter')
      return `${calls.length} calls`
    })

    // --- webviews ---------------------------------------------------------------------------
    let wv
    await check(
      'webview: child webview from JS (new Webview) with IPC',
      async () => {
        const { ready } = await childReady('wv-test')
        wv = new Webview(main, 'wv-test', {
          url: 'index.html?child',
          x: 10,
          y: 10,
          width: 300,
          height: 200
        })
        await created(wv)
        await ready
        const hosts = await webviewHosts()
        assert(hosts['wv-test'] === 'main', JSON.stringify(hosts))
        const all = (await getAllWebviews()).map((w) => w.label)
        assert(all.includes('wv-test') && all.includes('main'), all.join(', '))
        assert(getCurrentWebview().label === 'main', getCurrentWebview().label)
        return all.join(', ')
      }
    )
    if (wv) {
      await check(
        'webview: position / size / setPosition / setSize',
        async () => {
          await wv.setPosition(new LogicalPosition(20, 30))
          await wv.setSize(new LogicalSize(260, 180))
          const pos = await wv.position(),
            size = await wv.size()
          assert(
            near(pos.x, 20 * scale) && near(pos.y, 30 * scale),
            'position ' + JSON.stringify(pos)
          )
          assert(
            near(size.width, 260 * scale) && near(size.height, 180 * scale),
            'size ' + JSON.stringify(size)
          )
          const seen = await waitFor('page sees its new size', async () => {
            const r = await probe('wv-test')
            return near(r.width, 260, 1) && r
          })
          return `at ${pos.x},${pos.y} ${size.width}x${size.height}, page ${seen.width}x${seen.height}`
        }
      )
      await check('webview: setZoom changes the page scale', async () => {
        const before = await probe('wv-test')
        await wv.setZoom(2)
        const zoomed = await waitFor('zoomed', async () => {
          const r = await probe('wv-test')
          return near(r.dpr, before.dpr * 2, 0.05) && r
        })
        await wv.setZoom(1)
        await waitFor('unzoomed', async () =>
          near((await probe('wv-test')).dpr, before.dpr, 0.05)
        )
        return `devicePixelRatio ${before.dpr} -> ${zoomed.dpr}, innerWidth ${before.width} -> ${zoomed.width}`
      })
      await check(
        'webview: hide / show / setFocus / setBackgroundColor / setAutoResize',
        async () => {
          await wv.hide()
          await wv.show()
          await wv.setFocus()
          await wv.setBackgroundColor([255, 0, 0, 255])
          await wv.setAutoResize(true)
          await wv.setAutoResize(false)
          await probe('wv-test')
          return 'all accepted, page still responsive'
        }
      )
      await check(
        'event: emitTo a webview reaches only that webview',
        async () => {
          const replies = await collect('probe-reply')
          await emitTo(
            { kind: 'Webview', label: 'wv-test' },
            'probe',
            'to wv-test'
          )
          await waitFor('reply', () => replies.got.length > 0)
          await sleep(300)
          // `wintest` also listens for `probe` on its own webview and must not answer.
          await emitTo('wintest', 'probe', 'to wintest')
          await waitFor('wintest reply', () =>
            replies.got.some((r) => r.label === 'wintest')
          )
          await sleep(300)
          replies.stop()
          const got = replies.got.map((r) => `${r.label}:${r.payload}`)
          assert(
            got.length === 2
              && got[0] === 'wv-test:to wv-test'
              && got[1] === 'wintest:to wintest',
            got.join(', ')
          )
          return got.join(', ')
        }
      )
      await check('event: Rust emit_to a webview label', async () => {
        const replies = await collect('probe-reply')
        await invoke('emit_to', {
          target: 'wv-test',
          event: 'probe',
          payload: { from: 'rust' }
        })
        const r = await waitFor('reply', () => replies.got[0])
        await sleep(300)
        replies.stop()
        assert(
          replies.got.length === 1
            && r.label === 'wv-test'
            && r.payload.from === 'rust',
          JSON.stringify(replies.got)
        )
        return JSON.stringify(r.payload)
      })
      await check('webview: reparent to another window', async () => {
        await wv.reparent('wintest')
        const host = await waitFor(
          'new parent',
          async () =>
            (await webviewHosts())['wv-test'] === 'wintest' && 'wintest'
        )
        const r = await probe('wv-test', 'after reparent')
        assert(r.payload === 'after reparent', JSON.stringify(r))
        return `wv-test now in ${host}, IPC still works`
      })
      await check('webview: close removes only the webview', async () => {
        await wv.close()
        await waitFor(
          'gone',
          async () =>
            !(await getAllWebviews()).some((w) => w.label === 'wv-test')
        )
        assert((await labels()).includes('wintest'), 'host window closed too')
        await probe('wintest')
        return 'wv-test closed, wintest still alive'
      })
    }

    await check(
      'window: onCloseRequested preventDefault keeps it open',
      async () => {
        let requests = 0
        const un = await wt.onCloseRequested((e) => {
          if (++requests === 1) e.preventDefault()
        })
        await wt.close()
        await waitFor('close request', () => requests === 1)
        await sleep(300)
        assert(
          (await labels()).includes('wintest'),
          'closed despite preventDefault'
        )
        const destroyed = new Promise((res) =>
          wt.once('tauri://destroyed', res)
        )
        await wt.close()
        await destroyed
        un()
        await waitFor(
          'wintest gone',
          async () => !(await labels()).includes('wintest')
        )
        return `${requests} close requests: first prevented, second closed it`
      }
    )
  }

  // --- events -------------------------------------------------------------------------------
  await check('event: once fires once, unlisten stops delivery', async () => {
    let onceCount = 0,
      count = 0
    await once('e2e-once', () => onceCount++)
    const un = await listen('e2e-once', () => count++)
    await emit('e2e-once', 1)
    await emit('e2e-once', 2)
    await waitFor('two deliveries', () => count === 2)
    un()
    await emit('e2e-once', 3)
    await sleep(200)
    assert(onceCount === 1 && count === 2, `once ${onceCount}, listen ${count}`)
    return `once: ${onceCount}, listen: ${count}`
  })
  await check(
    'event: emitTo a window / any-label target and a missing target',
    async () => {
      const got = []
      const un1 = await main.listen('e2e-target', (e) =>
        got.push('window:' + e.payload)
      )
      const un2 = await getCurrentWebview().listen('e2e-target', (e) =>
        got.push('webview:' + e.payload)
      )
      await emitTo({ kind: 'Window', label: 'main' }, 'e2e-target', 'w')
      await emitTo({ kind: 'Webview', label: 'main' }, 'e2e-target', 'v')
      await emitTo('nobody', 'e2e-target', 'x')
      await waitFor('two deliveries', () => got.length >= 2)
      await sleep(200)
      un1()
      un2()
      assert(
        got.includes('window:w')
          && got.includes('webview:v')
          && !got.some((g) => g.endsWith(':x')),
        got.join(', ')
      )
      return got.join(', ')
    }
  )

  // --- IPC ----------------------------------------------------------------------------------
  const pattern = (n) =>
    new Uint8Array(n).map((_, i) => (Math.imul(i, 2654435761) >>> 24) & 255)
  async function rawRoundTrip(n) {
    const data = pattern(n)
    const out = new Uint8Array(await invoke('reverse_bytes', data))
    assert(out.length === data.length, `length ${out.length}, want ${n}`)
    for (let i = 0; i < n; i++)
      assert(
        out[i] === data[n - 1 - i],
        `byte ${i}: ${out[i]}, want ${data[n - 1 - i]}`
      )
    return `${n} bytes round-tripped`
  }
  await check('ipc: raw binary request and response (8 KiB)', () =>
    rawRoundTrip(8 * 1024)
  )
  // Above 16 KiB Chromium streams request bodies (regression: they used to hang or corrupt).
  await check(
    'ipc: raw binary request and response (20 KB)',
    () => rawRoundTrip(20_000),
    20000
  )
  await check(
    'ipc: raw binary request and response (4 MiB)',
    () => rawRoundTrip(4 * 1024 * 1024),
    20000
  )
  await check('ipc: custom request headers', async () => {
    const v = await invoke(
      'ipc_header',
      { name: 'x-test' },
      { headers: { 'X-Test': 'hello' } }
    )
    assert(v === 'hello', String(v))
    const ascii = await invoke(
      'ipc_header',
      { name: 'x-other' },
      { headers: new Headers({ 'X-Other': 'plain' }) }
    )
    assert(ascii === 'plain', String(ascii))
    return `x-test=${v}, x-other=${ascii}`
  })
  await check(
    'ipc: large JSON response (8 MB string)',
    async () => {
      const s = await invoke('big_string', { len: 8_000_000 })
      assert(
        s.length === 8_000_000 && s.slice(0, 12) === '012345678901',
        `${s.length}`
      )
      return `${s.length} chars`
    },
    20000
  )
  async function jsonArgs(n) {
    const bytes = Array.from({ length: n }, (_, i) => i & 255)
    const text = '日本語 ✓ 🦀 ' + 'x'.repeat(n)
    const [sum, chars] = await invoke('measure', { bytes, text })
    const want = bytes.reduce((a, b) => a + b, 0)
    assert(
      sum === want && chars === [...text].length,
      `${sum}/${want} ${chars}/${[...text].length}`
    )
    return `sum ${sum}, ${chars} chars`
  }
  await check('ipc: JSON array and unicode arguments', () => jsonArgs(2000))
  await check(
    'ipc: JSON arguments of ~18 KB (4000-element array)',
    () => jsonArgs(4000),
    20000
  )
  await check(
    'ipc: Channel streams 5000 messages in order, then raw chunks',
    async () => {
      const numbers = [],
        chunks = []
      const ch = new Channel((n) => numbers.push(n))
      const raw = new Channel((b) => chunks.push(new Uint8Array(b)))
      await invoke('stream_numbers', { count: 5000, numbers: ch, raw })
      await waitFor('5000 numbers', () => numbers.length === 5000, 10000)
      await waitFor('4 chunks', () => chunks.length === 4, 5000)
      assert(
        numbers.every((n, i) => n === i),
        'out of order'
      )
      assert(
        chunks.every(
          (c, i) => c.length === 65536 && c[0] === i && c[65535] === i
        ),
        chunks.map((c) => c.length).join()
      )
      return `${numbers.length} numbers, ${chunks.length} raw chunks of ${chunks[0].length} bytes`
    },
    20000
  )
  await check('ipc: 100 concurrent invokes', async () => {
    const results = await Promise.all(
      Array.from({ length: 100 }, (_, i) => invoke('greet', { name: 'n' + i }))
    )
    assert(
      results.every((r, i) => r.includes(`Hello, n${i}!`)),
      'mismatched replies'
    )
    return `${results.length} replies matched`
  })

  // --- custom protocols ---------------------------------------------------------------------
  const asyncUrl = (path) => convertFileSrc(path, 'async')
  async function postEcho(n) {
    const body = pattern(n)
    const r = await fetch(asyncUrl('echo') + '?q=1', {
      method: 'POST',
      body,
      headers: { 'X-Test': 'yes' }
    })
    const json = await r.json()
    const sum = body.reduce((a, b) => a + b, 0)
    assert(
      json.method === 'POST' && json.len === body.length && json.sum === sum,
      JSON.stringify(json)
    )
    assert(json.header === 'yes' && json.query === 'q=1', JSON.stringify(json))
    assert(
      r.headers.get('x-reply') === 'from-rust',
      'x-reply: ' + r.headers.get('x-reply')
    )
    const put = await (
      await fetch(asyncUrl('echo'), { method: 'PUT', body: 'abc' })
    ).json()
    assert(put.method === 'PUT' && put.len === 3, JSON.stringify(put))
    return JSON.stringify(json)
  }
  await check('protocol: async handler gets POST body, headers and query', () =>
    postEcho(8 * 1024)
  )
  await check('protocol: POST body of 200 KB', () => postEcho(200_000), 20000)
  await check(
    'protocol: large binary response (8 MiB)',
    async () => {
      const n = 8 * 1024 * 1024
      const buf = new Uint8Array(
        await (await fetch(asyncUrl('bytes') + '?n=' + n)).arrayBuffer()
      )
      assert(buf.length === n, 'length ' + buf.length)
      for (let i = 0; i < n; i += 9973)
        assert(buf[i] === i % 251, 'mismatch at ' + i)
      return `${buf.length} bytes`
    },
    20000
  )
  await check('protocol: redirect is followed', async () => {
    const r = await fetch(asyncUrl('redirect'))
    const json = await r.json()
    assert(json.query === 'redirected=1', JSON.stringify(json))
    return `${r.url} -> ${JSON.stringify(json)}`
  })
  await check('protocol: empty 404 is a network error', async () => {
    let r
    try {
      r = await fetch(asyncUrl('missing'))
    } catch (e) {
      return 'rejected: ' + e
    }
    throw new Error(
      `fetch resolved: ${r.status} ${r.statusText} body "${await r.text()}"`
    )
  })
  async function ranges(list) {
    const url = convertFileSrc('data', 'stream')
    const full = await (await fetch(url)).text()
    const out = []
    for (const [range, want] of list(full)) {
      const r = await fetch(url, { headers: { Range: range } })
      const text = await r.text()
      assert(
        r.status === 206 && text === want,
        `${range}: ${r.status} ${JSON.stringify(text)} want ${JSON.stringify(want)}`
      )
      out.push(`${range} -> ${r.headers.get('content-range')}`)
    }
    return out.join(', ')
  }
  await check('protocol: open-ended and single-byte ranges', () =>
    ranges((full) => [
      ['bytes=990-', full.slice(990)],
      ['bytes=0-0', full[0]],
      ['bytes=999-999', full[999]]
    ])
  )
  await check('protocol: suffix range (bytes=-N)', () =>
    ranges((full) => [['bytes=-5', full.slice(-5)]])
  )
  await check(
    'protocol: <audio> loads metadata and seeks over ranges',
    async () => {
      const audio = new Audio()
      audio.preload = 'auto'
      const meta = new Promise((res, rej) => {
        audio.onloadedmetadata = res
        audio.onerror = () =>
          rej(new Error('media error ' + (audio.error && audio.error.message)))
      })
      audio.src = convertFileSrc('tone.wav', 'stream')
      await meta
      assert(near(audio.duration, 3, 0.05), 'duration ' + audio.duration)
      const seeked = new Promise((res) => (audio.onseeked = res))
      audio.currentTime = 2.5
      await seeked
      assert(
        near(audio.currentTime, 2.5, 0.05),
        'currentTime ' + audio.currentTime
      )
      audio.removeAttribute('src')
      audio.load()
      return `duration ${audio.duration || 3}s, seeked to 2.5s`
    }
  )

  // --- tray ---------------------------------------------------------------------------------
  await check('tray: create, getById, update, close', async () => {
    const { TrayIcon } = T.tray
    let tray
    try {
      tray = await TrayIcon.new({
        id: 'e2e-tray',
        tooltip: 'e2e',
        icon: await T.app.defaultWindowIcon()
      })
    } catch (e) {
      if (offscreen) skip('no tray offscreen: ' + e)
      throw e
    }
    const found = await TrayIcon.getById('e2e-tray')
    assert(found && found.id === 'e2e-tray', 'getById')
    await tray.setTooltip('e2e updated')
    await tray.setVisible(false)
    await tray.close()
    assert(
      (await TrayIcon.getById('e2e-tray')) === null,
      'still registered after close'
    )
    return 'created, updated and removed'
  })

  await check('windows: only main is left', async () => {
    const left = await labels()
    assert(left.length === 1 && left[0] === 'main', left.join(', '))
    return left.join(', ')
  })
}
