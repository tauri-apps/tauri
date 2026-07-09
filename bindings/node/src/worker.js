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
const { api, check, takeString, eventsNext } = open(workerData.libPath)

const commands = new Map() // command name -> handler
const eventListeners = new Map() // listener id -> handler
const lifecycleListeners = new Map() // message type -> Set<handler>

/**
 * Mirrors `tauri::WebviewWindow` — one method per Rust method, on a window
 * handle. Obtain instances via `app.createWindow()` / `app.getWindow()`;
 * call `free()` when done with the handle (the window itself is unaffected).
 */
export class WebviewWindow {
  #handle
  constructor(handle) {
    this.#handle = handle
  }

  label() {
    const out = [null]
    check(api.windowLabel(this.#handle, out), 'window.label')
    return takeString(out)
  }
  title() {
    const out = [null]
    check(api.windowTitle(this.#handle, out), 'window.title')
    return takeString(out)
  }
  url() {
    const out = [null]
    check(api.windowUrl(this.#handle, out), 'window.url')
    return takeString(out)
  }
  scaleFactor() {
    const out = [0]
    check(api.windowScaleFactor(this.#handle, out), 'window.scaleFactor')
    return out[0]
  }
  innerSize() {
    const width = [0]
    const height = [0]
    check(api.windowInnerSize(this.#handle, width, height), 'window.innerSize')
    return { width: width[0], height: height[0] }
  }
  outerSize() {
    const width = [0]
    const height = [0]
    check(api.windowOuterSize(this.#handle, width, height), 'window.outerSize')
    return { width: width[0], height: height[0] }
  }
  innerPosition() {
    const x = [0]
    const y = [0]
    check(api.windowInnerPosition(this.#handle, x, y), 'window.innerPosition')
    return { x: x[0], y: y[0] }
  }
  outerPosition() {
    const x = [0]
    const y = [0]
    check(api.windowOuterPosition(this.#handle, x, y), 'window.outerPosition')
    return { x: x[0], y: y[0] }
  }

  #bool(fn, what) {
    const out = [false]
    check(fn(this.#handle, out), what)
    return out[0]
  }
  isVisible() {
    return this.#bool(api.windowIsVisible, 'window.isVisible')
  }
  isFocused() {
    return this.#bool(api.windowIsFocused, 'window.isFocused')
  }
  isFullscreen() {
    return this.#bool(api.windowIsFullscreen, 'window.isFullscreen')
  }
  isMaximized() {
    return this.#bool(api.windowIsMaximized, 'window.isMaximized')
  }
  isMinimized() {
    return this.#bool(api.windowIsMinimized, 'window.isMinimized')
  }
  isResizable() {
    return this.#bool(api.windowIsResizable, 'window.isResizable')
  }

  setTitle(title) {
    check(api.windowSetTitle(this.#handle, title), 'window.setTitle')
  }
  /** Sizes are logical (DPI-scaled) unless `physical: true`. */
  setSize({ width, height, physical = false }) {
    check(api.windowSetSize(this.#handle, width, height, physical), 'window.setSize')
  }
  setPosition({ x, y, physical = false }) {
    check(api.windowSetPosition(this.#handle, x, y, physical), 'window.setPosition')
  }
  setFullscreen(fullscreen) {
    check(api.windowSetFullscreen(this.#handle, fullscreen), 'window.setFullscreen')
  }
  setResizable(resizable) {
    check(api.windowSetResizable(this.#handle, resizable), 'window.setResizable')
  }
  setAlwaysOnTop(alwaysOnTop) {
    check(api.windowSetAlwaysOnTop(this.#handle, alwaysOnTop), 'window.setAlwaysOnTop')
  }
  setDecorations(decorations) {
    check(api.windowSetDecorations(this.#handle, decorations), 'window.setDecorations')
  }
  setFocus() {
    check(api.windowSetFocus(this.#handle), 'window.setFocus')
  }
  setZoom(scale) {
    check(api.windowSetZoom(this.#handle, scale), 'window.setZoom')
  }
  show() {
    check(api.windowShow(this.#handle), 'window.show')
  }
  hide() {
    check(api.windowHide(this.#handle), 'window.hide')
  }
  center() {
    check(api.windowCenter(this.#handle), 'window.center')
  }
  maximize() {
    check(api.windowMaximize(this.#handle), 'window.maximize')
  }
  unmaximize() {
    check(api.windowUnmaximize(this.#handle), 'window.unmaximize')
  }
  minimize() {
    check(api.windowMinimize(this.#handle), 'window.minimize')
  }
  unminimize() {
    check(api.windowUnminimize(this.#handle), 'window.unminimize')
  }
  close() {
    check(api.windowClose(this.#handle), 'window.close')
  }
  destroy() {
    check(api.windowDestroy(this.#handle), 'window.destroy')
  }

  eval(js) {
    check(api.windowEval(this.#handle, js), 'window.eval')
  }
  navigate(url) {
    check(api.windowNavigate(this.#handle, String(url)), 'window.navigate')
  }
  reload() {
    check(api.windowReload(this.#handle), 'window.reload')
  }

  /** Releases the handle; the window itself is unaffected. */
  free() {
    check(api.handleClose(this.#handle), 'window.free')
  }
}

export const app = {
  /** Handle a command registered in launch(): `handler(payload, { webview })`.
   * The return value (or thrown error) resolves/rejects the frontend invoke. */
  command(name, handler) {
    commands.set(name, handler)
    return app
  },

  /** Applies a plugin (from `definePlugin()`) in this worker, wiring up its
   * command handlers. Pass the same plugin to `launch({ plugins })` on the main
   * thread so its native side and ACL are set up. */
  plugin(plugin) {
    for (const [name, handler] of plugin.handlers) {
      commands.set(`plugin:${plugin.name}|${name}`, handler)
    }
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
    check(api.appListen(appId, event, out), `listen(${event})`)
    eventListeners.set(out[0], handler)
    return out[0]
  },

  unlisten(id) {
    eventListeners.delete(id)
    check(api.appUnlisten(appId, id), 'unlisten')
  },

  emit(event, payload) {
    check(api.appEmit(appId, event, JSON.stringify(payload ?? null)), `emit(${event})`)
  },

  emitTo(label, event, payload) {
    check(api.appEmitTo(appId, label, event, JSON.stringify(payload ?? null)), `emitTo(${event})`)
  },

  /**
   * Creates a webview window from a WindowConfig object (same shape as
   * entries of app.windows in tauri.conf.json). Call only while the app is
   * running (e.g. from the 'ready' handler onwards).
   */
  createWindow(config) {
    const out = [0]
    check(api.windowCreate(appId, JSON.stringify(config), out), `createWindow(${config?.label})`)
    return new WebviewWindow(out[0])
  },

  /** The window with the given label, or null. */
  getWindow(label) {
    const out = [0]
    const code = api.appGetWindow(appId, label, out)
    if (code === CODES.NOT_FOUND) return null
    check(code, `getWindow(${label})`)
    return new WebviewWindow(out[0])
  },

  /** Labels of all webview windows. */
  windowLabels() {
    const out = [null]
    check(api.appWindowLabels(appId, out), 'windowLabels')
    return JSON.parse(takeString(out))
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
  // Plugin invokes are keyed by the full `plugin:<name>|<command>` string.
  const key = message.plugin ? `plugin:${message.plugin}|${message.command}` : message.command
  const handler = commands.get(key)
  if (!handler) {
    api.invokeReject(message.id, JSON.stringify(`command ${key} not found`))
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
    if (code !== CODES.OK) throw new Error(`event pump failed (${code}): ${api.lastErrorMessage() ?? ''}`)

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
