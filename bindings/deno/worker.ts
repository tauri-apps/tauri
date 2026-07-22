// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Worker-side app API. Loaded by the module passed to `launch()`; talks to
// the running app entirely through thread-safe FFI calls and consumes the
// serialized event queue (tauri_events_next) — no native callbacks into JS.

import { CODES, cstr, open } from './ffi.ts'
import type { Plugin } from './plugin.ts'

const params = new URL(self.location.href).searchParams
const appParam = params.get('tauri-app')
if (!appParam) {
  throw new Error("'worker.ts' must be imported from the module passed to launch()")
}
const appId = BigInt(appParam)
const { sym, check, takeString, eventsNext } = open(params.get('tauri-lib') ?? undefined)

// deno-lint-ignore no-explicit-any
export type Json = any
export type CommandHandler = (
  payload: Json,
  context: { webview: string }
) => Json | Promise<Json>

/** A custom-protocol request, as delivered to a {@linkcode ProtocolHandler}. */
export interface ProtocolRequest {
  scheme: string
  webview: string
  method: string
  uri: string
  headers: Record<string, string>
  body: Uint8Array
}

/** A custom-protocol response; a bare string or Uint8Array means a plain 200. */
export interface ProtocolResponse {
  status?: number
  headers?: Record<string, string | string[]>
  body?: string | Uint8Array
}

export type ProtocolHandler = (
  request: ProtocolRequest
) => string | Uint8Array | ProtocolResponse | null | Promise<string | Uint8Array | ProtocolResponse | null>

const commands = new Map<string, CommandHandler>()
const protocols = new Map<string, ProtocolHandler>()
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
    return this.#string(sym.tauri_webview_window_label, 'window.label')
  }
  title(): string {
    return this.#string(sym.tauri_webview_window_title, 'window.title')
  }
  url(): string {
    return this.#string(sym.tauri_webview_window_url, 'window.url')
  }
  scaleFactor(): number {
    const out = new Float64Array(1)
    check(sym.tauri_webview_window_scale_factor(this.#handle, new Uint8Array(out.buffer)), 'window.scaleFactor')
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
    const [width, height] = this.#pair(sym.tauri_webview_window_inner_size, false, 'window.innerSize')
    return { width, height }
  }
  outerSize(): { width: number; height: number } {
    const [width, height] = this.#pair(sym.tauri_webview_window_outer_size, false, 'window.outerSize')
    return { width, height }
  }
  innerPosition(): { x: number; y: number } {
    const [x, y] = this.#pair(sym.tauri_webview_window_inner_position, true, 'window.innerPosition')
    return { x, y }
  }
  outerPosition(): { x: number; y: number } {
    const [x, y] = this.#pair(sym.tauri_webview_window_outer_position, true, 'window.outerPosition')
    return { x, y }
  }

  #bool(fn: (h: bigint, out: Uint8Array) => number, what: string): boolean {
    const out = new Uint8Array(1)
    check(fn(this.#handle, out), what)
    return out[0] !== 0
  }
  isVisible(): boolean {
    return this.#bool(sym.tauri_webview_window_is_visible, 'window.isVisible')
  }
  isFocused(): boolean {
    return this.#bool(sym.tauri_webview_window_is_focused, 'window.isFocused')
  }
  isFullscreen(): boolean {
    return this.#bool(sym.tauri_webview_window_is_fullscreen, 'window.isFullscreen')
  }
  isMaximized(): boolean {
    return this.#bool(sym.tauri_webview_window_is_maximized, 'window.isMaximized')
  }
  isMinimized(): boolean {
    return this.#bool(sym.tauri_webview_window_is_minimized, 'window.isMinimized')
  }
  isResizable(): boolean {
    return this.#bool(sym.tauri_webview_window_is_resizable, 'window.isResizable')
  }
  isDecorated(): boolean {
    return this.#bool(sym.tauri_webview_window_is_decorated, 'window.isDecorated')
  }
  isClosable(): boolean {
    return this.#bool(sym.tauri_webview_window_is_closable, 'window.isClosable')
  }
  isMaximizable(): boolean {
    return this.#bool(sym.tauri_webview_window_is_maximizable, 'window.isMaximizable')
  }
  isMinimizable(): boolean {
    return this.#bool(sym.tauri_webview_window_is_minimizable, 'window.isMinimizable')
  }
  isAlwaysOnTop(): boolean {
    return this.#bool(sym.tauri_webview_window_is_always_on_top, 'window.isAlwaysOnTop')
  }
  isEnabled(): boolean {
    return this.#bool(sym.tauri_webview_window_is_enabled, 'window.isEnabled')
  }
  isMenuVisible(): boolean {
    return this.#bool(sym.tauri_webview_window_is_menu_visible, 'window.isMenuVisible')
  }
  isDevtoolsOpen(): boolean {
    return this.#bool(sym.tauri_webview_window_is_devtools_open, 'window.isDevtoolsOpen')
  }

  theme(): string {
    return this.#string(sym.tauri_webview_window_theme, 'window.theme')
  }
  /** All available monitors, as Monitor objects. */
  availableMonitors(): Json[] {
    return JSON.parse(this.#string(sym.tauri_webview_window_available_monitors, 'window.availableMonitors') || '[]')
  }
  /** The monitor the window is on, or null. */
  currentMonitor(): Json {
    return JSON.parse(this.#string(sym.tauri_webview_window_current_monitor, 'window.currentMonitor') || 'null')
  }
  /** The primary monitor, or null. */
  primaryMonitor(): Json {
    return JSON.parse(this.#string(sym.tauri_webview_window_primary_monitor, 'window.primaryMonitor') || 'null')
  }
  /** The monitor containing the given physical point, or null. */
  monitorFromPoint(x: number, y: number): Json {
    const slot = new BigUint64Array(1)
    check(
      sym.tauri_webview_window_monitor_from_point(this.#handle, x, y, new Uint8Array(slot.buffer)),
      'window.monitorFromPoint'
    )
    return JSON.parse(takeString(slot) ?? 'null')
  }
  /** The current cursor position, in physical pixels. */
  cursorPosition(): { x: number; y: number } {
    const x = new Float64Array(1)
    const y = new Float64Array(1)
    check(
      sym.tauri_webview_window_cursor_position(this.#handle, new Uint8Array(x.buffer), new Uint8Array(y.buffer)),
      'window.cursorPosition'
    )
    return { x: x[0], y: y[0] }
  }

  setTitle(title: string) {
    check(sym.tauri_webview_window_set_title(this.#handle, cstr(title)), 'window.setTitle')
  }
  /** Sizes are logical (DPI-scaled) unless `physical: true`. */
  setSize({ width, height, physical = false }: { width: number; height: number; physical?: boolean }) {
    check(sym.tauri_webview_window_set_size(this.#handle, width, height, physical), 'window.setSize')
  }
  setPosition({ x, y, physical = false }: { x: number; y: number; physical?: boolean }) {
    check(sym.tauri_webview_window_set_position(this.#handle, x, y, physical), 'window.setPosition')
  }
  setFullscreen(fullscreen: boolean) {
    check(sym.tauri_webview_window_set_fullscreen(this.#handle, fullscreen), 'window.setFullscreen')
  }
  setResizable(resizable: boolean) {
    check(sym.tauri_webview_window_set_resizable(this.#handle, resizable), 'window.setResizable')
  }
  setAlwaysOnTop(alwaysOnTop: boolean) {
    check(sym.tauri_webview_window_set_always_on_top(this.#handle, alwaysOnTop), 'window.setAlwaysOnTop')
  }
  setDecorations(decorations: boolean) {
    check(sym.tauri_webview_window_set_decorations(this.#handle, decorations), 'window.setDecorations')
  }
  setFocus() {
    check(sym.tauri_webview_window_set_focus(this.#handle), 'window.setFocus')
  }
  setZoom(scale: number) {
    check(sym.tauri_webview_window_set_zoom(this.#handle, scale), 'window.setZoom')
  }
  setClosable(closable: boolean) {
    check(sym.tauri_webview_window_set_closable(this.#handle, closable), 'window.setClosable')
  }
  setMaximizable(maximizable: boolean) {
    check(sym.tauri_webview_window_set_maximizable(this.#handle, maximizable), 'window.setMaximizable')
  }
  setMinimizable(minimizable: boolean) {
    check(sym.tauri_webview_window_set_minimizable(this.#handle, minimizable), 'window.setMinimizable')
  }
  setAlwaysOnBottom(alwaysOnBottom: boolean) {
    check(sym.tauri_webview_window_set_always_on_bottom(this.#handle, alwaysOnBottom), 'window.setAlwaysOnBottom')
  }
  setContentProtected(protectedContent: boolean) {
    check(sym.tauri_webview_window_set_content_protected(this.#handle, protectedContent), 'window.setContentProtected')
  }
  setSkipTaskbar(skip: boolean) {
    check(sym.tauri_webview_window_set_skip_taskbar(this.#handle, skip), 'window.setSkipTaskbar')
  }
  setShadow(enable: boolean) {
    check(sym.tauri_webview_window_set_shadow(this.#handle, enable), 'window.setShadow')
  }
  setVisibleOnAllWorkspaces(visible: boolean) {
    check(sym.tauri_webview_window_set_visible_on_all_workspaces(this.#handle, visible), 'window.setVisibleOnAllWorkspaces')
  }
  setIgnoreCursorEvents(ignore: boolean) {
    check(sym.tauri_webview_window_set_ignore_cursor_events(this.#handle, ignore), 'window.setIgnoreCursorEvents')
  }
  setCursorVisible(visible: boolean) {
    check(sym.tauri_webview_window_set_cursor_visible(this.#handle, visible), 'window.setCursorVisible')
  }
  setCursorGrab(grab: boolean) {
    check(sym.tauri_webview_window_set_cursor_grab(this.#handle, grab), 'window.setCursorGrab')
  }
  setEnabled(enabled: boolean) {
    check(sym.tauri_webview_window_set_enabled(this.#handle, enabled), 'window.setEnabled')
  }
  setFocusable(focusable: boolean) {
    check(sym.tauri_webview_window_set_focusable(this.#handle, focusable), 'window.setFocusable')
  }
  setSimpleFullscreen(enable: boolean) {
    check(sym.tauri_webview_window_set_simple_fullscreen(this.#handle, enable), 'window.setSimpleFullscreen')
  }
  /** Sizes are logical (DPI-scaled) unless `physical: true`; omit or pass a
   * non-positive size to clear the constraint. */
  setMinSize(
    { width = 0, height = 0, physical = false }: { width?: number; height?: number; physical?: boolean } = {}
  ) {
    check(sym.tauri_webview_window_set_min_size(this.#handle, width, height, physical), 'window.setMinSize')
  }
  setMaxSize(
    { width = 0, height = 0, physical = false }: { width?: number; height?: number; physical?: boolean } = {}
  ) {
    check(sym.tauri_webview_window_set_max_size(this.#handle, width, height, physical), 'window.setMaxSize')
  }
  setCursorPosition({ x, y, physical = false }: { x: number; y: number; physical?: boolean }) {
    check(sym.tauri_webview_window_set_cursor_position(this.#handle, x, y, physical), 'window.setCursorPosition')
  }
  /** Pass 'light', 'dark', or null/'' to follow the system theme. */
  setTheme(theme: string | null) {
    check(sym.tauri_webview_window_set_theme(this.#handle, cstr(theme ?? '')), 'window.setTheme')
  }
  /** e.g. 'default', 'pointer', 'crosshair', 'grab', 'wait'. */
  setCursorIcon(icon: string) {
    check(sym.tauri_webview_window_set_cursor_icon(this.#handle, cstr(icon)), 'window.setCursorIcon')
  }
  /** Pass 'critical', 'informational', or null/'' to cancel. */
  requestUserAttention(kind: string | null) {
    check(sym.tauri_webview_window_request_user_attention(this.#handle, cstr(kind ?? '')), 'window.requestUserAttention')
  }
  /** { status, progress } — see Tauri's ProgressBarState. */
  setProgressBar(state: Json) {
    check(sym.tauri_webview_window_set_progress_bar(this.#handle, cstr(JSON.stringify(state ?? {}))), 'window.setProgressBar')
  }
  /** A WindowEffectsConfig object, or null to clear effects. */
  setEffects(effects: Json) {
    check(sym.tauri_webview_window_set_effects(this.#handle, cstr(JSON.stringify(effects ?? null))), 'window.setEffects')
  }
  /** A WindowSizeConstraints object, e.g. { minWidth: { Logical: 400 } }. */
  setSizeConstraints(constraints: Json) {
    check(
      sym.tauri_webview_window_set_size_constraints(this.#handle, cstr(JSON.stringify(constraints ?? {}))),
      'window.setSizeConstraints'
    )
  }
  /** RGBA channels 0-255. */
  setBackgroundColor({ r, g, b, a = 255 }: { r: number; g: number; b: number; a?: number }) {
    check(sym.tauri_webview_window_set_background_color(this.#handle, r, g, b, a), 'window.setBackgroundColor')
  }
  /** Pass a negative count to clear the badge. */
  setBadgeCount(count: number | null) {
    check(sym.tauri_webview_window_set_badge_count(this.#handle, count ?? -1), 'window.setBadgeCount')
  }
  /** macOS only: the dock badge label; null/'' clears it. */
  setBadgeLabel(label: string | null) {
    check(sym.tauri_webview_window_set_badge_label(this.#handle, cstr(label ?? '')), 'window.setBadgeLabel')
  }
  /** macOS only: 'visible', 'transparent' or 'overlay'. */
  setTitleBarStyle(style: string) {
    check(sym.tauri_webview_window_set_title_bar_style(this.#handle, cstr(style)), 'window.setTitleBarStyle')
  }
  /** Windows only: RGBA pixels (width*height*4 bytes), or null to clear. */
  setOverlayIcon(rgba: Uint8Array | null, width = 0, height = 0) {
    check(sym.tauri_webview_window_set_overlay_icon(this.#handle, rgba, width, height), 'window.setOverlayIcon')
  }
  /** macOS only: the NSWindow pointer, as an integer. */
  nsWindow(): bigint {
    const slot = new BigUint64Array(1)
    check(sym.tauri_webview_window_ns_window(this.#handle, new Uint8Array(slot.buffer)), 'window.nsWindow')
    return slot[0]
  }
  /** macOS only: the NSView pointer, as an integer. */
  nsView(): bigint {
    const slot = new BigUint64Array(1)
    check(sym.tauri_webview_window_ns_view(this.#handle, new Uint8Array(slot.buffer)), 'window.nsView')
    return slot[0]
  }
  /** Windows only: the window's HWND, as an integer. */
  hwnd(): bigint {
    const slot = new BigUint64Array(1)
    check(sym.tauri_webview_window_hwnd(this.#handle, new Uint8Array(slot.buffer)), 'window.hwnd')
    return slot[0]
  }
  show() {
    check(sym.tauri_webview_window_show(this.#handle), 'window.show')
  }
  hide() {
    check(sym.tauri_webview_window_hide(this.#handle), 'window.hide')
  }
  center() {
    check(sym.tauri_webview_window_center(this.#handle), 'window.center')
  }
  maximize() {
    check(sym.tauri_webview_window_maximize(this.#handle), 'window.maximize')
  }
  unmaximize() {
    check(sym.tauri_webview_window_unmaximize(this.#handle), 'window.unmaximize')
  }
  minimize() {
    check(sym.tauri_webview_window_minimize(this.#handle), 'window.minimize')
  }
  unminimize() {
    check(sym.tauri_webview_window_unminimize(this.#handle), 'window.unminimize')
  }
  close() {
    check(sym.tauri_webview_window_close(this.#handle), 'window.close')
  }
  destroy() {
    check(sym.tauri_webview_window_destroy(this.#handle), 'window.destroy')
  }

  eval(js: string) {
    check(sym.tauri_webview_window_eval(this.#handle, cstr(js)), 'window.eval')
  }
  navigate(url: string | URL) {
    check(sym.tauri_webview_window_navigate(this.#handle, cstr(String(url))), 'window.navigate')
  }
  reload() {
    check(sym.tauri_webview_window_reload(this.#handle), 'window.reload')
  }
  startDragging() {
    check(sym.tauri_webview_window_start_dragging(this.#handle), 'window.startDragging')
  }
  print() {
    check(sym.tauri_webview_window_print(this.#handle), 'window.print')
  }
  clearAllBrowsingData() {
    check(sym.tauri_webview_window_clear_all_browsing_data(this.#handle), 'window.clearAllBrowsingData')
  }
  hideMenu() {
    check(sym.tauri_webview_window_hide_menu(this.#handle), 'window.hideMenu')
  }
  showMenu() {
    check(sym.tauri_webview_window_show_menu(this.#handle), 'window.showMenu')
  }
  /** Requires a debug build or the tauri-ffi `devtools` feature. */
  openDevtools() {
    check(sym.tauri_webview_window_open_devtools(this.#handle), 'window.openDevtools')
  }
  closeDevtools() {
    check(sym.tauri_webview_window_close_devtools(this.#handle), 'window.closeDevtools')
  }

  /** Releases the handle; the window itself is unaffected. */
  free() {
    check(sym.tauri_handle_close(this.#handle), 'window.free')
  }
}

/** Platform base-directory kinds accepted by `app.path.<kind>Dir()`. */
const PATH_KINDS = [
  'appConfig', 'appData', 'appLocalData', 'appCache', 'appLog', 'audio', 'cache',
  'config', 'data', 'localData', 'desktop', 'document', 'download', 'executable',
  'font', 'home', 'picture', 'public', 'resource', 'runtime', 'template', 'video', 'temp'
]

/**
 * Mirrors `tauri::tray::TrayIcon`. Obtain via `app.createTray()`; call `free()`
 * when done (dropping the last handle removes the icon). Menus are not yet
 * exposed — set an icon, tooltip, title and listen for 'tray-event' via
 * `app.on('tray-event', handler)`.
 */
export class Tray {
  #handle: bigint
  constructor(handle: bigint) {
    this.#handle = handle
  }
  id(): string {
    const slot = new BigUint64Array(1)
    check(sym.tauri_tray_id(this.#handle, new Uint8Array(slot.buffer)), 'tray.id')
    return takeString(slot) ?? ''
  }
  /** Icon from a PNG/ICO file path; pass null/'' to clear. */
  setIcon(path: string | null) {
    check(sym.tauri_tray_set_icon(this.#handle, cstr(path ?? '')), 'tray.setIcon')
  }
  setIconAsTemplate(isTemplate: boolean) {
    check(sym.tauri_tray_set_icon_as_template(this.#handle, isTemplate), 'tray.setIconAsTemplate')
  }
  setTooltip(tooltip: string | null) {
    check(sym.tauri_tray_set_tooltip(this.#handle, cstr(tooltip ?? '')), 'tray.setTooltip')
  }
  setTitle(title: string | null) {
    check(sym.tauri_tray_set_title(this.#handle, cstr(title ?? '')), 'tray.setTitle')
  }
  setVisible(visible: boolean) {
    check(sym.tauri_tray_set_visible(this.#handle, visible), 'tray.setVisible')
  }
  setShowMenuOnLeftClick(enable: boolean) {
    check(sym.tauri_tray_set_show_menu_on_left_click(this.#handle, enable), 'tray.setShowMenuOnLeftClick')
  }
  /** Sets (or clears, when menu is null) the tray context menu. */
  setMenu(menu: Menu | null) {
    check(sym.tauri_tray_set_menu(this.#handle, menu ? menu.handle : 0n), 'tray.setMenu')
  }
  /** Releases the handle; dropping the last handle removes the icon. */
  free() {
    check(sym.tauri_handle_close(this.#handle), 'tray.free')
  }
}

// Shared low-level helpers for the window/webview handle classes below.
function outStr(fn: (h: bigint, out: Uint8Array) => number, handle: bigint, what: string): string {
  const slot = new BigUint64Array(1)
  check(fn(handle, new Uint8Array(slot.buffer)), what)
  return takeString(slot) ?? ''
}
function outBool(fn: (h: bigint, out: Uint8Array) => number, handle: bigint, what: string): boolean {
  const out = new Uint8Array(1)
  check(fn(handle, out), what)
  return out[0] !== 0
}
function outPair(
  fn: (h: bigint, a: Uint8Array, b: Uint8Array) => number,
  handle: bigint,
  Kind: typeof Int32Array | typeof Uint32Array | typeof Float64Array,
  what: string
): [number, number] {
  const a = new Kind(1)
  const b = new Kind(1)
  check(fn(handle, new Uint8Array(a.buffer), new Uint8Array(b.buffer)), what)
  return [a[0], b[0]]
}

/** Mirrors `tauri::Window` — a bare OS window that can host {@link Webview}s. */
export class Window {
  #handle: bigint
  constructor(handle: bigint) {
    this.#handle = handle
  }
  get handle(): bigint {
    return this.#handle
  }
  async addWebview(
    config: Json,
    { x = 0, y = 0, width, height, physical = false }: { x?: number; y?: number; width: number; height: number; physical?: boolean }
  ): Promise<Webview> {
    const slot = new BigUint64Array(1)
    check(
      await sym.tauri_window_add_webview(this.#handle, cstr(JSON.stringify(config)), x, y, width, height, physical, new Uint8Array(slot.buffer)),
      'window.addWebview'
    )
    return new Webview(slot[0])
  }
  webviews(): string[] {
    return JSON.parse(outStr(sym.tauri_window_webviews, this.#handle, 'window.webviews') || '[]')
  }
  label(): string { return outStr(sym.tauri_window_label, this.#handle, 'window.label') }
  title(): string { return outStr(sym.tauri_window_title, this.#handle, 'window.title') }
  theme(): string { return outStr(sym.tauri_window_theme, this.#handle, 'window.theme') }
  scaleFactor(): number { const o = new Float64Array(1); check(sym.tauri_window_scale_factor(this.#handle, new Uint8Array(o.buffer)), 'window.scaleFactor'); return o[0] }
  innerSize(): { width: number; height: number } { const [width, height] = outPair(sym.tauri_window_inner_size, this.#handle, Uint32Array, 'window.innerSize'); return { width, height } }
  outerSize(): { width: number; height: number } { const [width, height] = outPair(sym.tauri_window_outer_size, this.#handle, Uint32Array, 'window.outerSize'); return { width, height } }
  innerPosition(): { x: number; y: number } { const [x, y] = outPair(sym.tauri_window_inner_position, this.#handle, Int32Array, 'window.innerPosition'); return { x, y } }
  outerPosition(): { x: number; y: number } { const [x, y] = outPair(sym.tauri_window_outer_position, this.#handle, Int32Array, 'window.outerPosition'); return { x, y } }
  cursorPosition(): { x: number; y: number } { const [x, y] = outPair(sym.tauri_window_cursor_position, this.#handle, Float64Array, 'window.cursorPosition'); return { x, y } }
  isVisible(): boolean { return outBool(sym.tauri_window_is_visible, this.#handle, 'window.isVisible') }
  isFocused(): boolean { return outBool(sym.tauri_window_is_focused, this.#handle, 'window.isFocused') }
  isFullscreen(): boolean { return outBool(sym.tauri_window_is_fullscreen, this.#handle, 'window.isFullscreen') }
  isMaximized(): boolean { return outBool(sym.tauri_window_is_maximized, this.#handle, 'window.isMaximized') }
  isMinimized(): boolean { return outBool(sym.tauri_window_is_minimized, this.#handle, 'window.isMinimized') }
  isResizable(): boolean { return outBool(sym.tauri_window_is_resizable, this.#handle, 'window.isResizable') }
  isDecorated(): boolean { return outBool(sym.tauri_window_is_decorated, this.#handle, 'window.isDecorated') }
  isClosable(): boolean { return outBool(sym.tauri_window_is_closable, this.#handle, 'window.isClosable') }
  isMaximizable(): boolean { return outBool(sym.tauri_window_is_maximizable, this.#handle, 'window.isMaximizable') }
  isMinimizable(): boolean { return outBool(sym.tauri_window_is_minimizable, this.#handle, 'window.isMinimizable') }
  isAlwaysOnTop(): boolean { return outBool(sym.tauri_window_is_always_on_top, this.#handle, 'window.isAlwaysOnTop') }
  isEnabled(): boolean { return outBool(sym.tauri_window_is_enabled, this.#handle, 'window.isEnabled') }
  isMenuVisible(): boolean { return outBool(sym.tauri_window_is_menu_visible, this.#handle, 'window.isMenuVisible') }
  availableMonitors(): Json[] { return JSON.parse(outStr(sym.tauri_window_available_monitors, this.#handle, 'window.availableMonitors') || '[]') }
  currentMonitor(): Json { return JSON.parse(outStr(sym.tauri_window_current_monitor, this.#handle, 'window.currentMonitor') || 'null') }
  primaryMonitor(): Json { return JSON.parse(outStr(sym.tauri_window_primary_monitor, this.#handle, 'window.primaryMonitor') || 'null') }
  monitorFromPoint(x: number, y: number): Json { const s = new BigUint64Array(1); check(sym.tauri_window_monitor_from_point(this.#handle, x, y, new Uint8Array(s.buffer)), 'window.monitorFromPoint'); return JSON.parse(takeString(s) ?? 'null') }
  setTitle(title: string) { check(sym.tauri_window_set_title(this.#handle, cstr(title)), 'window.setTitle') }
  setSize({ width, height, physical = false }: { width: number; height: number; physical?: boolean }) { check(sym.tauri_window_set_size(this.#handle, width, height, physical), 'window.setSize') }
  setPosition({ x, y, physical = false }: { x: number; y: number; physical?: boolean }) { check(sym.tauri_window_set_position(this.#handle, x, y, physical), 'window.setPosition') }
  setMinSize({ width = 0, height = 0, physical = false }: { width?: number; height?: number; physical?: boolean } = {}) { check(sym.tauri_window_set_min_size(this.#handle, width, height, physical), 'window.setMinSize') }
  setMaxSize({ width = 0, height = 0, physical = false }: { width?: number; height?: number; physical?: boolean } = {}) { check(sym.tauri_window_set_max_size(this.#handle, width, height, physical), 'window.setMaxSize') }
  setCursorPosition({ x, y, physical = false }: { x: number; y: number; physical?: boolean }) { check(sym.tauri_window_set_cursor_position(this.#handle, x, y, physical), 'window.setCursorPosition') }
  setFullscreen(v: boolean) { check(sym.tauri_window_set_fullscreen(this.#handle, v), 'window.setFullscreen') }
  setResizable(v: boolean) { check(sym.tauri_window_set_resizable(this.#handle, v), 'window.setResizable') }
  setAlwaysOnTop(v: boolean) { check(sym.tauri_window_set_always_on_top(this.#handle, v), 'window.setAlwaysOnTop') }
  setAlwaysOnBottom(v: boolean) { check(sym.tauri_window_set_always_on_bottom(this.#handle, v), 'window.setAlwaysOnBottom') }
  setDecorations(v: boolean) { check(sym.tauri_window_set_decorations(this.#handle, v), 'window.setDecorations') }
  setClosable(v: boolean) { check(sym.tauri_window_set_closable(this.#handle, v), 'window.setClosable') }
  setMaximizable(v: boolean) { check(sym.tauri_window_set_maximizable(this.#handle, v), 'window.setMaximizable') }
  setMinimizable(v: boolean) { check(sym.tauri_window_set_minimizable(this.#handle, v), 'window.setMinimizable') }
  setContentProtected(v: boolean) { check(sym.tauri_window_set_content_protected(this.#handle, v), 'window.setContentProtected') }
  setSkipTaskbar(v: boolean) { check(sym.tauri_window_set_skip_taskbar(this.#handle, v), 'window.setSkipTaskbar') }
  setShadow(v: boolean) { check(sym.tauri_window_set_shadow(this.#handle, v), 'window.setShadow') }
  setVisibleOnAllWorkspaces(v: boolean) { check(sym.tauri_window_set_visible_on_all_workspaces(this.#handle, v), 'window.setVisibleOnAllWorkspaces') }
  setIgnoreCursorEvents(v: boolean) { check(sym.tauri_window_set_ignore_cursor_events(this.#handle, v), 'window.setIgnoreCursorEvents') }
  setCursorVisible(v: boolean) { check(sym.tauri_window_set_cursor_visible(this.#handle, v), 'window.setCursorVisible') }
  setCursorGrab(v: boolean) { check(sym.tauri_window_set_cursor_grab(this.#handle, v), 'window.setCursorGrab') }
  setEnabled(v: boolean) { check(sym.tauri_window_set_enabled(this.#handle, v), 'window.setEnabled') }
  setFocusable(v: boolean) { check(sym.tauri_window_set_focusable(this.#handle, v), 'window.setFocusable') }
  setSimpleFullscreen(v: boolean) { check(sym.tauri_window_set_simple_fullscreen(this.#handle, v), 'window.setSimpleFullscreen') }
  setTheme(theme: string | null) { check(sym.tauri_window_set_theme(this.#handle, cstr(theme ?? '')), 'window.setTheme') }
  setCursorIcon(icon: string) { check(sym.tauri_window_set_cursor_icon(this.#handle, cstr(icon)), 'window.setCursorIcon') }
  requestUserAttention(kind: string | null) { check(sym.tauri_window_request_user_attention(this.#handle, cstr(kind ?? '')), 'window.requestUserAttention') }
  setProgressBar(state: Json) { check(sym.tauri_window_set_progress_bar(this.#handle, cstr(JSON.stringify(state ?? {}))), 'window.setProgressBar') }
  setEffects(effects: Json) { check(sym.tauri_window_set_effects(this.#handle, cstr(JSON.stringify(effects ?? null))), 'window.setEffects') }
  setSizeConstraints(c: Json) { check(sym.tauri_window_set_size_constraints(this.#handle, cstr(JSON.stringify(c ?? {}))), 'window.setSizeConstraints') }
  setBackgroundColor({ r, g, b, a = 255 }: { r: number; g: number; b: number; a?: number }) { check(sym.tauri_window_set_background_color(this.#handle, r, g, b, a), 'window.setBackgroundColor') }
  setBadgeCount(count: number | null) { check(sym.tauri_window_set_badge_count(this.#handle, count ?? -1), 'window.setBadgeCount') }
  setBadgeLabel(label: string | null) { check(sym.tauri_window_set_badge_label(this.#handle, cstr(label ?? '')), 'window.setBadgeLabel') }
  /** macOS only: 'visible', 'transparent' or 'overlay'. */
  setTitleBarStyle(style: string) { check(sym.tauri_window_set_title_bar_style(this.#handle, cstr(style)), 'window.setTitleBarStyle') }
  /** Windows only: RGBA pixels (width*height*4 bytes), or null to clear. */
  setOverlayIcon(rgba: Uint8Array | null, width = 0, height = 0) { check(sym.tauri_window_set_overlay_icon(this.#handle, rgba, width, height), 'window.setOverlayIcon') }
  /** macOS only: the NSWindow pointer, as an integer. */
  nsWindow(): bigint { const s = new BigUint64Array(1); check(sym.tauri_window_ns_window(this.#handle, new Uint8Array(s.buffer)), 'window.nsWindow'); return s[0] }
  /** macOS only: the NSView pointer, as an integer. */
  nsView(): bigint { const s = new BigUint64Array(1); check(sym.tauri_window_ns_view(this.#handle, new Uint8Array(s.buffer)), 'window.nsView'); return s[0] }
  /** Windows only: the window's HWND, as an integer. */
  hwnd(): bigint { const s = new BigUint64Array(1); check(sym.tauri_window_hwnd(this.#handle, new Uint8Array(s.buffer)), 'window.hwnd'); return s[0] }
  setFocus() { check(sym.tauri_window_set_focus(this.#handle), 'window.setFocus') }
  show() { check(sym.tauri_window_show(this.#handle), 'window.show') }
  hide() { check(sym.tauri_window_hide(this.#handle), 'window.hide') }
  center() { check(sym.tauri_window_center(this.#handle), 'window.center') }
  maximize() { check(sym.tauri_window_maximize(this.#handle), 'window.maximize') }
  unmaximize() { check(sym.tauri_window_unmaximize(this.#handle), 'window.unmaximize') }
  minimize() { check(sym.tauri_window_minimize(this.#handle), 'window.minimize') }
  unminimize() { check(sym.tauri_window_unminimize(this.#handle), 'window.unminimize') }
  close() { check(sym.tauri_window_close(this.#handle), 'window.close') }
  destroy() { check(sym.tauri_window_destroy(this.#handle), 'window.destroy') }
  startDragging() { check(sym.tauri_window_start_dragging(this.#handle), 'window.startDragging') }
  hideMenu() { check(sym.tauri_window_hide_menu(this.#handle), 'window.hideMenu') }
  showMenu() { check(sym.tauri_window_show_menu(this.#handle), 'window.showMenu') }
  setMenu(menu: Menu) { check(sym.tauri_menu_set_as_window_menu(this.#handle, menu.handle), 'window.setMenu') }
  free() { check(sym.tauri_handle_close(this.#handle), 'window.free') }
}

/** Mirrors `tauri::Webview` — a webview hosted inside a {@link Window}. */
export class Webview {
  #handle: bigint
  constructor(handle: bigint) {
    this.#handle = handle
  }
  get handle(): bigint {
    return this.#handle
  }
  label(): string { return outStr(sym.tauri_webview_label, this.#handle, 'webview.label') }
  url(): string { return outStr(sym.tauri_webview_url, this.#handle, 'webview.url') }
  position(): { x: number; y: number } { const [x, y] = outPair(sym.tauri_webview_position, this.#handle, Int32Array, 'webview.position'); return { x, y } }
  size(): { width: number; height: number } { const [width, height] = outPair(sym.tauri_webview_size, this.#handle, Uint32Array, 'webview.size'); return { width, height } }
  window(): Window { const s = new BigUint64Array(1); check(sym.tauri_webview_get_window(this.#handle, new Uint8Array(s.buffer)), 'webview.window'); return new Window(s[0]) }
  eval(js: string) { check(sym.tauri_webview_eval(this.#handle, cstr(js)), 'webview.eval') }
  navigate(url: string | URL) { check(sym.tauri_webview_navigate(this.#handle, cstr(String(url))), 'webview.navigate') }
  reload() { check(sym.tauri_webview_reload(this.#handle), 'webview.reload') }
  print() { check(sym.tauri_webview_print(this.#handle), 'webview.print') }
  setZoom(scale: number) { check(sym.tauri_webview_set_zoom(this.#handle, scale), 'webview.setZoom') }
  setAutoResize(v: boolean) { check(sym.tauri_webview_set_auto_resize(this.#handle, v), 'webview.setAutoResize') }
  setSize({ width, height, physical = false }: { width: number; height: number; physical?: boolean }) { check(sym.tauri_webview_set_size(this.#handle, width, height, physical), 'webview.setSize') }
  setPosition({ x, y, physical = false }: { x: number; y: number; physical?: boolean }) { check(sym.tauri_webview_set_position(this.#handle, x, y, physical), 'webview.setPosition') }
  setBackgroundColor({ r, g, b, a = 255 }: { r: number; g: number; b: number; a?: number }) { check(sym.tauri_webview_set_background_color(this.#handle, r, g, b, a), 'webview.setBackgroundColor') }
  setFocus() { check(sym.tauri_webview_set_focus(this.#handle), 'webview.setFocus') }
  show() { check(sym.tauri_webview_show(this.#handle), 'webview.show') }
  hide() { check(sym.tauri_webview_hide(this.#handle), 'webview.hide') }
  close() { check(sym.tauri_webview_close(this.#handle), 'webview.close') }
  clearAllBrowsingData() { check(sym.tauri_webview_clear_all_browsing_data(this.#handle), 'webview.clearAllBrowsingData') }
  reparent(window: Window) { check(sym.tauri_webview_reparent(this.#handle, window.handle), 'webview.reparent') }
  openDevtools() { check(sym.tauri_webview_open_devtools(this.#handle), 'webview.openDevtools') }
  closeDevtools() { check(sym.tauri_webview_close_devtools(this.#handle), 'webview.closeDevtools') }
  isDevtoolsOpen(): boolean { return outBool(sym.tauri_webview_is_devtools_open, this.#handle, 'webview.isDevtoolsOpen') }
  free() { check(sym.tauri_handle_close(this.#handle), 'webview.free') }
}

/** A menu item (`tauri::menu::MenuItemKind`). */
export class MenuItem {
  #handle: bigint
  constructor(handle: bigint) {
    this.#handle = handle
  }
  get handle(): bigint {
    return this.#handle
  }
  id(): string { return outStr(sym.tauri_menu_item_id, this.#handle, 'menuItem.id') }
  setText(text: string) { check(sym.tauri_menu_item_set_text(this.#handle, cstr(text)), 'menuItem.setText') }
  setEnabled(v: boolean) { check(sym.tauri_menu_item_set_enabled(this.#handle, v), 'menuItem.setEnabled') }
  setChecked(v: boolean) { check(sym.tauri_menu_item_set_checked(this.#handle, v), 'menuItem.setChecked') }
  setAccelerator(accel: string | null) { check(sym.tauri_menu_item_set_accelerator(this.#handle, cstr(accel ?? '')), 'menuItem.setAccelerator') }
  /** Appends a child (only valid on a submenu). */
  append(item: MenuItem) { check(sym.tauri_submenu_append(this.#handle, item.handle), 'submenu.append') }
  free() { check(sym.tauri_handle_close(this.#handle), 'menuItem.free') }
}

/** A menu (`tauri::menu::Menu`). */
export class Menu {
  #handle: bigint
  constructor(handle: bigint) {
    this.#handle = handle
  }
  get handle(): bigint {
    return this.#handle
  }
  append(item: MenuItem) { check(sym.tauri_menu_append(this.#handle, item.handle), 'menu.append') }
  free() { check(sym.tauri_handle_close(this.#handle), 'menu.free') }
}

/** The worker-side app API — see {@linkcode app}. */
export interface AppApi {
  /** Handle a command registered in launch(): `handler(payload, { webview })`. */
  command(name: string, handler: CommandHandler): AppApi
  /** Applies a plugin (from `definePlugin()`) in this worker, wiring up its
   * command handlers. Pass the same plugin to `launch({ plugins })`. */
  plugin(plugin: Plugin): AppApi
  /** Serves a custom URI scheme registered in `launch({ protocols })`. Throwing
   * produces a 500 and an unhandled scheme a 404 — a request must always be
   * answered or the webview waits forever. */
  protocol(scheme: string, handler: ProtocolHandler): AppApi
  /** Lifecycle events: 'ready' | 'exit' | 'exit-requested' | 'window-event' |
   * 'page-load' | 'webview-event' | 'menu-event' | 'tray-event' |
   * 'single-instance' (the last four require the matching launch() option). */
  on(type: string, handler: (message: Json) => void): AppApi
  /** Listen to a Tauri event from any source (frontend `emit`, host `emit`). */
  listen(event: string, handler: (payload: Json, message: Json) => void): number
  unlisten(id: number): void
  emit(event: string, payload?: Json): void
  emitTo(label: string, event: string, payload?: Json): void
  /** Creates a webview window from a WindowConfig object (async: creation
   * blocks on the event loop, declared nonblocking in symbols.ts). */
  createWindow(config: { label?: string } & Json): Promise<WebviewWindow>
  /** The window with the given label, or null. */
  getWindow(label: string): WebviewWindow | null
  /** Labels of all webview windows. */
  windowLabels(): string[]
  /** The app's resolved configuration (tauri.conf.json shape). */
  config(): Json
  /** The app package info: { name, version, authors, description, crateName }. */
  packageInfo(): Json
  /** Platform base directories (`Manager::path`), e.g. `app.path.appDataDir()`. */
  path: Record<string, () => string>
  /** The focused webview window, or null. */
  getFocusedWindow(): WebviewWindow | null
  /** Adds a capability (JSON/TOML string or object) to the running app. */
  addCapability(capability: string | Json): void
  /** Like listen(), but auto-removed after the first delivery. */
  once(event: string, handler: (payload: Json, message: Json) => void): number
  /** Sets the app-wide theme: 'light', 'dark', or null to follow the system. */
  setTheme(theme: string | null): void
  /** The current cursor position, in physical screen pixels. */
  cursorPosition(): { x: number; y: number }
  /** Requests an app restart (exits and relaunches). */
  requestRestart(): void
  /** macOS only: 'regular', 'accessory' (no Dock icon) or 'prohibited'. */
  setActivationPolicy(policy: string): void
  /** macOS only: shows or hides the app's Dock icon. */
  setDockVisibility(visible: boolean): void
  /** macOS only: shows the application without focusing it. */
  show(): void
  /** macOS only: hides the application, like Cmd+H. */
  hide(): void
  /** Creates a system tray icon. Listen via `app.on('tray-event', handler)`. */
  createTray(id?: string): Tray
  removeTrayById(id: string): void
  /** Creates a bare OS window (no webview) that can host webviews. */
  createBareWindow(config: Json): Promise<Window>
  /** The bare window with the given label, or null. */
  getBareWindow(label: string): Window | null
  /** Labels of all bare windows. */
  bareWindowLabels(): string[]
  /** Creates an empty menu. */
  createMenu(): Menu
  /** A normal menu item. */
  menuItem(opts: { id?: string; text: string; enabled?: boolean; accelerator?: string }): MenuItem
  /** A checkable menu item. */
  checkMenuItem(opts: { id?: string; text: string; enabled?: boolean; checked?: boolean; accelerator?: string }): MenuItem
  /** A predefined menu item ('separator', 'copy', 'quit', …). */
  predefinedMenuItem(kind: string, text?: string): MenuItem
  /** A submenu. */
  submenu(opts: { id?: string; text: string; enabled?: boolean }): MenuItem
  /** Sets a menu as the app-wide menu (macOS menu bar). */
  setAppMenu(menu: Menu): void
  exit(code?: number): void
}

export const app: AppApi = {
  /** Handle a command registered in launch(): `handler(payload, { webview })`. */
  command(name: string, handler: CommandHandler) {
    commands.set(name, handler)
    return app
  },

  /** Applies a plugin (from `definePlugin()`) in this worker, wiring up its
   * command handlers. Pass the same plugin to `launch({ plugins })`. */
  plugin(plugin: Plugin) {
    for (const [name, handler] of plugin.handlers) {
      commands.set(`plugin:${plugin.name}|${name}`, handler)
    }
    return app
  },

  /** Serves a custom URI scheme registered in `launch({ protocols })`. */
  protocol(scheme: string, handler: ProtocolHandler) {
    protocols.set(scheme, handler)
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
    const code = await sym.tauri_webview_window_create(
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
    const code = sym.tauri_app_get_webview_window(appId, cstr(label), new Uint8Array(slot.buffer))
    if (code === CODES.NOT_FOUND) return null
    check(code, `getWindow(${label})`)
    return new WebviewWindow(slot[0])
  },

  /** Labels of all webview windows. */
  windowLabels(): string[] {
    const slot = new BigUint64Array(1)
    check(sym.tauri_app_webview_window_labels(appId, new Uint8Array(slot.buffer)), 'windowLabels')
    return JSON.parse(takeString(slot) ?? '[]')
  },

  /** The app's resolved configuration (tauri.conf.json shape). */
  config(): Json {
    const slot = new BigUint64Array(1)
    check(sym.tauri_app_config(appId, new Uint8Array(slot.buffer)), 'config')
    return JSON.parse(takeString(slot) ?? 'null')
  },

  /** The app package info: { name, version, authors, description, crateName }. */
  packageInfo(): Json {
    const slot = new BigUint64Array(1)
    check(sym.tauri_app_package_info(appId, new Uint8Array(slot.buffer)), 'packageInfo')
    return JSON.parse(takeString(slot) ?? 'null')
  },

  path: Object.fromEntries(
    PATH_KINDS.map((kind) => [
      `${kind}Dir`,
      (): string => {
        const slot = new BigUint64Array(1)
        check(sym.tauri_app_path(appId, cstr(kind), new Uint8Array(slot.buffer)), `path.${kind}Dir`)
        return takeString(slot) ?? ''
      }
    ])
  ),

  getFocusedWindow(): WebviewWindow | null {
    const slot = new BigUint64Array(1)
    const code = sym.tauri_app_get_focused_window(appId, new Uint8Array(slot.buffer))
    if (code === CODES.NOT_FOUND) return null
    check(code, 'getFocusedWindow')
    return new WebviewWindow(slot[0])
  },

  addCapability(capability: string | Json) {
    const json = typeof capability === 'string' ? capability : JSON.stringify(capability)
    check(sym.tauri_app_add_capability(appId, cstr(json)), 'addCapability')
  },

  once(event: string, handler: (payload: Json, message: Json) => void): number {
    const out = new Uint32Array(1)
    check(sym.tauri_app_once(appId, cstr(event), new Uint8Array(out.buffer)), `once(${event})`)
    const id = out[0]
    eventListeners.set(id, (payload: Json, message: Json) => {
      eventListeners.delete(id)
      handler(payload, message)
    })
    return id
  },

  setTheme(theme: string | null) {
    check(sym.tauri_app_set_theme(appId, cstr(theme ?? '')), 'setTheme')
  },

  cursorPosition(): { x: number; y: number } {
    const x = new Float64Array(1)
    const y = new Float64Array(1)
    check(
      sym.tauri_app_cursor_position(appId, new Uint8Array(x.buffer), new Uint8Array(y.buffer)),
      'cursorPosition'
    )
    return { x: x[0], y: y[0] }
  },

  requestRestart() {
    check(sym.tauri_app_request_restart(appId), 'requestRestart')
  },

  setActivationPolicy(policy: string) {
    check(sym.tauri_app_set_activation_policy(appId, cstr(policy)), 'setActivationPolicy')
  },

  setDockVisibility(visible: boolean) {
    check(sym.tauri_app_set_dock_visibility(appId, visible), 'setDockVisibility')
  },

  show() {
    check(sym.tauri_app_show(appId), 'show')
  },

  hide() {
    check(sym.tauri_app_hide(appId), 'hide')
  },

  createTray(id = ''): Tray {
    const slot = new BigUint64Array(1)
    check(sym.tauri_tray_new(appId, cstr(id), new Uint8Array(slot.buffer)), `createTray(${id})`)
    return new Tray(slot[0])
  },

  removeTrayById(id: string) {
    check(sym.tauri_app_remove_tray_by_id(appId, cstr(id)), 'removeTrayById')
  },

  async createBareWindow(config: Json): Promise<Window> {
    const slot = new BigUint64Array(1)
    check(await sym.tauri_window_create(appId, cstr(JSON.stringify(config)), new Uint8Array(slot.buffer)), `createBareWindow(${config?.label})`)
    return new Window(slot[0])
  },

  getBareWindow(label: string): Window | null {
    const slot = new BigUint64Array(1)
    const code = sym.tauri_app_get_window(appId, cstr(label), new Uint8Array(slot.buffer))
    if (code === CODES.NOT_FOUND) return null
    check(code, `getBareWindow(${label})`)
    return new Window(slot[0])
  },

  bareWindowLabels(): string[] {
    const slot = new BigUint64Array(1)
    check(sym.tauri_app_window_labels(appId, new Uint8Array(slot.buffer)), 'bareWindowLabels')
    return JSON.parse(takeString(slot) ?? '[]')
  },

  createMenu(): Menu {
    const slot = new BigUint64Array(1)
    check(sym.tauri_menu_new(appId, new Uint8Array(slot.buffer)), 'createMenu')
    return new Menu(slot[0])
  },

  menuItem({ id = '', text, enabled = true, accelerator = '' }: { id?: string; text: string; enabled?: boolean; accelerator?: string }): MenuItem {
    const slot = new BigUint64Array(1)
    check(sym.tauri_menu_item_new(appId, cstr(id), cstr(text), enabled, cstr(accelerator), new Uint8Array(slot.buffer)), 'menuItem')
    return new MenuItem(slot[0])
  },

  checkMenuItem({ id = '', text, enabled = true, checked = false, accelerator = '' }: { id?: string; text: string; enabled?: boolean; checked?: boolean; accelerator?: string }): MenuItem {
    const slot = new BigUint64Array(1)
    check(sym.tauri_menu_check_item_new(appId, cstr(id), cstr(text), enabled, checked, cstr(accelerator), new Uint8Array(slot.buffer)), 'checkMenuItem')
    return new MenuItem(slot[0])
  },

  predefinedMenuItem(kind: string, text = ''): MenuItem {
    const slot = new BigUint64Array(1)
    check(sym.tauri_menu_predefined_item_new(appId, cstr(kind), cstr(text), new Uint8Array(slot.buffer)), 'predefinedMenuItem')
    return new MenuItem(slot[0])
  },

  submenu({ id = '', text, enabled = true }: { id?: string; text: string; enabled?: boolean }): MenuItem {
    const slot = new BigUint64Array(1)
    check(sym.tauri_submenu_new(appId, cstr(id), cstr(text), enabled, new Uint8Array(slot.buffer)), 'submenu')
    return new MenuItem(slot[0])
  },

  setAppMenu(menu: Menu): void {
    check(sym.tauri_menu_set_as_app_menu(menu.handle), 'setAppMenu')
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
  // Plugin invokes are keyed by the full `plugin:<name>|<command>` string.
  const key = message.plugin ? `plugin:${message.plugin}|${message.command}` : message.command
  const handler = commands.get(key)
  if (!handler) {
    sym.tauri_invoke_reject(message.id, cstr(JSON.stringify(`command ${key} not found`)))
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

const encoder = new TextEncoder()

function base64Bytes(value: string): Uint8Array {
  const binary = atob(value)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return bytes
}

/** Normalizes a handler's return value into a status/headers/body triple. */
function toResponse(
  value: string | Uint8Array | ProtocolResponse | null
): { status: number; headers: Record<string, string | string[]>; body: Uint8Array } {
  if (value == null) return { status: 204, headers: {}, body: new Uint8Array() }
  if (typeof value === 'string') return { status: 200, headers: {}, body: encoder.encode(value) }
  if (value instanceof Uint8Array) return { status: 200, headers: {}, body: value }
  const { status = 200, headers = {}, body = '' } = value
  return { status, headers, body: typeof body === 'string' ? encoder.encode(body) : body }
}

async function handleProtocolRequest(message: Json) {
  const handler = protocols.get(message.scheme)
  let response
  try {
    response = handler
      ? toResponse(await handler({ ...message, body: base64Bytes(message.body ?? '') }))
      : toResponse({ status: 404, body: `no handler for ${message.scheme}://` })
  } catch (error) {
    console.error(`[tauri-ffi] protocol '${message.scheme}' handler threw:`, error)
    response = toResponse({ status: 500, body: error instanceof Error ? error.message : String(error) })
  }
  // Always responds: an unanswered request hangs the webview forever.
  check(
    sym.tauri_uri_scheme_respond(
      message.id,
      response.status,
      cstr(JSON.stringify(response.headers)),
      response.body,
      response.body.length
    ),
    'uri_scheme_respond'
  )
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
      case 'uri-scheme-request':
        handleProtocolRequest(message) // likewise
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
