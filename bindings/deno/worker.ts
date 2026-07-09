// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Worker-side app API. Loaded by the module passed to `launch()`; talks to
// the running app entirely through thread-safe FFI calls and consumes the
// serialized event queue (tauri_events_next) — no native callbacks into JS.

import { CODES, cstr, open } from './ffi.ts'

const params = new URL(self.location.href).searchParams
const appParam = params.get('tauri-app')
if (!appParam) {
  throw new Error("'worker.ts' must be imported from the module passed to launch()")
}
const appId = BigInt(appParam)
const { sym, check, takeString, eventsNext } = open(params.get('tauri-lib') ?? undefined)

// deno-lint-ignore no-explicit-any
type Json = any
type CommandHandler = (payload: Json, context: { webview: string }) => Json | Promise<Json>

const commands = new Map<string, CommandHandler>()
const eventListeners = new Map<number, (payload: Json, message: Json) => void>()
const lifecycleListeners = new Map<string, Set<(message: Json) => void>>()

/**
 * Mirrors `tauri::WebviewWindow` — one method per Rust method, on a window
 * handle. Obtain instances via `app.createWindow()` / `app.getWindow()`;
 * call `free()` when done with the handle (the window itself is unaffected).
 */
export class WebviewWindow {
  #handle: bigint
  constructor(handle: bigint) {
    this.#handle = handle
  }

  #string(fn: (h: bigint, out: Uint8Array) => number, what: string): string {
    const slot = new BigUint64Array(1)
    check(fn(this.#handle, new Uint8Array(slot.buffer)), what)
    return takeString(slot) ?? ''
  }
  label(): string {
    return this.#string(sym.tauri_window_label, 'window.label')
  }
  title(): string {
    return this.#string(sym.tauri_window_title, 'window.title')
  }
  url(): string {
    return this.#string(sym.tauri_window_url, 'window.url')
  }
  scaleFactor(): number {
    const out = new Float64Array(1)
    check(sym.tauri_window_scale_factor(this.#handle, new Uint8Array(out.buffer)), 'window.scaleFactor')
    return out[0]
  }

  #pair(
    fn: (h: bigint, a: Uint8Array, b: Uint8Array) => number,
    signed: boolean,
    what: string
  ): [number, number] {
    const a = signed ? new Int32Array(1) : new Uint32Array(1)
    const b = signed ? new Int32Array(1) : new Uint32Array(1)
    check(fn(this.#handle, new Uint8Array(a.buffer), new Uint8Array(b.buffer)), what)
    return [a[0], b[0]]
  }
  innerSize(): { width: number; height: number } {
    const [width, height] = this.#pair(sym.tauri_window_inner_size, false, 'window.innerSize')
    return { width, height }
  }
  outerSize(): { width: number; height: number } {
    const [width, height] = this.#pair(sym.tauri_window_outer_size, false, 'window.outerSize')
    return { width, height }
  }
  innerPosition(): { x: number; y: number } {
    const [x, y] = this.#pair(sym.tauri_window_inner_position, true, 'window.innerPosition')
    return { x, y }
  }
  outerPosition(): { x: number; y: number } {
    const [x, y] = this.#pair(sym.tauri_window_outer_position, true, 'window.outerPosition')
    return { x, y }
  }

  #bool(fn: (h: bigint, out: Uint8Array) => number, what: string): boolean {
    const out = new Uint8Array(1)
    check(fn(this.#handle, out), what)
    return out[0] !== 0
  }
  isVisible(): boolean {
    return this.#bool(sym.tauri_window_is_visible, 'window.isVisible')
  }
  isFocused(): boolean {
    return this.#bool(sym.tauri_window_is_focused, 'window.isFocused')
  }
  isFullscreen(): boolean {
    return this.#bool(sym.tauri_window_is_fullscreen, 'window.isFullscreen')
  }
  isMaximized(): boolean {
    return this.#bool(sym.tauri_window_is_maximized, 'window.isMaximized')
  }
  isMinimized(): boolean {
    return this.#bool(sym.tauri_window_is_minimized, 'window.isMinimized')
  }
  isResizable(): boolean {
    return this.#bool(sym.tauri_window_is_resizable, 'window.isResizable')
  }

  setTitle(title: string) {
    check(sym.tauri_window_set_title(this.#handle, cstr(title)), 'window.setTitle')
  }
  /** Sizes are logical (DPI-scaled) unless `physical: true`. */
  setSize({ width, height, physical = false }: { width: number; height: number; physical?: boolean }) {
    check(sym.tauri_window_set_size(this.#handle, width, height, physical), 'window.setSize')
  }
  setPosition({ x, y, physical = false }: { x: number; y: number; physical?: boolean }) {
    check(sym.tauri_window_set_position(this.#handle, x, y, physical), 'window.setPosition')
  }
  setFullscreen(fullscreen: boolean) {
    check(sym.tauri_window_set_fullscreen(this.#handle, fullscreen), 'window.setFullscreen')
  }
  setResizable(resizable: boolean) {
    check(sym.tauri_window_set_resizable(this.#handle, resizable), 'window.setResizable')
  }
  setAlwaysOnTop(alwaysOnTop: boolean) {
    check(sym.tauri_window_set_always_on_top(this.#handle, alwaysOnTop), 'window.setAlwaysOnTop')
  }
  setDecorations(decorations: boolean) {
    check(sym.tauri_window_set_decorations(this.#handle, decorations), 'window.setDecorations')
  }
  setFocus() {
    check(sym.tauri_window_set_focus(this.#handle), 'window.setFocus')
  }
  setZoom(scale: number) {
    check(sym.tauri_window_set_zoom(this.#handle, scale), 'window.setZoom')
  }
  show() {
    check(sym.tauri_window_show(this.#handle), 'window.show')
  }
  hide() {
    check(sym.tauri_window_hide(this.#handle), 'window.hide')
  }
  center() {
    check(sym.tauri_window_center(this.#handle), 'window.center')
  }
  maximize() {
    check(sym.tauri_window_maximize(this.#handle), 'window.maximize')
  }
  unmaximize() {
    check(sym.tauri_window_unmaximize(this.#handle), 'window.unmaximize')
  }
  minimize() {
    check(sym.tauri_window_minimize(this.#handle), 'window.minimize')
  }
  unminimize() {
    check(sym.tauri_window_unminimize(this.#handle), 'window.unminimize')
  }
  close() {
    check(sym.tauri_window_close(this.#handle), 'window.close')
  }
  destroy() {
    check(sym.tauri_window_destroy(this.#handle), 'window.destroy')
  }

  eval(js: string) {
    check(sym.tauri_window_eval(this.#handle, cstr(js)), 'window.eval')
  }
  navigate(url: string | URL) {
    check(sym.tauri_window_navigate(this.#handle, cstr(String(url))), 'window.navigate')
  }
  reload() {
    check(sym.tauri_window_reload(this.#handle), 'window.reload')
  }

  /** Releases the handle; the window itself is unaffected. */
  free() {
    check(sym.tauri_handle_close(this.#handle), 'window.free')
  }
}

export const app = {
  /** Handle a command registered in launch(): `handler(payload, { webview })`. */
  command(name: string, handler: CommandHandler) {
    commands.set(name, handler)
    return app
  },

  /** Lifecycle events: 'ready' | 'exit' | 'exit-requested' | 'window-event'. */
  on(type: string, handler: (message: Json) => void) {
    if (!lifecycleListeners.has(type)) lifecycleListeners.set(type, new Set())
    lifecycleListeners.get(type)!.add(handler)
    return app
  },

  /** Listen to a Tauri event from any source (frontend `emit`, host `emit`). */
  listen(event: string, handler: (payload: Json, message: Json) => void): number {
    const out = new Uint32Array(1)
    check(sym.tauri_app_listen(appId, cstr(event), new Uint8Array(out.buffer)), `listen(${event})`)
    eventListeners.set(out[0], handler)
    return out[0]
  },

  unlisten(id: number) {
    eventListeners.delete(id)
    check(sym.tauri_app_unlisten(appId, id), 'unlisten')
  },

  emit(event: string, payload?: Json) {
    check(sym.tauri_app_emit(appId, cstr(event), cstr(JSON.stringify(payload ?? null))), `emit(${event})`)
  },

  emitTo(label: string, event: string, payload?: Json) {
    check(
      sym.tauri_app_emit_to(appId, cstr(label), cstr(event), cstr(JSON.stringify(payload ?? null))),
      `emitTo(${event})`
    )
  },

  /**
   * Creates a webview window from a WindowConfig object. Async because the
   * creation blocks on the event loop (declared nonblocking in symbols.ts).
   */
  async createWindow(config: { label?: string } & Json): Promise<WebviewWindow> {
    const slot = new BigUint64Array(1)
    const code = await sym.tauri_window_create(
      appId,
      cstr(JSON.stringify(config)),
      new Uint8Array(slot.buffer)
    )
    check(code, `createWindow(${config?.label})`)
    return new WebviewWindow(slot[0])
  },

  /** The window with the given label, or null. */
  getWindow(label: string): WebviewWindow | null {
    const slot = new BigUint64Array(1)
    const code = sym.tauri_app_get_window(appId, cstr(label), new Uint8Array(slot.buffer))
    if (code === CODES.NOT_FOUND) return null
    check(code, `getWindow(${label})`)
    return new WebviewWindow(slot[0])
  },

  /** Labels of all webview windows. */
  windowLabels(): string[] {
    const slot = new BigUint64Array(1)
    check(sym.tauri_app_window_labels(appId, new Uint8Array(slot.buffer)), 'windowLabels')
    return JSON.parse(takeString(slot) ?? '[]')
  },

  exit(code = 0) {
    check(sym.tauri_app_exit(appId, code), 'exit')
  }
}

function fire(type: string, message: Json) {
  lifecycleListeners.get(type)?.forEach((handler) => {
    Promise.resolve()
      .then(() => handler(message))
      .catch((error) => console.error(`[tauri-ffi] '${type}' handler threw:`, error))
  })
}

async function handleInvoke(message: Json) {
  const handler = commands.get(message.command)
  if (!handler) {
    sym.tauri_invoke_reject(message.id, cstr(JSON.stringify(`command ${message.command} not found`)))
    return
  }
  try {
    const result = await handler(message.payload, { webview: message.webview })
    check(sym.tauri_invoke_resolve(message.id, cstr(JSON.stringify(result ?? null))), 'invoke_resolve')
  } catch (error) {
    sym.tauri_invoke_reject(
      message.id,
      cstr(JSON.stringify(error instanceof Error ? error.message : String(error)))
    )
  }
}

async function pump() {
  for (;;) {
    const { code, json } = await eventsNext(appId, 1000)
    if (code === CODES.TIMEOUT) continue
    if (code === CODES.CLOSED) return
    if (code !== CODES.OK) throw new Error(`event pump failed (${code})`)

    const message = JSON.parse(json!)
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
setTimeout(() => {
  pump().catch((error) => {
    console.error('[tauri-ffi] event pump crashed:', error)
    Deno.exit(1)
  })
}, 0)
