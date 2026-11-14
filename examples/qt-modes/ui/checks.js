// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The same IPC checks under every mode; `window.__QT_MODE__` (set by the app through an
// initialization script) says what the mode must look like. Each check reports through the
// `report` command, so a broken IPC shows up as missing reports (the app times out).
;(async () => {
  const mode = window.__QT_MODE__
  const ipc = window.__TAURI_INTERNALS__
  const log = (s) => (document.getElementById('log').textContent += s + '\n')

  // Tauri warns when the `ipc://` custom protocol fails and it falls back to postMessage.
  const warnings = []
  const warn = console.warn
  console.warn = (...args) => {
    warnings.push(args.map(String).join(' '))
    warn(...args)
  }

  let failures = 0
  async function check(name, fn) {
    let ok = true
    let detail
    try {
      detail = String(
        await Promise.race([
          fn(),
          new Promise((_, reject) =>
            setTimeout(() => reject(new Error('timeout')), 15000)
          )
        ])
      )
    } catch (e) {
      ok = false
      detail = String(e && e.stack ? e.stack.split('\n')[0] : e)
    }
    if (!ok) failures++
    log(`${ok ? 'PASS' : 'FAIL'} ${name}: ${detail}`)
    await ipc.invoke('report', { name: `${mode.name}: ${name}`, ok, detail })
  }
  // JSON with object keys sorted (serde_json returns them sorted).
  const canonical = (v) =>
    JSON.stringify(v, (_, x) =>
      x && typeof x === 'object' && !Array.isArray(x)
        ? Object.fromEntries(
            Object.entries(x).sort(([a], [b]) => (a < b ? -1 : 1))
          )
        : x
    )
  const assert = (cond, msg) => {
    if (!cond) throw new Error(msg)
  }

  await check('page origin', () => {
    assert(
      location.origin === mode.origin,
      `${location.origin}, want ${mode.origin}`
    )
    return location.href
  })
  await check('inline script allowed by the CSP', () => {
    assert(window.__inlineScriptRan === true, 'the inline <script> was blocked')
    return mode.csp ? 'ran under the CSP' : 'ran (no CSP)'
  })
  await check('withGlobalTauri', () => {
    const has = typeof window.__TAURI__ !== 'undefined'
    assert(
      has === mode.global,
      `window.__TAURI__ ${has ? 'present' : 'absent'}`
    )
    return has ? 'window.__TAURI__ present' : 'window.__TAURI__ absent'
  })
  await check('isolation iframe', () => {
    const iframe = document.getElementById('__tauri_isolation__')
    assert(!!iframe === mode.isolation, iframe ? 'present' : 'absent')
    return iframe ? `present (${iframe.src.split('://')[0]}://…)` : 'absent'
  })
  await check('invoke with JSON arguments', async () => {
    const value = { text: '日本語 ✓ 🦀', n: 42, list: [1, 2, 3] }
    const r = await ipc.invoke('echo', { value })
    assert(canonical(r.value) === canonical(value), JSON.stringify(r))
    assert(
      r.isolated === mode.isolation,
      `isolation hook ${r.isolated ? 'ran' : "didn't run"}`
    )
    return JSON.stringify(r)
  })
  await check('invoke with a 200 KB argument', async () => {
    const value = 'x'.repeat(200 * 1024)
    const r = await ipc.invoke('echo', { value })
    assert(r.value.length === value.length, `${r.value.length} chars`)
    return `${r.value.length} chars round-tripped`
  })
  await check('raw binary request and response (64 KiB)', async () => {
    const data = new Uint8Array(64 * 1024).map(
      (_, i) => (Math.imul(i, 2654435761) >>> 24) & 255
    )
    const out = new Uint8Array(await ipc.invoke('echo_bytes', data))
    assert(out.length === data.length, `length ${out.length}`)
    for (let i = 0; i < data.length; i++)
      assert(out[i] === data[data.length - 1 - i], `byte ${i}`)
    return `${out.length} bytes reversed`
  })
  await check('command errors reject', async () => {
    try {
      await ipc.invoke('fail')
    } catch (e) {
      assert(String(e).includes('expected failure'), String(e))
      return `rejected: ${e}`
    }
    throw new Error('resolved')
  })
  await check('channel streams 200 messages in order', async () => {
    // What @tauri-apps/api's Channel does: messages are { message, index } (or { end, index })
    // and may arrive out of order, so keep them by index.
    const got = []
    const id = ipc.transformCallback((m) => {
      if (m && typeof m === 'object' && 'message' in m) got[m.index] = m.message
    })
    await ipc.invoke('stream_numbers', {
      count: 200,
      channel: `__CHANNEL__:${id}`
    })
    for (let i = 0; i < 100 && got.length < 200; i++)
      await new Promise((r) => setTimeout(r, 50))
    assert(
      got.length === 200 && got.every((n, i) => n === i),
      `${got.filter(() => true).length} messages`
    )
    return `${got.length} messages in order`
  })
  await check('events from JS to Rust and back', async () => {
    const label = mode.webview
    const got = new Promise((resolve) => {
      const id = ipc.transformCallback((e) => resolve(e.payload))
      ipc.invoke('plugin:event|listen', {
        event: 'modes-pong',
        target: { kind: 'Any' },
        handler: id
      })
    })
    await new Promise((r) => setTimeout(r, 100))
    await ipc.invoke('plugin:event|emit', {
      event: 'modes-ping',
      payload: label
    })
    const payload = await got
    assert(payload === `pong:${label}`, JSON.stringify(payload))
    return JSON.stringify(payload)
  })
  await check('core plugin command (window title)', async () => {
    const title = await ipc.invoke('plugin:window|title', {
      label: mode.window
    })
    assert(typeof title === 'string', typeof title)
    return JSON.stringify(title)
  })
  await check('IPC transport', () => {
    const fellBack = warnings.some((w) =>
      w.includes('IPC custom protocol failed')
    )
    assert(
      fellBack === (mode.transport === 'postMessage'),
      `fell back to postMessage: ${fellBack}`
    )
    return fellBack
      ? 'postMessage (custom protocol blocked)'
      : 'ipc:// custom protocol'
  })
  if (mode.denied) {
    await check(
      'IPC is rejected for an origin without a capability',
      async () => {
        const title = await ipc.invoke('denied_result')
        assert(title.startsWith('denied:rejected'), title)
        return title
      }
    )
  }

  await ipc.invoke('done', { failures })
})()
