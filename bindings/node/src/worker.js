// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Worker-side app API. Loaded by the module passed to `launch()`; talks to
// the running app entirely through thread-safe FFI calls and consumes the
// serialized event queue (tauri_events_next) — no native callbacks into JS.

import { workerData } from 'node:worker_threads'
import { writeSync } from 'node:fs'
import { format } from 'node:util'
import { open, CODES } from './ffi.js'

// Worker stdout/stderr are normally relayed through the parent thread's event
// loop — which launch() keeps blocked inside tauri_app_run, so nothing would
// ever print. Route console.* straight to the process file descriptors.
for (const [method, fd] of [['log', 1], ['info', 1], ['warn', 2], ['error', 2]]) {
  console[method] = (...args) => writeSync(fd, `${format(...args)}\n`)
}

if (!workerData?.app) {
  throw new Error(
    "'@tauri-apps/node/worker' must be imported from the module passed to launch()"
  )
}

const appId = Number(workerData.app)
const { api, check, eventsNext } = open(workerData.libPath)

const commands = new Map() // command name -> handler
const eventListeners = new Map() // listener id -> handler
const lifecycleListeners = new Map() // message type -> Set<handler>

export const app = {
  /** Handle a command registered in launch(): `handler(payload, { webview })`.
   * The return value (or thrown error) resolves/rejects the frontend invoke. */
  command(name, handler) {
    commands.set(name, handler)
    return app
  },

  /** Lifecycle events: 'ready' | 'exit' | 'exit-requested' | 'window-event'. */
  on(type, handler) {
    if (!lifecycleListeners.has(type)) lifecycleListeners.set(type, new Set())
    lifecycleListeners.get(type).add(handler)
    return app
  },

  /** Listen to a Tauri event from any source (frontend `emit`, host `emit`). */
  listen(event, handler) {
    const out = [0]
    check(api.listen(appId, event, out), `listen(${event})`)
    eventListeners.set(out[0], handler)
    return out[0]
  },

  unlisten(id) {
    eventListeners.delete(id)
    check(api.unlisten(appId, id), 'unlisten')
  },

  emit(event, payload) {
    check(api.emit(appId, event, JSON.stringify(payload ?? null)), `emit(${event})`)
  },

  emitTo(label, event, payload) {
    check(api.emitTo(appId, label, event, JSON.stringify(payload ?? null)), `emitTo(${event})`)
  },

  /** Evaluate JavaScript in a webview window. */
  eval(label, js) {
    check(api.webviewEval(appId, label, js), 'eval')
  },

  exit(code = 0) {
    check(api.appExit(appId, code), 'exit')
  }
}

function fire(type, message) {
  lifecycleListeners.get(type)?.forEach((handler) => {
    try {
      handler(message)
    } catch (error) {
      console.error(`[tauri-ffi] '${type}' handler threw:`, error)
    }
  })
}

async function handleInvoke(message) {
  const handler = commands.get(message.command)
  if (!handler) {
    api.invokeReject(message.id, JSON.stringify(`command ${message.command} not found`))
    return
  }
  try {
    const result = await handler(message.payload, { webview: message.webview })
    check(api.invokeResolve(message.id, JSON.stringify(result ?? null)), 'invoke_resolve')
  } catch (error) {
    api.invokeReject(message.id, JSON.stringify(error?.message ?? String(error)))
  }
}

async function pump() {
  for (;;) {
    const { code, json } = await eventsNext(appId, 1000)
    if (code === CODES.TIMEOUT) continue
    if (code === CODES.CLOSED) return
    if (code !== CODES.OK) throw new Error(`event pump failed (${code}): ${api.lastError() ?? ''}`)

    const message = JSON.parse(json)
    switch (message.type) {
      case 'invoke':
        handleInvoke(message) // deliberately not awaited: handlers may be slow
        break
      case 'event':
        eventListeners.get(message.id)?.(message.payload, message)
        break
      case 'exit':
        fire('exit', message)
        return
      default:
        fire(message.type, message)
    }
  }
}

// Start after the app module finished registering its handlers.
setImmediate(() => {
  pump().catch((error) => {
    console.error('[tauri-ffi] event pump crashed:', error)
    process.exit(1)
  })
})
