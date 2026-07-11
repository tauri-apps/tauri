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
    check(api.webviewWindowLabel(this.#handle, out), 'window.label')
    return takeString(out)
  }
  title() {
    const out = [null]
    check(api.webviewWindowTitle(this.#handle, out), 'window.title')
    return takeString(out)
  }
  url() {
    const out = [null]
    check(api.webviewWindowUrl(this.#handle, out), 'window.url')
    return takeString(out)
  }
  scaleFactor() {
    const out = [0]
    check(api.webviewWindowScaleFactor(this.#handle, out), 'window.scaleFactor')
    return out[0]
  }
  innerSize() {
    const width = [0]
    const height = [0]
    check(api.webviewWindowInnerSize(this.#handle, width, height), 'window.innerSize')
    return { width: width[0], height: height[0] }
  }
  outerSize() {
    const width = [0]
    const height = [0]
    check(api.webviewWindowOuterSize(this.#handle, width, height), 'window.outerSize')
    return { width: width[0], height: height[0] }
  }
  innerPosition() {
    const x = [0]
    const y = [0]
    check(api.webviewWindowInnerPosition(this.#handle, x, y), 'window.innerPosition')
    return { x: x[0], y: y[0] }
  }
  outerPosition() {
    const x = [0]
    const y = [0]
    check(api.webviewWindowOuterPosition(this.#handle, x, y), 'window.outerPosition')
    return { x: x[0], y: y[0] }
  }

  theme() {
    const out = [null]
    check(api.webviewWindowTheme(this.#handle, out), 'window.theme')
    return takeString(out)
  }

  #bool(fn, what) {
    const out = [false]
    check(fn(this.#handle, out), what)
    return out[0]
  }
  isVisible() {
    return this.#bool(api.webviewWindowIsVisible, 'window.isVisible')
  }
  isFocused() {
    return this.#bool(api.webviewWindowIsFocused, 'window.isFocused')
  }
  isFullscreen() {
    return this.#bool(api.webviewWindowIsFullscreen, 'window.isFullscreen')
  }
  isMaximized() {
    return this.#bool(api.webviewWindowIsMaximized, 'window.isMaximized')
  }
  isMinimized() {
    return this.#bool(api.webviewWindowIsMinimized, 'window.isMinimized')
  }
  isResizable() {
    return this.#bool(api.webviewWindowIsResizable, 'window.isResizable')
  }
  isDecorated() {
    return this.#bool(api.webviewWindowIsDecorated, 'window.isDecorated')
  }
  isClosable() {
    return this.#bool(api.webviewWindowIsClosable, 'window.isClosable')
  }
  isMaximizable() {
    return this.#bool(api.webviewWindowIsMaximizable, 'window.isMaximizable')
  }
  isMinimizable() {
    return this.#bool(api.webviewWindowIsMinimizable, 'window.isMinimizable')
  }
  isAlwaysOnTop() {
    return this.#bool(api.webviewWindowIsAlwaysOnTop, 'window.isAlwaysOnTop')
  }
  isEnabled() {
    return this.#bool(api.webviewWindowIsEnabled, 'window.isEnabled')
  }
  isMenuVisible() {
    return this.#bool(api.webviewWindowIsMenuVisible, 'window.isMenuVisible')
  }
  isDevtoolsOpen() {
    return this.#bool(api.webviewWindowIsDevtoolsOpen, 'window.isDevtoolsOpen')
  }

  /** All available monitors, as Monitor objects. */
  availableMonitors() {
    const out = [null]
    check(api.webviewWindowAvailableMonitors(this.#handle, out), 'window.availableMonitors')
    return JSON.parse(takeString(out))
  }
  /** The monitor the window is on, or null. */
  currentMonitor() {
    const out = [null]
    check(api.webviewWindowCurrentMonitor(this.#handle, out), 'window.currentMonitor')
    return JSON.parse(takeString(out))
  }
  /** The primary monitor, or null. */
  primaryMonitor() {
    const out = [null]
    check(api.webviewWindowPrimaryMonitor(this.#handle, out), 'window.primaryMonitor')
    return JSON.parse(takeString(out))
  }
  /** The monitor containing the given physical point, or null. */
  monitorFromPoint(x, y) {
    const out = [null]
    check(api.webviewWindowMonitorFromPoint(this.#handle, x, y, out), 'window.monitorFromPoint')
    return JSON.parse(takeString(out))
  }
  /** The current cursor position, in physical pixels. */
  cursorPosition() {
    const x = [0]
    const y = [0]
    check(api.webviewWindowCursorPosition(this.#handle, x, y), 'window.cursorPosition')
    return { x: x[0], y: y[0] }
  }

  setTitle(title) {
    check(api.webviewWindowSetTitle(this.#handle, title), 'window.setTitle')
  }
  /** Sizes are logical (DPI-scaled) unless `physical: true`. */
  setSize({ width, height, physical = false }) {
    check(api.webviewWindowSetSize(this.#handle, width, height, physical), 'window.setSize')
  }
  setPosition({ x, y, physical = false }) {
    check(api.webviewWindowSetPosition(this.#handle, x, y, physical), 'window.setPosition')
  }
  setFullscreen(fullscreen) {
    check(api.webviewWindowSetFullscreen(this.#handle, fullscreen), 'window.setFullscreen')
  }
  setResizable(resizable) {
    check(api.webviewWindowSetResizable(this.#handle, resizable), 'window.setResizable')
  }
  setAlwaysOnTop(alwaysOnTop) {
    check(api.webviewWindowSetAlwaysOnTop(this.#handle, alwaysOnTop), 'window.setAlwaysOnTop')
  }
  setDecorations(decorations) {
    check(api.webviewWindowSetDecorations(this.#handle, decorations), 'window.setDecorations')
  }
  setFocus() {
    check(api.webviewWindowSetFocus(this.#handle), 'window.setFocus')
  }
  setZoom(scale) {
    check(api.webviewWindowSetZoom(this.#handle, scale), 'window.setZoom')
  }
  setClosable(closable) {
    check(api.webviewWindowSetClosable(this.#handle, closable), 'window.setClosable')
  }
  setMaximizable(maximizable) {
    check(api.webviewWindowSetMaximizable(this.#handle, maximizable), 'window.setMaximizable')
  }
  setMinimizable(minimizable) {
    check(api.webviewWindowSetMinimizable(this.#handle, minimizable), 'window.setMinimizable')
  }
  setAlwaysOnBottom(alwaysOnBottom) {
    check(api.webviewWindowSetAlwaysOnBottom(this.#handle, alwaysOnBottom), 'window.setAlwaysOnBottom')
  }
  setContentProtected(protected_) {
    check(api.webviewWindowSetContentProtected(this.#handle, protected_), 'window.setContentProtected')
  }
  setSkipTaskbar(skip) {
    check(api.webviewWindowSetSkipTaskbar(this.#handle, skip), 'window.setSkipTaskbar')
  }
  setShadow(enable) {
    check(api.webviewWindowSetShadow(this.#handle, enable), 'window.setShadow')
  }
  setVisibleOnAllWorkspaces(visible) {
    check(api.webviewWindowSetVisibleOnAllWorkspaces(this.#handle, visible), 'window.setVisibleOnAllWorkspaces')
  }
  setIgnoreCursorEvents(ignore) {
    check(api.webviewWindowSetIgnoreCursorEvents(this.#handle, ignore), 'window.setIgnoreCursorEvents')
  }
  setCursorVisible(visible) {
    check(api.webviewWindowSetCursorVisible(this.#handle, visible), 'window.setCursorVisible')
  }
  setCursorGrab(grab) {
    check(api.webviewWindowSetCursorGrab(this.#handle, grab), 'window.setCursorGrab')
  }
  setEnabled(enabled) {
    check(api.webviewWindowSetEnabled(this.#handle, enabled), 'window.setEnabled')
  }
  setFocusable(focusable) {
    check(api.webviewWindowSetFocusable(this.#handle, focusable), 'window.setFocusable')
  }
  setSimpleFullscreen(enable) {
    check(api.webviewWindowSetSimpleFullscreen(this.#handle, enable), 'window.setSimpleFullscreen')
  }
  /** Sizes are logical (DPI-scaled) unless `physical: true`; omit or pass a
   * non-positive size to clear the constraint. */
  setMinSize({ width = 0, height = 0, physical = false } = {}) {
    check(api.webviewWindowSetMinSize(this.#handle, width, height, physical), 'window.setMinSize')
  }
  setMaxSize({ width = 0, height = 0, physical = false } = {}) {
    check(api.webviewWindowSetMaxSize(this.#handle, width, height, physical), 'window.setMaxSize')
  }
  setCursorPosition({ x, y, physical = false }) {
    check(api.webviewWindowSetCursorPosition(this.#handle, x, y, physical), 'window.setCursorPosition')
  }
  /** Pass 'light', 'dark', or null/'' to follow the system theme. */
  setTheme(theme) {
    check(api.webviewWindowSetTheme(this.#handle, theme ?? ''), 'window.setTheme')
  }
  /** e.g. 'default', 'pointer', 'crosshair', 'grab', 'wait'. */
  setCursorIcon(icon) {
    check(api.webviewWindowSetCursorIcon(this.#handle, icon), 'window.setCursorIcon')
  }
  /** Pass 'critical', 'informational', or null/'' to cancel. */
  requestUserAttention(kind) {
    check(api.webviewWindowRequestUserAttention(this.#handle, kind ?? ''), 'window.requestUserAttention')
  }
  /** { status, progress } — see Tauri's ProgressBarState. */
  setProgressBar(state) {
    check(api.webviewWindowSetProgressBar(this.#handle, JSON.stringify(state ?? {})), 'window.setProgressBar')
  }
  /** A WindowEffectsConfig object, or null to clear effects. */
  setEffects(effects) {
    check(api.webviewWindowSetEffects(this.#handle, JSON.stringify(effects ?? null)), 'window.setEffects')
  }
  /** A WindowSizeConstraints object, e.g. { minWidth: { Logical: 400 } }. */
  setSizeConstraints(constraints) {
    check(api.webviewWindowSetSizeConstraints(this.#handle, JSON.stringify(constraints ?? {})), 'window.setSizeConstraints')
  }
  /** RGBA channels 0-255. */
  setBackgroundColor({ r, g, b, a = 255 }) {
    check(api.webviewWindowSetBackgroundColor(this.#handle, r, g, b, a), 'window.setBackgroundColor')
  }
  /** Pass a negative count to clear the badge. */
  setBadgeCount(count) {
    check(api.webviewWindowSetBadgeCount(this.#handle, count ?? -1), 'window.setBadgeCount')
  }
  /** macOS only: the dock badge label; null/'' clears it. */
  setBadgeLabel(label) {
    check(api.webviewWindowSetBadgeLabel(this.#handle, label ?? ''), 'window.setBadgeLabel')
  }
  /** macOS only: 'visible', 'transparent' or 'overlay'. */
  setTitleBarStyle(style) {
    check(api.webviewWindowSetTitleBarStyle(this.#handle, style), 'window.setTitleBarStyle')
  }
  /** Windows only: RGBA pixels (width*height*4 bytes), or null to clear. */
  setOverlayIcon(rgba, width = 0, height = 0) {
    check(api.webviewWindowSetOverlayIcon(this.#handle, rgba, width, height), 'window.setOverlayIcon')
  }
  /** macOS only: the NSWindow pointer, as an integer. */
  nsWindow() {
    const out = [0]
    check(api.webviewWindowNsWindow(this.#handle, out), 'window.nsWindow')
    return out[0]
  }
  /** macOS only: the NSView pointer, as an integer. */
  nsView() {
    const out = [0]
    check(api.webviewWindowNsView(this.#handle, out), 'window.nsView')
    return out[0]
  }
  /** Windows only: the window's HWND, as an integer. */
  hwnd() {
    const out = [0]
    check(api.webviewWindowHwnd(this.#handle, out), 'window.hwnd')
    return out[0]
  }
  show() {
    check(api.webviewWindowShow(this.#handle), 'window.show')
  }
  hide() {
    check(api.webviewWindowHide(this.#handle), 'window.hide')
  }
  center() {
    check(api.webviewWindowCenter(this.#handle), 'window.center')
  }
  maximize() {
    check(api.webviewWindowMaximize(this.#handle), 'window.maximize')
  }
  unmaximize() {
    check(api.webviewWindowUnmaximize(this.#handle), 'window.unmaximize')
  }
  minimize() {
    check(api.webviewWindowMinimize(this.#handle), 'window.minimize')
  }
  unminimize() {
    check(api.webviewWindowUnminimize(this.#handle), 'window.unminimize')
  }
  close() {
    check(api.webviewWindowClose(this.#handle), 'window.close')
  }
  destroy() {
    check(api.webviewWindowDestroy(this.#handle), 'window.destroy')
  }

  eval(js) {
    check(api.webviewWindowEval(this.#handle, js), 'window.eval')
  }
  navigate(url) {
    check(api.webviewWindowNavigate(this.#handle, String(url)), 'window.navigate')
  }
  reload() {
    check(api.webviewWindowReload(this.#handle), 'window.reload')
  }
  startDragging() {
    check(api.webviewWindowStartDragging(this.#handle), 'window.startDragging')
  }
  print() {
    check(api.webviewWindowPrint(this.#handle), 'window.print')
  }
  clearAllBrowsingData() {
    check(api.webviewWindowClearAllBrowsingData(this.#handle), 'window.clearAllBrowsingData')
  }
  hideMenu() {
    check(api.webviewWindowHideMenu(this.#handle), 'window.hideMenu')
  }
  showMenu() {
    check(api.webviewWindowShowMenu(this.#handle), 'window.showMenu')
  }
  /** Requires a debug build or the tauri-ffi `devtools` feature. */
  openDevtools() {
    check(api.webviewWindowOpenDevtools(this.#handle), 'window.openDevtools')
  }
  closeDevtools() {
    check(api.webviewWindowCloseDevtools(this.#handle), 'window.closeDevtools')
  }

  /** Releases the handle; the window itself is unaffected. */
  free() {
    check(api.handleClose(this.#handle), 'window.free')
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
  #handle
  constructor(handle) {
    this.#handle = handle
  }
  id() {
    const out = [null]
    check(api.trayId(this.#handle, out), 'tray.id')
    return takeString(out)
  }
  /** Icon from a PNG/ICO file path; pass null/'' to clear. */
  setIcon(path) {
    check(api.traySetIcon(this.#handle, path ?? ''), 'tray.setIcon')
  }
  setIconAsTemplate(isTemplate) {
    check(api.traySetIconAsTemplate(this.#handle, isTemplate), 'tray.setIconAsTemplate')
  }
  setTooltip(tooltip) {
    check(api.traySetTooltip(this.#handle, tooltip ?? ''), 'tray.setTooltip')
  }
  setTitle(title) {
    check(api.traySetTitle(this.#handle, title ?? ''), 'tray.setTitle')
  }
  setVisible(visible) {
    check(api.traySetVisible(this.#handle, visible), 'tray.setVisible')
  }
  setShowMenuOnLeftClick(enable) {
    check(api.traySetShowMenuOnLeftClick(this.#handle, enable), 'tray.setShowMenuOnLeftClick')
  }
  /** Sets (or clears, when menu is null) the tray context menu. */
  setMenu(menu) {
    check(api.traySetMenu(this.#handle, menu ? menu.handle : 0), 'tray.setMenu')
  }
  /** Releases the handle; dropping the last handle removes the icon. */
  free() {
    check(api.handleClose(this.#handle), 'tray.free')
  }
}

/**
 * Mirrors `tauri::Window` — a bare OS window with no webview, that can host one
 * or more {@link Webview}s (multiwebview). For the common single-webview case
 * use {@link WebviewWindow}. Obtain via `app.createBareWindow()`.
 */
export class Window {
  #handle
  constructor(handle) {
    this.#handle = handle
  }
  get handle() {
    return this.#handle
  }
  #str(fn, what) {
    const out = [null]
    check(fn(this.#handle, out), what)
    return takeString(out)
  }
  #bool(fn, what) {
    const out = [false]
    check(fn(this.#handle, out), what)
    return out[0]
  }
  #act(fn, what) {
    check(fn(this.#handle), what)
  }
  #set(fn, what, value) {
    check(fn(this.#handle, value), what)
  }

  /** Adds a child webview at { x, y, width, height } (logical unless physical). */
  addWebview(config, { x = 0, y = 0, width, height, physical = false }) {
    const out = [0]
    check(api.windowAddWebview(this.#handle, JSON.stringify(config), x, y, width, height, physical, out), 'window.addWebview')
    return new Webview(out[0])
  }
  /** Labels of this window's child webviews. */
  webviews() {
    return JSON.parse(this.#str(api.windowWebviews, 'window.webviews'))
  }

  label() { return this.#str(api.windowLabel, 'window.label') }
  title() { return this.#str(api.windowTitle, 'window.title') }
  theme() { return this.#str(api.windowTheme, 'window.theme') }
  scaleFactor() { const o = [0]; check(api.windowScaleFactor(this.#handle, o), 'window.scaleFactor'); return o[0] }
  innerSize() { const w = [0], h = [0]; check(api.windowInnerSize(this.#handle, w, h), 'window.innerSize'); return { width: w[0], height: h[0] } }
  outerSize() { const w = [0], h = [0]; check(api.windowOuterSize(this.#handle, w, h), 'window.outerSize'); return { width: w[0], height: h[0] } }
  innerPosition() { const x = [0], y = [0]; check(api.windowInnerPosition(this.#handle, x, y), 'window.innerPosition'); return { x: x[0], y: y[0] } }
  outerPosition() { const x = [0], y = [0]; check(api.windowOuterPosition(this.#handle, x, y), 'window.outerPosition'); return { x: x[0], y: y[0] } }
  cursorPosition() { const x = [0], y = [0]; check(api.windowCursorPosition(this.#handle, x, y), 'window.cursorPosition'); return { x: x[0], y: y[0] } }

  isVisible() { return this.#bool(api.windowIsVisible, 'window.isVisible') }
  isFocused() { return this.#bool(api.windowIsFocused, 'window.isFocused') }
  isFullscreen() { return this.#bool(api.windowIsFullscreen, 'window.isFullscreen') }
  isMaximized() { return this.#bool(api.windowIsMaximized, 'window.isMaximized') }
  isMinimized() { return this.#bool(api.windowIsMinimized, 'window.isMinimized') }
  isResizable() { return this.#bool(api.windowIsResizable, 'window.isResizable') }
  isDecorated() { return this.#bool(api.windowIsDecorated, 'window.isDecorated') }
  isClosable() { return this.#bool(api.windowIsClosable, 'window.isClosable') }
  isMaximizable() { return this.#bool(api.windowIsMaximizable, 'window.isMaximizable') }
  isMinimizable() { return this.#bool(api.windowIsMinimizable, 'window.isMinimizable') }
  isAlwaysOnTop() { return this.#bool(api.windowIsAlwaysOnTop, 'window.isAlwaysOnTop') }
  isEnabled() { return this.#bool(api.windowIsEnabled, 'window.isEnabled') }
  isMenuVisible() { return this.#bool(api.windowIsMenuVisible, 'window.isMenuVisible') }

  availableMonitors() { return JSON.parse(this.#str(api.windowAvailableMonitors, 'window.availableMonitors') || '[]') }
  currentMonitor() { return JSON.parse(this.#str(api.windowCurrentMonitor, 'window.currentMonitor') || 'null') }
  primaryMonitor() { return JSON.parse(this.#str(api.windowPrimaryMonitor, 'window.primaryMonitor') || 'null') }
  monitorFromPoint(x, y) { const o = [null]; check(api.windowMonitorFromPoint(this.#handle, x, y, o), 'window.monitorFromPoint'); return JSON.parse(takeString(o) || 'null') }

  setTitle(title) { this.#set(api.windowSetTitle, 'window.setTitle', title) }
  setSize({ width, height, physical = false }) { check(api.windowSetSize(this.#handle, width, height, physical), 'window.setSize') }
  setPosition({ x, y, physical = false }) { check(api.windowSetPosition(this.#handle, x, y, physical), 'window.setPosition') }
  setMinSize({ width = 0, height = 0, physical = false } = {}) { check(api.windowSetMinSize(this.#handle, width, height, physical), 'window.setMinSize') }
  setMaxSize({ width = 0, height = 0, physical = false } = {}) { check(api.windowSetMaxSize(this.#handle, width, height, physical), 'window.setMaxSize') }
  setCursorPosition({ x, y, physical = false }) { check(api.windowSetCursorPosition(this.#handle, x, y, physical), 'window.setCursorPosition') }
  setFullscreen(v) { this.#set(api.windowSetFullscreen, 'window.setFullscreen', v) }
  setResizable(v) { this.#set(api.windowSetResizable, 'window.setResizable', v) }
  setAlwaysOnTop(v) { this.#set(api.windowSetAlwaysOnTop, 'window.setAlwaysOnTop', v) }
  setAlwaysOnBottom(v) { this.#set(api.windowSetAlwaysOnBottom, 'window.setAlwaysOnBottom', v) }
  setDecorations(v) { this.#set(api.windowSetDecorations, 'window.setDecorations', v) }
  setClosable(v) { this.#set(api.windowSetClosable, 'window.setClosable', v) }
  setMaximizable(v) { this.#set(api.windowSetMaximizable, 'window.setMaximizable', v) }
  setMinimizable(v) { this.#set(api.windowSetMinimizable, 'window.setMinimizable', v) }
  setContentProtected(v) { this.#set(api.windowSetContentProtected, 'window.setContentProtected', v) }
  setSkipTaskbar(v) { this.#set(api.windowSetSkipTaskbar, 'window.setSkipTaskbar', v) }
  setShadow(v) { this.#set(api.windowSetShadow, 'window.setShadow', v) }
  setVisibleOnAllWorkspaces(v) { this.#set(api.windowSetVisibleOnAllWorkspaces, 'window.setVisibleOnAllWorkspaces', v) }
  setIgnoreCursorEvents(v) { this.#set(api.windowSetIgnoreCursorEvents, 'window.setIgnoreCursorEvents', v) }
  setCursorVisible(v) { this.#set(api.windowSetCursorVisible, 'window.setCursorVisible', v) }
  setCursorGrab(v) { this.#set(api.windowSetCursorGrab, 'window.setCursorGrab', v) }
  setEnabled(v) { this.#set(api.windowSetEnabled, 'window.setEnabled', v) }
  setFocusable(v) { this.#set(api.windowSetFocusable, 'window.setFocusable', v) }
  setSimpleFullscreen(v) { this.#set(api.windowSetSimpleFullscreen, 'window.setSimpleFullscreen', v) }
  setTheme(theme) { this.#set(api.windowSetTheme, 'window.setTheme', theme ?? '') }
  setCursorIcon(icon) { this.#set(api.windowSetCursorIcon, 'window.setCursorIcon', icon) }
  requestUserAttention(kind) { this.#set(api.windowRequestUserAttention, 'window.requestUserAttention', kind ?? '') }
  setProgressBar(state) { this.#set(api.windowSetProgressBar, 'window.setProgressBar', JSON.stringify(state ?? {})) }
  setEffects(effects) { this.#set(api.windowSetEffects, 'window.setEffects', JSON.stringify(effects ?? null)) }
  setSizeConstraints(c) { this.#set(api.windowSetSizeConstraints, 'window.setSizeConstraints', JSON.stringify(c ?? {})) }
  setBackgroundColor({ r, g, b, a = 255 }) { check(api.windowSetBackgroundColor(this.#handle, r, g, b, a), 'window.setBackgroundColor') }
  setBadgeCount(count) { this.#set(api.windowSetBadgeCount, 'window.setBadgeCount', count ?? -1) }
  setBadgeLabel(label) { this.#set(api.windowSetBadgeLabel, 'window.setBadgeLabel', label ?? '') }
  /** macOS only: 'visible', 'transparent' or 'overlay'. */
  setTitleBarStyle(style) { this.#set(api.windowSetTitleBarStyle, 'window.setTitleBarStyle', style) }
  /** Windows only: RGBA pixels (width*height*4 bytes), or null to clear. */
  setOverlayIcon(rgba, width = 0, height = 0) { check(api.windowSetOverlayIcon(this.#handle, rgba, width, height), 'window.setOverlayIcon') }
  /** macOS only: the NSWindow pointer, as an integer. */
  nsWindow() { const o = [0]; check(api.windowNsWindow(this.#handle, o), 'window.nsWindow'); return o[0] }
  /** macOS only: the NSView pointer, as an integer. */
  nsView() { const o = [0]; check(api.windowNsView(this.#handle, o), 'window.nsView'); return o[0] }
  /** Windows only: the window's HWND, as an integer. */
  hwnd() { const o = [0]; check(api.windowHwnd(this.#handle, o), 'window.hwnd'); return o[0] }
  setFocus() { this.#act(api.windowSetFocus, 'window.setFocus') }
  show() { this.#act(api.windowShow, 'window.show') }
  hide() { this.#act(api.windowHide, 'window.hide') }
  center() { this.#act(api.windowCenter, 'window.center') }
  maximize() { this.#act(api.windowMaximize, 'window.maximize') }
  unmaximize() { this.#act(api.windowUnmaximize, 'window.unmaximize') }
  minimize() { this.#act(api.windowMinimize, 'window.minimize') }
  unminimize() { this.#act(api.windowUnminimize, 'window.unminimize') }
  close() { this.#act(api.windowClose, 'window.close') }
  destroy() { this.#act(api.windowDestroy, 'window.destroy') }
  startDragging() { this.#act(api.windowStartDragging, 'window.startDragging') }
  hideMenu() { this.#act(api.windowHideMenu, 'window.hideMenu') }
  showMenu() { this.#act(api.windowShowMenu, 'window.showMenu') }
  /** Attaches a Menu as this window's menu. */
  setMenu(menu) { check(api.menuSetAsWindowMenu(this.#handle, menu.handle), 'window.setMenu') }
  free() { check(api.handleClose(this.#handle), 'window.free') }
}

/** Mirrors `tauri::Webview` — a webview hosted inside a {@link Window}. */
export class Webview {
  #handle
  constructor(handle) {
    this.#handle = handle
  }
  get handle() {
    return this.#handle
  }
  label() { const o = [null]; check(api.webviewLabel(this.#handle, o), 'webview.label'); return takeString(o) }
  url() { const o = [null]; check(api.webviewUrl(this.#handle, o), 'webview.url'); return takeString(o) }
  position() { const x = [0], y = [0]; check(api.webviewPosition(this.#handle, x, y), 'webview.position'); return { x: x[0], y: y[0] } }
  size() { const w = [0], h = [0]; check(api.webviewSize(this.#handle, w, h), 'webview.size'); return { width: w[0], height: h[0] } }
  /** The parent window. */
  window() { const o = [0]; check(api.webviewGetWindow(this.#handle, o), 'webview.window'); return new Window(o[0]) }
  eval(js) { check(api.webviewEval(this.#handle, js), 'webview.eval') }
  navigate(url) { check(api.webviewNavigate(this.#handle, String(url)), 'webview.navigate') }
  reload() { check(api.webviewReload(this.#handle), 'webview.reload') }
  print() { check(api.webviewPrint(this.#handle), 'webview.print') }
  setZoom(scale) { check(api.webviewSetZoom(this.#handle, scale), 'webview.setZoom') }
  setAutoResize(v) { check(api.webviewSetAutoResize(this.#handle, v), 'webview.setAutoResize') }
  setSize({ width, height, physical = false }) { check(api.webviewSetSize(this.#handle, width, height, physical), 'webview.setSize') }
  setPosition({ x, y, physical = false }) { check(api.webviewSetPosition(this.#handle, x, y, physical), 'webview.setPosition') }
  setBackgroundColor({ r, g, b, a = 255 }) { check(api.webviewSetBackgroundColor(this.#handle, r, g, b, a), 'webview.setBackgroundColor') }
  setFocus() { check(api.webviewSetFocus(this.#handle), 'webview.setFocus') }
  show() { check(api.webviewShow(this.#handle), 'webview.show') }
  hide() { check(api.webviewHide(this.#handle), 'webview.hide') }
  close() { check(api.webviewClose(this.#handle), 'webview.close') }
  clearAllBrowsingData() { check(api.webviewClearAllBrowsingData(this.#handle), 'webview.clearAllBrowsingData') }
  reparent(window) { check(api.webviewReparent(this.#handle, window.handle), 'webview.reparent') }
  openDevtools() { check(api.webviewOpenDevtools(this.#handle), 'webview.openDevtools') }
  closeDevtools() { check(api.webviewCloseDevtools(this.#handle), 'webview.closeDevtools') }
  isDevtoolsOpen() { const o = [false]; check(api.webviewIsDevtoolsOpen(this.#handle, o), 'webview.isDevtoolsOpen'); return o[0] }
  free() { check(api.handleClose(this.#handle), 'webview.free') }
}

/** A menu item (`tauri::menu::MenuItemKind`). Menu clicks arrive via
 * `app.on('menu-event', ({ id }) => …)`. */
export class MenuItem {
  #handle
  constructor(handle) {
    this.#handle = handle
  }
  get handle() {
    return this.#handle
  }
  id() { const o = [null]; check(api.menuItemId(this.#handle, o), 'menuItem.id'); return takeString(o) }
  setText(text) { check(api.menuItemSetText(this.#handle, text), 'menuItem.setText') }
  setEnabled(v) { check(api.menuItemSetEnabled(this.#handle, v), 'menuItem.setEnabled') }
  setChecked(v) { check(api.menuItemSetChecked(this.#handle, v), 'menuItem.setChecked') }
  setAccelerator(accel) { check(api.menuItemSetAccelerator(this.#handle, accel ?? ''), 'menuItem.setAccelerator') }
  /** Appends a child (only valid on a submenu). */
  append(item) { check(api.submenuAppend(this.#handle, item.handle), 'submenu.append') }
  free() { check(api.handleClose(this.#handle), 'menuItem.free') }
}

/** A menu (`tauri::menu::Menu`). Build it, then attach with
 * `app.setAppMenu(menu)`, `window.setMenu(menu)` or `tray.setMenu(menu)`. */
export class Menu {
  #handle
  constructor(handle) {
    this.#handle = handle
  }
  get handle() {
    return this.#handle
  }
  append(item) { check(api.menuAppend(this.#handle, item.handle), 'menu.append') }
  free() { check(api.handleClose(this.#handle), 'menu.free') }
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
    check(api.webviewWindowCreate(appId, JSON.stringify(config), out), `createWindow(${config?.label})`)
    return new WebviewWindow(out[0])
  },

  /** The window with the given label, or null. */
  getWindow(label) {
    const out = [0]
    const code = api.appGetWebviewWindow(appId, label, out)
    if (code === CODES.NOT_FOUND) return null
    check(code, `getWindow(${label})`)
    return new WebviewWindow(out[0])
  },

  /** Labels of all webview windows. */
  windowLabels() {
    const out = [null]
    check(api.appWebviewWindowLabels(appId, out), 'windowLabels')
    return JSON.parse(takeString(out))
  },

  /** The app's resolved configuration (tauri.conf.json shape). */
  config() {
    const out = [null]
    check(api.appConfig(appId, out), 'config')
    return JSON.parse(takeString(out))
  },

  /** The app package info: { name, version, authors, description, crateName }. */
  packageInfo() {
    const out = [null]
    check(api.appPackageInfo(appId, out), 'packageInfo')
    return JSON.parse(takeString(out))
  },

  /**
   * Platform base directories (`Manager::path`). One method per kind, e.g.
   * `app.path.appDataDir()`, `app.path.homeDir()`, `app.path.tempDir()`.
   */
  path: Object.fromEntries(
    PATH_KINDS.map((kind) => [
      `${kind}Dir`,
      () => {
        const out = [null]
        check(api.appPath(appId, kind, out), `path.${kind}Dir`)
        return takeString(out)
      }
    ])
  ),

  /** The focused webview window, or null. */
  getFocusedWindow() {
    const out = [0]
    const code = api.appGetFocusedWindow(appId, out)
    if (code === CODES.NOT_FOUND) return null
    check(code, 'getFocusedWindow')
    return new WebviewWindow(out[0])
  },

  /** Adds a capability (JSON/TOML string) to the running app. */
  addCapability(capability) {
    const json = typeof capability === 'string' ? capability : JSON.stringify(capability)
    check(api.appAddCapability(appId, json), 'addCapability')
  },

  /** Like listen(), but auto-removed after the first delivery. */
  once(event, handler) {
    const out = [0]
    check(api.appOnce(appId, event, out), `once(${event})`)
    const id = out[0]
    eventListeners.set(id, (payload, message) => {
      eventListeners.delete(id)
      handler(payload, message)
    })
    return id
  },

  /** Sets the app-wide theme: 'light', 'dark', or null to follow the system. */
  setTheme(theme) {
    check(api.appSetTheme(appId, theme ?? ''), 'setTheme')
  },

  /** The current cursor position, in physical screen pixels. */
  cursorPosition() {
    const x = [0]
    const y = [0]
    check(api.appCursorPosition(appId, x, y), 'cursorPosition')
    return { x: x[0], y: y[0] }
  },

  /** Requests an app restart (exits and relaunches). */
  requestRestart() {
    check(api.appRequestRestart(appId), 'requestRestart')
  },

  /** macOS only: 'regular', 'accessory' (no Dock icon) or 'prohibited'. */
  setActivationPolicy(policy) {
    check(api.appSetActivationPolicy(appId, policy), 'setActivationPolicy')
  },

  /** macOS only: shows or hides the app's Dock icon. */
  setDockVisibility(visible) {
    check(api.appSetDockVisibility(appId, visible), 'setDockVisibility')
  },

  /** macOS only: shows the application without focusing it. */
  show() {
    check(api.appShow(appId), 'show')
  },

  /** macOS only: hides the application, like Cmd+H. */
  hide() {
    check(api.appHide(appId), 'hide')
  },

  /** Creates a system tray icon. Listen via `app.on('tray-event', handler)`. */
  createTray(id = '') {
    const out = [0]
    check(api.trayNew(appId, id, out), `createTray(${id})`)
    return new Tray(out[0])
  },

  removeTrayById(id) {
    check(api.appRemoveTrayById(appId, id), 'removeTrayById')
  },

  /** Creates a bare OS window (no webview) that can host {@link Webview}s. */
  createBareWindow(config) {
    const out = [0]
    check(api.windowCreate(appId, JSON.stringify(config), out), `createBareWindow(${config?.label})`)
    return new Window(out[0])
  },

  /** The bare window with the given label, or null. */
  getBareWindow(label) {
    const out = [0]
    const code = api.appGetWindow(appId, label, out)
    if (code === CODES.NOT_FOUND) return null
    check(code, `getBareWindow(${label})`)
    return new Window(out[0])
  },

  /** Labels of all bare windows. */
  bareWindowLabels() {
    const out = [null]
    check(api.appWindowLabels(appId, out), 'bareWindowLabels')
    return JSON.parse(takeString(out))
  },

  /** Creates an empty {@link Menu}. */
  createMenu() {
    const out = [0]
    check(api.menuNew(appId, out), 'createMenu')
    return new Menu(out[0])
  },

  /** A normal menu item. */
  menuItem({ id = '', text, enabled = true, accelerator = '' }) {
    const out = [0]
    check(api.menuItemNew(appId, id, text, enabled, accelerator, out), 'menuItem')
    return new MenuItem(out[0])
  },

  /** A checkable menu item. */
  checkMenuItem({ id = '', text, enabled = true, checked = false, accelerator = '' }) {
    const out = [0]
    check(api.menuCheckItemNew(appId, id, text, enabled, checked, accelerator, out), 'checkMenuItem')
    return new MenuItem(out[0])
  },

  /** A predefined menu item ('separator', 'copy', 'quit', …). */
  predefinedMenuItem(kind, text = '') {
    const out = [0]
    check(api.menuPredefinedItemNew(appId, kind, text, out), 'predefinedMenuItem')
    return new MenuItem(out[0])
  },

  /** A submenu (append children with `.append()`). */
  submenu({ id = '', text, enabled = true }) {
    const out = [0]
    check(api.submenuNew(appId, id, text, enabled, out), 'submenu')
    return new MenuItem(out[0])
  },

  /** Sets a menu as the app-wide menu (macOS menu bar). */
  setAppMenu(menu) {
    check(api.menuSetAsAppMenu(menu.handle), 'setAppMenu')
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
