// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides APIs to create windows, communicate with other windows and manipulate the current window.
 *
 * #### Window events
 *
 * Events can be listened to using {@link Window.listen}:
 * ```typescript
 * import { getCurrentWindow } from "@tauri-apps/api/window";
 * getCurrentWindow().listen("my-window-event", ({ event, payload }) => { });
 * ```
 *
 * This package is also accessible with `window.__TAURI__.window` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * @remarks `core:window:default` only enables the getters (sizes, positions,
 * monitors, title, theme and the `is*` queries). Creating a window and every
 * method that changes one needs its own permission, which each member documents,
 * added to a capability in your app.
 *
 * @module
 */

import {
  LogicalPosition,
  LogicalSize,
  PhysicalPosition,
  PhysicalSize,
  Position,
  Size
} from './dpi'
import type { Event, EventName, EventCallback, UnlistenFn } from './event'
import {
  TauriEvent,
  // imported for documentation purposes
  type EventTarget,
  emit,
  emitTo,
  listen,
  once
} from './event'
import { invoke } from './core'
import { WebviewWindow } from './webviewWindow'
import type { DragDropEvent } from './webview'
import { type JsImage, transformImage } from './image'

/**
 * Allows you to retrieve information about a given monitor.
 *
 * @since 1.0.0
 */
export interface Monitor {
  /** Human-readable name of the monitor */
  name: string | null
  /**
   * The monitor's resolution in physical pixels.
   *
   * Use {@linkcode Monitor.scaleFactor} to convert to logical pixels:
   * ```typescript
   * const logicalSize = monitor.size.toLogical(monitor.scaleFactor);
   * ```
   */
  size: PhysicalSize
  /**
   * the Top-left corner position of the monitor relative to the larger full screen area, in physical pixels.
   *
   * Note that window creation options such as `x`, `y`, `width` and `height` expect
   * logical pixels, so convert with {@linkcode Monitor.scaleFactor} first:
   * ```typescript
   * import { currentMonitor } from '@tauri-apps/api/window';
   * import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
   *
   * const monitor = await currentMonitor();
   * if (monitor) {
   *   const position = monitor.position.toLogical(monitor.scaleFactor);
   *   const webview = new WebviewWindow('my-label', { x: position.x, y: position.y });
   * }
   * ```
   */
  position: PhysicalPosition
  /**
   * The monitor's work area (the monitor area excluding taskbars and docks) in physical pixels.
   *
   * Use {@linkcode Monitor.scaleFactor} to convert to logical pixels as shown in
   * {@linkcode Monitor.position}.
   */
  workArea: {
    position: PhysicalPosition
    size: PhysicalSize
  }
  /**
   * The scale factor that can be used to map physical pixels to logical pixels,
   * e.g. `monitor.position.toLogical(monitor.scaleFactor)`.
   */
  scaleFactor: number
}

/** The light or dark appearance of a window or of the system. */
type Theme = 'light' | 'dark'

/**
 * The style of the window title bar. **macOS only**
 *
 * - `visible`: the standard title bar.
 * - `transparent`: the title bar is transparent but the traffic lights stay in place.
 * - `overlay`: the content extends under a transparent title bar, leaving only the
 *   traffic lights visible.
 */
type TitleBarStyle = 'visible' | 'transparent' | 'overlay'

/** The edge or corner a resize drag started from, see {@link Window.startResizeDragging}. */
type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West'

/**
 * The payload for the `scaleChange` event.
 *
 * @since 1.0.2
 */
interface ScaleFactorChanged {
  /** The new window scale factor. */
  scaleFactor: number
  /** The new window size */
  size: PhysicalSize
}

/**
 * Attention type to request on a window.
 *
 * @since 1.0.0
 */
enum UserAttentionType {
  /**
   * #### Platform-specific
   * - **macOS:** Bounces the dock icon until the application is in focus.
   * - **Windows:** Flashes both the window and the taskbar button until the application is in focus.
   */
  Critical = 1,
  /**
   * #### Platform-specific
   * - **macOS:** Bounces the dock icon once.
   * - **Windows:** Flashes the taskbar button until the application is in focus.
   */
  Informational
}

/**
 * The event passed to a {@link Window.onCloseRequested} handler.
 *
 * Call {@link CloseRequestedEvent.preventDefault} to keep the window open, for
 * example to ask the user to save their work first.
 */
class CloseRequestedEvent {
  /** Event name */
  event: EventName
  /** Event identifier used to unlisten */
  id: number
  private _preventDefault = false

  constructor(event: Event<unknown>) {
    this.event = event.event
    this.id = event.id
  }

  /**
   * Prevents the window from being closed.
   *
   * Must be called before the handler returns (await anything you need first),
   * otherwise the window is destroyed as usual.
   */
  preventDefault(): void {
    this._preventDefault = true
  }

  /** Whether {@link CloseRequestedEvent.preventDefault} was called on this event. */
  isPreventDefault(): boolean {
    return this._preventDefault
  }
}

/**
 * The cursor shape to display over a window, see {@link Window.setCursorIcon}.
 *
 * The names mirror the CSS `cursor` keywords in camelCase (e.g. `notAllowed` for
 * `not-allowed`, `eResize` for `e-resize`). Support varies per platform and window
 * manager; unsupported values fall back to the default arrow.
 */
export type CursorIcon =
  | 'default'
  | 'crosshair'
  | 'hand'
  | 'arrow'
  | 'move'
  | 'text'
  | 'wait'
  | 'help'
  | 'progress'
  // something cannot be done
  | 'notAllowed'
  | 'contextMenu'
  | 'cell'
  | 'verticalText'
  | 'alias'
  | 'copy'
  | 'noDrop'
  // something can be grabbed
  | 'grab'
  /// something is grabbed
  | 'grabbing'
  | 'allScroll'
  | 'zoomIn'
  | 'zoomOut'
  // edge is to be moved
  | 'eResize'
  | 'nResize'
  | 'neResize'
  | 'nwResize'
  | 'sResize'
  | 'seResize'
  | 'swResize'
  | 'wResize'
  | 'ewResize'
  | 'nsResize'
  | 'neswResize'
  | 'nwseResize'
  | 'colResize'
  | 'rowResize'

/**
 * The state of the taskbar/dock progress bar, see {@link Window.setProgressBar}.
 *
 * @since 2.0.0
 */
export enum ProgressBarStatus {
  /**
   * Hide progress bar.
   */
  None = 'none',
  /**
   * Normal state.
   */
  Normal = 'normal',
  /**
   * Indeterminate state. **Treated as Normal on Linux and macOS**
   */
  Indeterminate = 'indeterminate',
  /**
   * Paused state. **Treated as Normal on Linux**
   */
  Paused = 'paused',
  /**
   * Error state. **Treated as Normal on linux**
   */
  Error = 'error'
}

/**
 * Inner size constraints for a window, see {@link Window.setSizeConstraints}.
 *
 * Unlike {@link Window.setMinSize} and {@link Window.setMaxSize}, each dimension is
 * independent: you can constrain only the width and leave the height free.
 *
 * All values are in **logical pixels**. Leave a field out (or set it to
 * `undefined`) to remove that constraint.
 *
 * @example
 * ```typescript
 * import { getCurrentWindow } from '@tauri-apps/api/window';
 * // at least 300 logical pixels wide, height unconstrained
 * await getCurrentWindow().setSizeConstraints({ minWidth: 300 });
 * ```
 *
 * @since 2.0.0
 */
export interface WindowSizeConstraints {
  /** The minimum width, in logical pixels. */
  minWidth?: number
  /** The minimum height, in logical pixels. */
  minHeight?: number
  /** The maximum width, in logical pixels. */
  maxWidth?: number
  /** The maximum height, in logical pixels. */
  maxHeight?: number
}

/**
 * The taskbar/dock progress indicator state, see {@link Window.setProgressBar}.
 *
 * @since 2.0.0
 */
export interface ProgressBarState {
  /**
   * The progress bar status.
   */
  status?: ProgressBarStatus
  /**
   * The progress bar progress. This can be a value ranging from `0` to `100`
   */
  progress?: number
}

/**
 * Get an instance of `Window` for the current window.
 *
 * @since 1.0.0
 */
function getCurrentWindow(): Window {
  return new Window(window.__TAURI_INTERNALS__.metadata.currentWindow.label, {
    // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
    skip: true
  })
}

/**
 * Gets a list of instances of `Window` for all available windows.
 *
 * @remarks Uses the `core:window:allow-get-all-windows` permission, which is part
 * of `core:window:default`.
 *
 * @since 1.0.0
 */
async function getAllWindows(): Promise<Window[]> {
  return invoke<string[]>('plugin:window|get_all_windows').then((windows) =>
    windows.map(
      (w) =>
        new Window(w, {
          // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
          skip: true
        })
    )
  )
}

/** @ignore */
// events that are emitted right here instead of by the created window
const localTauriEvents = ['tauri://created', 'tauri://error']
/** @ignore */
export type WindowLabel = string

/**
 * Create new window or get a handle to an existing one.
 *
 * Windows are identified by a *label*  a unique identifier that can be used to reference it later.
 * It may only contain alphanumeric characters `a-zA-Z` plus the following special characters `-`, `/`, `:` and `_`.
 *
 * @example
 * ```typescript
 * import { Window } from "@tauri-apps/api/window"
 *
 * const appWindow = new Window('theUniqueLabel');
 *
 * appWindow.once('tauri://created', function () {
 *  // window successfully created
 * });
 * appWindow.once('tauri://error', function (e) {
 *  // an error happened creating the window
 * });
 *
 * // emit an event to the backend
 * await appWindow.emit("some-event", "data");
 * // listen to an event from the backend
 * const unlisten = await appWindow.listen("event-name", e => {});
 * unlisten();
 * ```
 *
 * @since 2.0.0
 */
class Window {
  /** The window label. It is a unique identifier for the window, can be used to reference it later. */
  label: WindowLabel
  /** Local event listeners. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  listeners: Record<string, Array<EventCallback<any>>>

  /**
   * Creates a new Window.
   * @example
   * ```typescript
   * import { Window } from '@tauri-apps/api/window';
   * const appWindow = new Window('my-label');
   * appWindow.once('tauri://created', function () {
   *  // window successfully created
   * });
   * appWindow.once('tauri://error', function (e) {
   *  // an error happened creating the window
   * });
   * ```
   *
   * @param label The unique window label. Must be alphanumeric: `a-zA-Z-/:_`.
   * @param options The window configuration, see {@link WindowOptions}.
   * @returns The {@link Window} instance to communicate with the window.
   *
   * @remarks Requires the `core:window:allow-create` permission (not included in
   * `core:window:default`).
   */
  constructor(label: WindowLabel, options: WindowOptions = {}) {
    this.label = label
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.listeners = Object.create(null)

    // @ts-expect-error `skip` is not a public API so it is not defined in WindowOptions
    if (!options?.skip) {
      invoke('plugin:window|create', {
        options: {
          ...options,
          parent:
            typeof options.parent === 'string'
              ? options.parent
              : options.parent?.label,
          label
        }
      })
        .then(async () => this.emit('tauri://created'))
        .catch(async (e: string) => this.emit('tauri://error', e))
    }
  }

  /**
   * Gets the Window associated with the given label.
   * @example
   * ```typescript
   * import { Window } from '@tauri-apps/api/window';
   * const mainWindow = Window.getByLabel('main');
   * ```
   *
   * @param label The window label.
   * @returns The Window instance to communicate with the window or null if the window doesn't exist.
   */
  static async getByLabel(label: string): Promise<Window | null> {
    return (await getAllWindows()).find((w) => w.label === label) ?? null
  }

  /**
   * Get an instance of `Window` for the current window.
   */
  static getCurrent(): Window {
    return getCurrentWindow()
  }

  /**
   * Gets a list of instances of `Window` for all available windows.
   */
  static async getAll(): Promise<Window[]> {
    return getAllWindows()
  }

  /**
   *  Gets the focused window.
   * @example
   * ```typescript
   * import { Window } from '@tauri-apps/api/window';
   * const focusedWindow = Window.getFocusedWindow();
   * ```
   *
   * @returns The Window instance or `undefined` if there is not any focused window.
   */
  static async getFocusedWindow(): Promise<Window | null> {
    for (const w of await getAllWindows()) {
      if (await w.isFocused()) {
        return w
      }
    }
    return null
  }

  /**
   * Listen to an emitted event on this window.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const unlisten = await getCurrentWindow().listen<string>('state-changed', (event) => {
   *   console.log(`Got error: ${payload}`);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async listen<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return () => {
        // eslint-disable-next-line security/detect-object-injection
        const listeners = this.listeners[event]
        listeners.splice(listeners.indexOf(handler), 1)
      }
    }
    return listen(event, handler, {
      target: { kind: 'Window', label: this.label }
    })
  }

  /**
   * Listen to an emitted event on this window only once.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const unlisten = await getCurrentWindow().once<null>('initialized', (event) => {
   *   console.log(`Window initialized!`);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener removes itself after the first event and is also removed
   * automatically when this window is destroyed. Do call the returned function if
   * the listener's own scope ends before the event arrives.
   */
  async once<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return () => {
        // eslint-disable-next-line security/detect-object-injection
        const listeners = this.listeners[event]
        listeners.splice(listeners.indexOf(handler), 1)
      }
    }
    return once(event, handler, {
      target: { kind: 'Window', label: this.label }
    })
  }

  /**
   * Emits an event to all {@link EventTarget|targets}.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().emit('window-loaded', { loggedIn: true, token: 'authToken' });
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param payload Event payload.
   */
  async emit<T>(event: string, payload?: T): Promise<void> {
    if (localTauriEvents.includes(event)) {
      // eslint-disable-next-line
      for (const handler of this.listeners[event] || []) {
        handler({
          event,
          id: -1,
          payload
        })
      }
      return
    }
    return emit<T>(event, payload)
  }

  /**
   * Emits an event to all {@link EventTarget|targets} matching the given target.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().emitTo('main', 'window-loaded', { loggedIn: true, token: 'authToken' });
   * ```
   * @param target Label of the target Window/Webview/WebviewWindow or raw {@link EventTarget} object.
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param payload Event payload.
   */
  async emitTo<T>(
    target: string | EventTarget,
    event: string,
    payload?: T
  ): Promise<void> {
    if (localTauriEvents.includes(event)) {
      // eslint-disable-next-line security/detect-object-injection
      for (const handler of this.listeners[event] || []) {
        handler({
          event,
          id: -1,
          payload
        })
      }
      return
    }
    return emitTo<T>(target, event, payload)
  }

  /** @ignore */
  _handleTauriEvent<T>(event: string, handler: EventCallback<T>): boolean {
    if (localTauriEvents.includes(event)) {
      if (!(event in this.listeners)) {
        // eslint-disable-next-line
        this.listeners[event] = [handler]
      } else {
        // eslint-disable-next-line
        this.listeners[event].push(handler)
      }
      return true
    }
    return false
  }

  // Getters
  /**
   * The scale factor that can be used to map physical pixels to logical pixels.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const factor = await getCurrentWindow().scaleFactor();
   * ```
   *
   * @returns The window's monitor scale factor.
   */
  async scaleFactor(): Promise<number> {
    return invoke('plugin:window|scale_factor', {
      label: this.label
    })
  }

  /**
   * The position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const position = await getCurrentWindow().innerPosition();
   * ```
   *
   * @returns The window's inner position.
   */
  async innerPosition(): Promise<PhysicalPosition> {
    return invoke<{ x: number; y: number }>('plugin:window|inner_position', {
      label: this.label
    }).then((p) => new PhysicalPosition(p))
  }

  /**
   * The position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const position = await getCurrentWindow().outerPosition();
   * ```
   *
   * @returns The window's outer position.
   */
  async outerPosition(): Promise<PhysicalPosition> {
    return invoke<{ x: number; y: number }>('plugin:window|outer_position', {
      label: this.label
    }).then((p) => new PhysicalPosition(p))
  }

  /**
   * The physical size of the window's client area.
   * The client area is the content of the window, excluding the title bar and borders.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const size = await getCurrentWindow().innerSize();
   * ```
   *
   * @returns The window's inner size.
   */
  async innerSize(): Promise<PhysicalSize> {
    return invoke<{ width: number; height: number }>(
      'plugin:window|inner_size',
      {
        label: this.label
      }
    ).then((s) => new PhysicalSize(s))
  }

  /**
   * The physical size of the entire window.
   * These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const size = await getCurrentWindow().outerSize();
   * ```
   *
   * @returns The window's outer size.
   */
  async outerSize(): Promise<PhysicalSize> {
    return invoke<{ width: number; height: number }>(
      'plugin:window|outer_size',
      {
        label: this.label
      }
    ).then((s) => new PhysicalSize(s))
  }

  /**
   * Gets the window's current fullscreen state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const fullscreen = await getCurrentWindow().isFullscreen();
   * ```
   *
   * @returns Whether the window is in fullscreen mode or not.
   */
  async isFullscreen(): Promise<boolean> {
    return invoke('plugin:window|is_fullscreen', {
      label: this.label
    })
  }

  /**
   * Gets the window's current minimized state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const minimized = await getCurrentWindow().isMinimized();
   * ```
   */
  async isMinimized(): Promise<boolean> {
    return invoke('plugin:window|is_minimized', {
      label: this.label
    })
  }

  /**
   * Gets the window's current maximized state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const maximized = await getCurrentWindow().isMaximized();
   * ```
   *
   * @returns Whether the window is maximized or not.
   */
  async isMaximized(): Promise<boolean> {
    return invoke('plugin:window|is_maximized', {
      label: this.label
    })
  }

  /**
   * Gets the window's current focus state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const focused = await getCurrentWindow().isFocused();
   * ```
   *
   * @returns Whether the window is focused or not.
   */
  async isFocused(): Promise<boolean> {
    return invoke('plugin:window|is_focused', {
      label: this.label
    })
  }

  /**
   * Gets the window's current decorated state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const decorated = await getCurrentWindow().isDecorated();
   * ```
   *
   * @returns Whether the window is decorated or not.
   */
  async isDecorated(): Promise<boolean> {
    return invoke('plugin:window|is_decorated', {
      label: this.label
    })
  }

  /**
   * Gets the window's current resizable state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const resizable = await getCurrentWindow().isResizable();
   * ```
   *
   * @returns Whether the window is resizable or not.
   */
  async isResizable(): Promise<boolean> {
    return invoke('plugin:window|is_resizable', {
      label: this.label
    })
  }

  /**
   * Gets the window's native maximize button state.
   *
   * #### Platform-specific
   *
   * - **Linux / iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const maximizable = await getCurrentWindow().isMaximizable();
   * ```
   *
   * @returns Whether the window's native maximize button is enabled or not.
   */
  async isMaximizable(): Promise<boolean> {
    return invoke('plugin:window|is_maximizable', {
      label: this.label
    })
  }

  /**
   * Gets the window's native minimize button state.
   *
   * #### Platform-specific
   *
   * - **Linux / iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const minimizable = await getCurrentWindow().isMinimizable();
   * ```
   *
   * @returns Whether the window's native minimize button is enabled or not.
   */
  async isMinimizable(): Promise<boolean> {
    return invoke('plugin:window|is_minimizable', {
      label: this.label
    })
  }

  /**
   * Gets the window's native close button state.
   *
   * #### Platform-specific
   *
   * - **iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const closable = await getCurrentWindow().isClosable();
   * ```
   *
   * @returns Whether the window's native close button is enabled or not.
   */
  async isClosable(): Promise<boolean> {
    return invoke('plugin:window|is_closable', {
      label: this.label
    })
  }

  /**
   * Gets the window's current visible state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const visible = await getCurrentWindow().isVisible();
   * ```
   *
   * @returns Whether the window is visible or not.
   */
  async isVisible(): Promise<boolean> {
    return invoke('plugin:window|is_visible', {
      label: this.label
    })
  }

  /**
   * Gets the window's current title.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const title = await getCurrentWindow().title();
   * ```
   */
  async title(): Promise<string> {
    return invoke('plugin:window|title', {
      label: this.label
    })
  }

  /**
   * Gets the window's current theme.
   *
   * #### Platform-specific
   *
   * - **macOS:** Theme was introduced on macOS 10.14. Returns `light` on macOS 10.13 and below.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const theme = await getCurrentWindow().theme();
   * ```
   *
   * @returns The window theme.
   */
  async theme(): Promise<Theme | null> {
    return invoke('plugin:window|theme', {
      label: this.label
    })
  }

  /**
   * Whether the window is configured to be always on top of other windows or not.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const alwaysOnTop = await getCurrentWindow().isAlwaysOnTop();
   * ```
   *
   * @returns Whether the window is visible or not.
   */
  async isAlwaysOnTop(): Promise<boolean> {
    return invoke('plugin:window|is_always_on_top', {
      label: this.label
    })
  }

  /**
   * The name of the Android activity hosting this window.
   *
   * On Android each window is backed by its own activity. Use this to tell windows
   * apart from native code, or to match a window with the activity declared in your
   * `AndroidManifest.xml`.
   *
   * #### Platform-specific
   *
   * - **Android:** Supported.
   * - **Windows / Linux / macOS / iOS:** Unsupported, the call rejects because the
   *   command is not registered.
   *
   * See the [mobile multiwindow guide](https://tauri.app/learn/mobile-multiwindow/).
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const activity = await getCurrentWindow().activityName();
   * ```
   *
   * @returns The activity name.
   *
   * @remarks Uses the `core:window:allow-activity-name` permission, which is part
   * of `core:window:default`.
   *
   * @since 2.11.0
   */
  async activityName(): Promise<string> {
    return invoke('plugin:window|activity_name', {
      label: this.label
    })
  }

  /**
   * The identifier of the iOS scene hosting this window.
   *
   * On iOS each window is backed by a `UIScene`. Use this to tell windows apart
   * from native code, or to match a window with a scene configuration declared in
   * your `Info.plist`.
   *
   * #### Platform-specific
   *
   * - **iOS:** Supported.
   * - **Windows / Linux / macOS / Android:** Unsupported, the call rejects because
   *   the command is not registered.
   *
   * See the [mobile multiwindow guide](https://tauri.app/learn/mobile-multiwindow/).
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const scene = await getCurrentWindow().sceneIdentifier();
   * ```
   *
   * @returns The scene identifier.
   *
   * @remarks Uses the `core:window:allow-scene-identifier` permission, which is part
   * of `core:window:default`.
   *
   * @since 2.11.0
   */
  async sceneIdentifier(): Promise<string> {
    return invoke('plugin:window|scene_identifier', {
      label: this.label
    })
  }

  // Setters

  /**
   * Centers the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().center();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-center` permission (not included
   * in `core:window:default`).
   */
  async center(): Promise<void> {
    return invoke('plugin:window|center', {
      label: this.label
    })
  }

  /**
   *  Requests user attention to the window, this has no effect if the application
   * is already focused. How requesting for user attention manifests is platform dependent,
   * see `UserAttentionType` for details.
   *
   * Providing `null` will unset the request for user attention. Unsetting the request for
   * user attention might not be done automatically by the WM when the window receives input.
   *
   * #### Platform-specific
   *
   * - **macOS:** `null` has no effect.
   * - **Linux:** Urgency levels have the same effect.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().requestUserAttention();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-request-user-attention` permission (not included
   * in `core:window:default`).
   */
  async requestUserAttention(
    requestType: UserAttentionType | null
  ): Promise<void> {
    let requestType_ = null
    if (requestType) {
      if (requestType === UserAttentionType.Critical) {
        requestType_ = { type: 'Critical' }
      } else {
        requestType_ = { type: 'Informational' }
      }
    }

    return invoke('plugin:window|request_user_attention', {
      label: this.label,
      value: requestType_
    })
  }

  /**
   * Updates the window resizable flag.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setResizable(false);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-resizable` permission (not included
   * in `core:window:default`).
   */
  async setResizable(resizable: boolean): Promise<void> {
    return invoke('plugin:window|set_resizable', {
      label: this.label,
      value: resizable
    })
  }

  /**
   * Enable or disable the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setEnabled(false);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-enabled` permission (not included
   * in `core:window:default`).
   *
   * @since 2.0.0
   */
  async setEnabled(enabled: boolean): Promise<void> {
    return invoke('plugin:window|set_enabled', {
      label: this.label,
      value: enabled
    })
  }

  /**
   * Whether the window is enabled or disabled.
   *
   * A disabled window ignores all user input; see {@link Window.setEnabled}.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const enabled = await getCurrentWindow().isEnabled();
   * ```
   *
   * @returns `true` when the window accepts user input, `false` when it is disabled.
   *
   * @since 2.0.0
   */
  async isEnabled(): Promise<boolean> {
    return invoke('plugin:window|is_enabled', {
      label: this.label
    })
  }

  /**
   * Sets whether the window's native maximize button is enabled or not.
   * If resizable is set to false, this setting is ignored.
   *
   * #### Platform-specific
   *
   * - **macOS:** Disables the "zoom" button in the window titlebar, which is also used to enter fullscreen mode.
   * - **Linux / iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setMaximizable(false);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-maximizable` permission (not included
   * in `core:window:default`).
   */
  async setMaximizable(maximizable: boolean): Promise<void> {
    return invoke('plugin:window|set_maximizable', {
      label: this.label,
      value: maximizable
    })
  }

  /**
   * Sets whether the window's native minimize button is enabled or not.
   *
   * #### Platform-specific
   *
   * - **Linux / iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setMinimizable(false);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-minimizable` permission (not included
   * in `core:window:default`).
   */
  async setMinimizable(minimizable: boolean): Promise<void> {
    return invoke('plugin:window|set_minimizable', {
      label: this.label,
      value: minimizable
    })
  }

  /**
   * Sets whether the window's native close button is enabled or not.
   *
   * #### Platform-specific
   *
   * - **Linux:** GTK+ will do its best to convince the window manager not to show a close button. Depending on the system, this function may not have any effect when called on a window that is already visible
   * - **iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setClosable(false);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-closable` permission (not included
   * in `core:window:default`).
   */
  async setClosable(closable: boolean): Promise<void> {
    return invoke('plugin:window|set_closable', {
      label: this.label,
      value: closable
    })
  }

  /**
   * Sets the window title.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setTitle('Tauri');
   * ```
   *
   * @param title The new title
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-title` permission (not included
   * in `core:window:default`).
   */
  async setTitle(title: string): Promise<void> {
    return invoke('plugin:window|set_title', {
      label: this.label,
      value: title
    })
  }

  /**
   * Maximizes the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().maximize();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-maximize` permission (not included
   * in `core:window:default`).
   */
  async maximize(): Promise<void> {
    return invoke('plugin:window|maximize', {
      label: this.label
    })
  }

  /**
   * Unmaximizes the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().unmaximize();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-unmaximize` permission (not included
   * in `core:window:default`).
   */
  async unmaximize(): Promise<void> {
    return invoke('plugin:window|unmaximize', {
      label: this.label
    })
  }

  /**
   * Toggles the window maximized state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().toggleMaximize();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-toggle-maximize` permission (not included
   * in `core:window:default`).
   */
  async toggleMaximize(): Promise<void> {
    return invoke('plugin:window|toggle_maximize', {
      label: this.label
    })
  }

  /**
   * Minimizes the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().minimize();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-minimize` permission (not included
   * in `core:window:default`).
   */
  async minimize(): Promise<void> {
    return invoke('plugin:window|minimize', {
      label: this.label
    })
  }

  /**
   * Unminimizes the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().unminimize();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-unminimize` permission (not included
   * in `core:window:default`).
   */
  async unminimize(): Promise<void> {
    return invoke('plugin:window|unminimize', {
      label: this.label
    })
  }

  /**
   * Sets the window visibility to true.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().show();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-show` permission (not included
   * in `core:window:default`).
   */
  async show(): Promise<void> {
    return invoke('plugin:window|show', {
      label: this.label
    })
  }

  /**
   * Sets the window visibility to false.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().hide();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-hide` permission (not included
   * in `core:window:default`).
   */
  async hide(): Promise<void> {
    return invoke('plugin:window|hide', {
      label: this.label
    })
  }

  /**
   * Closes the window.
   *
   * Note this emits a closeRequested event so you can intercept it. To force window close, use {@link Window.destroy}.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().close();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-close` permission (not included
   * in `core:window:default`).
   */
  async close(): Promise<void> {
    return invoke('plugin:window|close', {
      label: this.label
    })
  }

  /**
   * Destroys the window. Behaves like {@link Window.close} but forces the window close instead of emitting a closeRequested event.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().destroy();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-destroy` permission (not included
   * in `core:window:default`).
   */
  async destroy(): Promise<void> {
    return invoke('plugin:window|destroy', {
      label: this.label
    })
  }

  /**
   * Whether the window should have borders and bars.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setDecorations(false);
   * ```
   *
   * @param decorations Whether the window should have borders and bars.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-decorations` permission (not included
   * in `core:window:default`).
   */
  async setDecorations(decorations: boolean): Promise<void> {
    return invoke('plugin:window|set_decorations', {
      label: this.label,
      value: decorations
    })
  }

  /**
   * Whether or not the window should have shadow.
   *
   * #### Platform-specific
   *
   * - **Windows:**
   *   - `false` has no effect on decorated window, shadows are always ON.
   *   - `true` will make undecorated window have a 1px white border,
   * and on Windows 11, it will have a rounded corners.
   * - **Linux:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setShadow(false);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-shadow` permission (not included
   * in `core:window:default`).
   */
  async setShadow(enable: boolean): Promise<void> {
    return invoke('plugin:window|set_shadow', {
      label: this.label,
      value: enable
    })
  }

  /**
   * Applies platform-specific window effects such as Mica, Acrylic, Blur or the
   * macOS vibrancy materials.
   *
   * Requires the window to be transparent, so create it with `transparent: true`
   * (or set `transparent` in `tauri.conf.json`). Give the page a transparent or
   * translucent background as well, otherwise the effect is hidden behind your own
   * background color.
   *
   * Conflicting effects are resolved by applying the first supported one and
   * ignoring the rest, so you can list a Windows and a macOS effect together.
   *
   * #### Platform-specific
   *
   * - **Windows:** If the window uses decorations or shadows, you may need
   *   [this workaround](https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891).
   * - **Linux:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow, Effect, EffectState } from '@tauri-apps/api/window';
   *
   * await getCurrentWindow().setEffects({
   *   effects: [Effect.Mica, Effect.Acrylic, Effect.UnderWindowBackground],
   *   state: EffectState.Active,
   *   radius: 8
   * });
   * ```
   *
   * @param effects The effects to apply, see {@link Effects}.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-effects` permission (not included
   * in `core:window:default`).
   *
   * @since 2.0.0
   */
  async setEffects(effects: Effects): Promise<void> {
    return invoke('plugin:window|set_effects', {
      label: this.label,
      value: effects
    })
  }

  /**
   * Clear any applied effects if possible.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().clearEffects();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-effects` permission (not included
   * in `core:window:default`), the same one used by {@link Window.setEffects}.
   *
   * @since 2.0.0
   */
  async clearEffects(): Promise<void> {
    return invoke('plugin:window|set_effects', {
      label: this.label,
      value: null
    })
  }

  /**
   * Whether the window should always be on top of other windows.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setAlwaysOnTop(true);
   * ```
   *
   * @param alwaysOnTop Whether the window should always be on top of other windows or not.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-always-on-top` permission (not included
   * in `core:window:default`).
   */
  async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
    return invoke('plugin:window|set_always_on_top', {
      label: this.label,
      value: alwaysOnTop
    })
  }

  /**
   * Whether the window should always be below other windows.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setAlwaysOnBottom(true);
   * ```
   *
   * @param alwaysOnBottom Whether the window should always be below other windows or not.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-always-on-bottom` permission (not included
   * in `core:window:default`).
   */
  async setAlwaysOnBottom(alwaysOnBottom: boolean): Promise<void> {
    return invoke('plugin:window|set_always_on_bottom', {
      label: this.label,
      value: alwaysOnBottom
    })
  }

  /**
   * Prevents the window contents from being captured by other apps.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setContentProtected(true);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-content-protected` permission (not included
   * in `core:window:default`).
   */
  async setContentProtected(protected_: boolean): Promise<void> {
    return invoke('plugin:window|set_content_protected', {
      label: this.label,
      value: protected_
    })
  }

  /**
   * Resizes the window with a new inner size.
   * @example
   * ```typescript
   * import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
   * await getCurrentWindow().setSize(new LogicalSize(600, 500));
   * ```
   *
   * @param size The logical or physical inner size.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-size` permission (not included
   * in `core:window:default`).
   */
  async setSize(size: LogicalSize | PhysicalSize | Size): Promise<void> {
    return invoke('plugin:window|set_size', {
      label: this.label,
      value: size instanceof Size ? size : new Size(size)
    })
  }

  /**
   * Sets the window minimum inner size. If the `size` argument is not provided, the constraint is unset.
   * @example
   * ```typescript
   * import { getCurrentWindow, PhysicalSize } from '@tauri-apps/api/window';
   * await getCurrentWindow().setMinSize(new PhysicalSize(600, 500));
   * ```
   *
   * @param size The logical or physical inner size, or `null` to unset the constraint.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-min-size` permission (not included
   * in `core:window:default`).
   */
  async setMinSize(
    size: LogicalSize | PhysicalSize | Size | null | undefined
  ): Promise<void> {
    return invoke('plugin:window|set_min_size', {
      label: this.label,
      value: size instanceof Size ? size : size ? new Size(size) : null
    })
  }

  /**
   * Sets the window maximum inner size. If the `size` argument is undefined, the constraint is unset.
   * @example
   * ```typescript
   * import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
   * await getCurrentWindow().setMaxSize(new LogicalSize(600, 500));
   * ```
   *
   * @param size The logical or physical inner size, or `null` to unset the constraint.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-max-size` permission (not included
   * in `core:window:default`).
   */
  async setMaxSize(
    size: LogicalSize | PhysicalSize | Size | null | undefined
  ): Promise<void> {
    return invoke('plugin:window|set_max_size', {
      label: this.label,
      value: size instanceof Size ? size : size ? new Size(size) : null
    })
  }

  /**
   * Sets the window inner size constraints.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setSizeConstraints({ minWidth: 300 });
   * ```
   *
   * @param constraints The inner size constraints in **logical pixels**, or `null`
   * to unset every constraint. Each field is optional and independent, so
   * `{ minWidth: 300 }` constrains only the width.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-size-constraints` permission (not included
   * in `core:window:default`).
   */
  async setSizeConstraints(
    constraints: WindowSizeConstraints | null | undefined
  ): Promise<void> {
    function logical(pixel?: number): { Logical: number } | null {
      return pixel ? { Logical: pixel } : null
    }

    return invoke('plugin:window|set_size_constraints', {
      label: this.label,
      value: {
        minWidth: logical(constraints?.minWidth),
        minHeight: logical(constraints?.minHeight),
        maxWidth: logical(constraints?.maxWidth),
        maxHeight: logical(constraints?.maxHeight)
      }
    })
  }

  /**
   * Sets the window outer position.
   * @example
   * ```typescript
   * import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';
   * await getCurrentWindow().setPosition(new LogicalPosition(600, 500));
   * ```
   *
   * @param position The new position, in logical or physical pixels.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-position` permission (not included
   * in `core:window:default`).
   */
  async setPosition(
    position: LogicalPosition | PhysicalPosition | Position
  ): Promise<void> {
    return invoke('plugin:window|set_position', {
      label: this.label,
      value: position instanceof Position ? position : new Position(position)
    })
  }

  /**
   * Sets the window fullscreen state.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setFullscreen(true);
   * ```
   *
   * @param fullscreen Whether the window should go to fullscreen or not.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-fullscreen` permission (not included
   * in `core:window:default`).
   */
  async setFullscreen(fullscreen: boolean): Promise<void> {
    return invoke('plugin:window|set_fullscreen', {
      label: this.label,
      value: fullscreen
    })
  }

  /**
   * Sets the window as fullscreen on the monitor that contains the given physical position.
   *
   * Does nothing if no monitor contains the position.
   * @example
   * ```typescript
   * import { getCurrentWindow, availableMonitors } from '@tauri-apps/api/window';
   * const monitors = await availableMonitors();
   * if (monitors.length > 1) {
   *   await getCurrentWindow().setFullscreenOnMonitor(monitors[1].position);
   * }
   * ```
   *
   * @param position A physical position inside the target monitor, such as {@linkcode Monitor.position}.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-fullscreen-on-monitor` permission (not included
   * in `core:window:default`).
   *
   * @since 2.12.0
   */
  async setFullscreenOnMonitor(position: PhysicalPosition): Promise<void> {
    return invoke('plugin:window|set_fullscreen_on_monitor', {
      label: this.label,
      value: position
    })
  }

  /**
   * On macOS, Toggles a fullscreen mode that doesn’t require a new macOS space. Returns a boolean indicating whether the transition was successful (this won’t work if the window was already in the native fullscreen).
   * This is how fullscreen used to work on macOS in versions before Lion. And allows the user to have a fullscreen window without using another space or taking control over the entire monitor.
   *
   * On other platforms, this is the same as {@link Window.setFullscreen}.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setSimpleFullscreen(true);
   * ```
   *
   * @param fullscreen Whether the window should go to simple fullscreen or not.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-simple-fullscreen` permission (not included
   * in `core:window:default`).
   *
   * @since 2.8.0
   */
  async setSimpleFullscreen(fullscreen: boolean): Promise<void> {
    return invoke('plugin:window|set_simple_fullscreen', {
      label: this.label,
      value: fullscreen
    })
  }

  /**
   * Bring the window to front and focus.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setFocus();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-focus` permission (not included
   * in `core:window:default`).
   */
  async setFocus(): Promise<void> {
    return invoke('plugin:window|set_focus', {
      label: this.label
    })
  }

  /**
   * Sets whether the window can be focused.
   *
   * #### Platform-specific
   *
   * - **macOS**: If the window is already focused, it is not possible to unfocus it after calling `set_focusable(false)`.
   *   In this case, you might consider calling {@link Window.setFocus} but it will move the window to the back i.e. at the bottom in terms of z-order.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setFocusable(true);
   * ```
   *
   * @param focusable Whether the window can be focused.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-focusable` permission (not included
   * in `core:window:default`).
   */
  async setFocusable(focusable: boolean): Promise<void> {
    return invoke('plugin:window|set_focusable', {
      label: this.label,
      value: focusable
    })
  }

  /**
   * Sets the window icon.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setIcon('/tauri/awesome.png');
   * ```
   *
   * Note that you may need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   *
   * @param icon Icon bytes or path to the icon file.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-icon` permission (not included
   * in `core:window:default`).
   */
  async setIcon(icon: JsImage): Promise<void> {
    return invoke('plugin:window|set_icon', {
      label: this.label,
      value: transformImage(icon)
    })
  }

  /**
   * Whether the window icon should be hidden from the taskbar or not.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setSkipTaskbar(true);
   * ```
   *
   * @param skip true to hide window icon, false to show it.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-skip-taskbar` permission (not included
   * in `core:window:default`).
   */
  async setSkipTaskbar(skip: boolean): Promise<void> {
    return invoke('plugin:window|set_skip_taskbar', {
      label: this.label,
      value: skip
    })
  }

  /**
   * Grabs the cursor, preventing it from leaving the window.
   *
   * There's no guarantee that the cursor will be hidden. You should
   * hide it by yourself if you want so.
   *
   * #### Platform-specific
   *
   * - **Linux:** Unsupported.
   * - **macOS:** This locks the cursor in a fixed location, which looks visually awkward.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setCursorGrab(true);
   * ```
   *
   * @param grab `true` to grab the cursor icon, `false` to release it.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-cursor-grab` permission (not included
   * in `core:window:default`).
   */
  async setCursorGrab(grab: boolean): Promise<void> {
    return invoke('plugin:window|set_cursor_grab', {
      label: this.label,
      value: grab
    })
  }

  /**
   * Modifies the cursor's visibility.
   *
   * #### Platform-specific
   *
   * - **Windows:** The cursor is only hidden within the confines of the window.
   * - **macOS:** The cursor is hidden as long as the window has input focus, even if the cursor is
   *   outside of the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setCursorVisible(false);
   * ```
   *
   * @param visible If `false`, this will hide the cursor. If `true`, this will show the cursor.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-cursor-visible` permission (not included
   * in `core:window:default`).
   */
  async setCursorVisible(visible: boolean): Promise<void> {
    return invoke('plugin:window|set_cursor_visible', {
      label: this.label,
      value: visible
    })
  }

  /**
   * Modifies the cursor icon of the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setCursorIcon('help');
   * ```
   *
   * @param icon The new cursor icon.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-cursor-icon` permission (not included
   * in `core:window:default`).
   */
  async setCursorIcon(icon: CursorIcon): Promise<void> {
    return invoke('plugin:window|set_cursor_icon', {
      label: this.label,
      value: icon
    })
  }

  /**
   * Sets the window background color.
   *
   * #### Platform-specific:
   *
   * - **Windows:** alpha channel is ignored.
   * - **iOS / Android:** Unsupported.
   *
   * This sets the color of the window itself, which is what you see while the
   * webview is still loading or behind a transparent page. To also change the
   * webview background use {@link WebviewWindow.setBackgroundColor}.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setBackgroundColor('#2f2f2f');
   * // also accepts an RGB/RGBA tuple or an object
   * await getCurrentWindow().setBackgroundColor([47, 47, 47, 255]);
   * ```
   *
   * @param color The new background color, see {@link Color}.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-background-color` permission (not included
   * in `core:window:default`).
   *
   * @since 2.1.0
   */
  async setBackgroundColor(color: Color): Promise<void> {
    return invoke('plugin:window|set_background_color', {
      label: this.label,
      value: color
    })
  }

  /**
   * Changes the position of the cursor in window coordinates.
   * @example
   * ```typescript
   * import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window';
   * await getCurrentWindow().setCursorPosition(new LogicalPosition(600, 300));
   * ```
   *
   * @param position The new cursor position.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-cursor-position` permission (not included
   * in `core:window:default`).
   */
  async setCursorPosition(
    position: LogicalPosition | PhysicalPosition | Position
  ): Promise<void> {
    return invoke('plugin:window|set_cursor_position', {
      label: this.label,
      value: position instanceof Position ? position : new Position(position)
    })
  }

  /**
   * Changes the cursor events behavior.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setIgnoreCursorEvents(true);
   * ```
   *
   * @param ignore `true` to ignore the cursor events; `false` to process them as usual.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-ignore-cursor-events` permission (not included
   * in `core:window:default`).
   */
  async setIgnoreCursorEvents(ignore: boolean): Promise<void> {
    return invoke('plugin:window|set_ignore_cursor_events', {
      label: this.label,
      value: ignore
    })
  }

  /**
   * Starts dragging the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().startDragging();
   * ```
   *
   * @return A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-start-dragging` permission (not included
   * in `core:window:default`).
   */
  async startDragging(): Promise<void> {
    return invoke('plugin:window|start_dragging', {
      label: this.label
    })
  }

  /**
   * Starts resize-dragging the window.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().startResizeDragging();
   * ```
   *
   * @return A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-start-resize-dragging` permission (not included
   * in `core:window:default`).
   */
  async startResizeDragging(direction: ResizeDirection): Promise<void> {
    return invoke('plugin:window|start_resize_dragging', {
      label: this.label,
      value: direction
    })
  }

  /**
   * Sets the badge count. It is app wide and not specific to this window.
   *
   * #### Platform-specific
   *
   * - **Windows**: Unsupported. Use {@linkcode Window.setOverlayIcon} instead.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setBadgeCount(5);
   * ```
   *
   * @param count The badge count. Use `undefined` to remove the badge.
   * @return A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-badge-count` permission (not included
   * in `core:window:default`).
   */
  async setBadgeCount(count?: number): Promise<void> {
    return invoke('plugin:window|set_badge_count', {
      label: this.label,
      value: count
    })
  }

  /**
   * Sets the badge label shown on the app's dock icon. **macOS only**
   *
   * Unlike {@linkcode Window.setBadgeCount}, which takes a number, this shows
   * arbitrary short text. It is app-wide and not specific to this window.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setBadgeLabel("Hello");
   * ```
   *
   * @param label The badge label. Use `undefined` to remove the badge.
   * @return A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-badge-label` permission (not included
   * in `core:window:default`).
   */
  async setBadgeLabel(label?: string): Promise<void> {
    return invoke('plugin:window|set_badge_label', {
      label: this.label,
      value: label
    })
  }

  /**
   * Sets the overlay icon. **Windows only**
   * The overlay icon can be set for every window.
   *
   *
   * Note that you may need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   *
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setOverlayIcon("/tauri/awesome.png");
   * ```
   *
   * @param icon Icon bytes or path to the icon file. Use `undefined` to remove the overlay icon.
   * @return A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-overlay-icon` permission (not included
   * in `core:window:default`).
   */
  async setOverlayIcon(icon?: JsImage): Promise<void> {
    return invoke('plugin:window|set_overlay_icon', {
      label: this.label,
      value: icon ? transformImage(icon) : undefined
    })
  }

  /**
   * Sets the taskbar progress state.
   *
   * #### Platform-specific
   *
   * - **Linux / macOS**: Progress bar is app-wide and not specific to this window.
   * - **Linux**: Only supported desktop environments with `libunity` (e.g. GNOME).
   *
   * @example
   * ```typescript
   * import { getCurrentWindow, ProgressBarStatus } from '@tauri-apps/api/window';
   * await getCurrentWindow().setProgressBar({
   *   status: ProgressBarStatus.Normal,
   *   progress: 50,
   * });
   * ```
   *
   * @return A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-progress-bar` permission (not included
   * in `core:window:default`).
   */
  async setProgressBar(state: ProgressBarState): Promise<void> {
    return invoke('plugin:window|set_progress_bar', {
      label: this.label,
      value: state
    })
  }

  /**
   * Sets whether the window should be visible on all workspaces or virtual desktops.
   *
   * #### Platform-specific
   *
   * - **Windows / iOS / Android:** Unsupported.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setVisibleOnAllWorkspaces(true);
   * ```
   *
   * @param visible Whether the window should follow the user across workspaces.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-visible-on-all-workspaces` permission (not included
   * in `core:window:default`).
   *
   * @since 2.0.0
   */
  async setVisibleOnAllWorkspaces(visible: boolean): Promise<void> {
    return invoke('plugin:window|set_visible_on_all_workspaces', {
      label: this.label,
      value: visible
    })
  }

  /**
   * Sets the title bar style. **macOS only**.
   *
   * Use `transparent` or `overlay` to build a custom title bar: with `overlay` the
   * page extends under the title bar and only the traffic lights remain, so leave
   * room for them in your layout and pair it with {@link Window.startDragging} on a
   * draggable region.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setTitleBarStyle('overlay');
   * ```
   *
   * @param style The new title bar style, see {@link TitleBarStyle}.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-title-bar-style` permission (not included
   * in `core:window:default`).
   *
   * @since 2.0.0
   */
  async setTitleBarStyle(style: TitleBarStyle): Promise<void> {
    return invoke('plugin:window|set_title_bar_style', {
      label: this.label,
      value: style
    })
  }

  /**
   * Set window theme, pass in `null` or `undefined` to follow system theme
   *
   * #### Platform-specific
   *
   * - **Linux / macOS**: Theme is app-wide and not specific to this window.
   * - **iOS / Android:** Unsupported.
   *
   * Use {@link Window.theme} to read the effective theme and
   * {@link Window.onThemeChanged} to react to changes.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * await getCurrentWindow().setTheme('dark');
   * // follow the system theme again
   * await getCurrentWindow().setTheme(null);
   * ```
   *
   * @param theme The theme to apply, or `null`/`undefined` to follow the system theme.
   * @returns A promise indicating the success or failure of the operation.
   *
   * @remarks Requires the `core:window:allow-set-theme` permission (not included
   * in `core:window:default`).
   *
   * @since 2.0.0
   */
  async setTheme(theme?: Theme | null): Promise<void> {
    return invoke('plugin:window|set_theme', {
      label: this.label,
      value: theme
    })
  }

  // Listeners

  /**
   * Listen to window resize.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * const unlisten = await getCurrentWindow().onResized(({ payload: size }) => {
   *  console.log('Window resized', size);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onResized(handler: EventCallback<PhysicalSize>): Promise<UnlistenFn> {
    return this.listen<PhysicalSize>(TauriEvent.WINDOW_RESIZED, (e) => {
      e.payload = new PhysicalSize(e.payload)
      handler(e)
    })
  }

  /**
   * Listen to window move.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * const unlisten = await getCurrentWindow().onMoved(({ payload: position }) => {
   *  console.log('Window moved', position);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onMoved(handler: EventCallback<PhysicalPosition>): Promise<UnlistenFn> {
    return this.listen<PhysicalPosition>(TauriEvent.WINDOW_MOVED, (e) => {
      e.payload = new PhysicalPosition(e.payload)
      handler(e)
    })
  }

  /**
   * Listen to window close requested. Emitted when the user requests to closes the window.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * // `confirm` comes from the dialog plugin, which you have to add separately:
   * // `pnpm tauri add dialog`
   * import { confirm } from '@tauri-apps/plugin-dialog';
   * const unlisten = await getCurrentWindow().onCloseRequested(async (event) => {
   *   const confirmed = await confirm('Are you sure?');
   *   if (!confirmed) {
   *     // user did not confirm closing the window; let's prevent it
   *     event.preventDefault();
   *   }
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onCloseRequested(
    handler: (event: CloseRequestedEvent) => void | Promise<void>
  ): Promise<UnlistenFn> {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    return this.listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async (event) => {
      const evt = new CloseRequestedEvent(event)
      await handler(evt)
      if (!evt.isPreventDefault()) {
        await this.destroy()
      }
    })
  }

  /**
   * Listen to a file drop event.
   * The listener is triggered when the user hovers the selected files on the webview,
   * drops the files or cancels the operation.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * const unlisten = await getCurrentWindow().onDragDropEvent((event) => {
   *  if (event.payload.type === 'over') {
   *    console.log('User hovering', event.payload.position);
   *  } else if (event.payload.type === 'drop') {
   *    console.log('User dropped', event.payload.paths);
   *  } else {
   *    console.log('File drop cancelled');
   *  }
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onDragDropEvent(
    handler: EventCallback<DragDropEvent>
  ): Promise<UnlistenFn> {
    type DragPayload = { paths: string[]; position: PhysicalPosition }

    const unlistenDrag = await this.listen<DragPayload>(
      TauriEvent.DRAG_ENTER,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'enter',
            paths: event.payload.paths,
            position: new PhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenDragOver = await this.listen<DragPayload>(
      TauriEvent.DRAG_OVER,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'over',
            position: new PhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenDrop = await this.listen<DragPayload>(
      TauriEvent.DRAG_DROP,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'drop',
            paths: event.payload.paths,
            position: new PhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenCancel = await this.listen<null>(
      TauriEvent.DRAG_LEAVE,
      (event) => {
        handler({ ...event, payload: { type: 'leave' } })
      }
    )

    return () => {
      unlistenDrag()
      unlistenDrop()
      unlistenDragOver()
      unlistenCancel()
    }
  }

  /**
   * Listen to window focus change.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * const unlisten = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
   *  console.log('Focus changed, window is focused? ' + focused);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onFocusChanged(handler: EventCallback<boolean>): Promise<UnlistenFn> {
    const unlistenFocus = await this.listen<PhysicalPosition>(
      TauriEvent.WINDOW_FOCUS,
      (event) => {
        handler({ ...event, payload: true })
      }
    )
    const unlistenBlur = await this.listen<PhysicalPosition>(
      TauriEvent.WINDOW_BLUR,
      (event) => {
        handler({ ...event, payload: false })
      }
    )
    return () => {
      unlistenFocus()
      unlistenBlur()
    }
  }

  /**
   * Listen to window scale change. Emitted when the window's scale factor has changed.
   * The following user actions can cause DPI changes:
   * - Changing the display's resolution.
   * - Changing the display's scale factor (e.g. in Control Panel on Windows).
   * - Moving the window to a display with a different scale factor.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * const unlisten = await getCurrentWindow().onScaleChanged(({ payload }) => {
   *  console.log('Scale changed', payload.scaleFactor, payload.size);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onScaleChanged(
    handler: EventCallback<ScaleFactorChanged>
  ): Promise<UnlistenFn> {
    return this.listen<ScaleFactorChanged>(
      TauriEvent.WINDOW_SCALE_FACTOR_CHANGED,
      handler
    )
  }

  /**
   * Listen to the system theme change.
   *
   * @example
   * ```typescript
   * import { getCurrentWindow } from "@tauri-apps/api/window";
   * const unlisten = await getCurrentWindow().onThemeChanged(({ payload: theme }) => {
   *  console.log('New theme: ' + theme);
   * });
   *
   * // call unlisten when your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   *
   * @remarks The listener is removed automatically when this window is destroyed,
   * so you do not need to unlisten just because the window is closing. Do call the
   * returned function when the listener's own scope ends, e.g. on page navigation
   * or when a component unmounts.
   */
  async onThemeChanged(handler: EventCallback<Theme>): Promise<UnlistenFn> {
    return this.listen<Theme>(TauriEvent.WINDOW_THEME_CHANGED, handler)
  }
}

/**
 * An RGBA color. Each value has minimum of 0 and maximum of 255.
 *
 * It can be either a string `#ffffff`, an array of 3 or 4 elements or an object.
 *
 * @since 2.0.0
 */
type Color =
  | [number, number, number]
  | [number, number, number, number]
  | { red: number; green: number; blue: number; alpha: number }
  | string

/**
 * Background throttling policy
 *
 * @since 2.0.0
 */
enum BackgroundThrottlingPolicy {
  Disabled = 'disabled',
  Throttle = 'throttle',
  Suspend = 'suspend'
}

/**
 * The scrollbar style to use in the webview.
 *
 * ## Platform-specific
 *
 * **Windows**: This option must be given the same value for all webviews.
 *
 * @since 2.8.0
 */
enum ScrollBarStyle {
  /**
   * The default scrollbar style for the webview.
   */
  Default = 'default',
  /**
   * Fluent UI style overlay scrollbars. **Windows Only**
   *
   * Requires WebView2 Runtime version 125.0.2535.41 or higher, does nothing on older versions,
   * see https://learn.microsoft.com/en-us/microsoft-edge/webview2/release-notes/?tabs=dotnetcsharp#10253541
   */
  FluentOverlay = 'fluentOverlay'
}

/**
 * Platform-specific window effects
 *
 * @since 2.0.0
 */
enum Effect {
  /**
   * A default material appropriate for the view's effectiveAppearance.  **macOS 10.14-**
   *
   * @deprecated since macOS 10.14. You should instead choose an appropriate semantic material.
   */
  AppearanceBased = 'appearanceBased',
  /**
   *  **macOS 10.14-**
   *
   * @deprecated since macOS 10.14. Use a semantic material instead.
   */
  Light = 'light',
  /**
   *  **macOS 10.14-**
   *
   * @deprecated since macOS 10.14. Use a semantic material instead.
   */
  Dark = 'dark',
  /**
   *  **macOS 10.14-**
   *
   * @deprecated since macOS 10.14. Use a semantic material instead.
   */
  MediumLight = 'mediumLight',
  /**
   *  **macOS 10.14-**
   *
   * @deprecated since macOS 10.14. Use a semantic material instead.
   */
  UltraDark = 'ultraDark',
  /**
   *  **macOS 10.10+**
   */
  Titlebar = 'titlebar',
  /**
   *  **macOS 10.10+**
   */
  Selection = 'selection',
  /**
   *  **macOS 10.11+**
   */
  Menu = 'menu',
  /**
   *  **macOS 10.11+**
   */
  Popover = 'popover',
  /**
   *  **macOS 10.11+**
   */
  Sidebar = 'sidebar',
  /**
   *  **macOS 10.14+**
   */
  HeaderView = 'headerView',
  /**
   *  **macOS 10.14+**
   */
  Sheet = 'sheet',
  /**
   *  **macOS 10.14+**
   */
  WindowBackground = 'windowBackground',
  /**
   *  **macOS 10.14+**
   */
  HudWindow = 'hudWindow',
  /**
   *  **macOS 10.14+**
   */
  FullScreenUI = 'fullScreenUI',
  /**
   *  **macOS 10.14+**
   */
  Tooltip = 'tooltip',
  /**
   *  **macOS 10.14+**
   */
  ContentBackground = 'contentBackground',
  /**
   *  **macOS 10.14+**
   */
  UnderWindowBackground = 'underWindowBackground',
  /**
   *  **macOS 10.14+**
   */
  UnderPageBackground = 'underPageBackground',
  /**
   *  **Windows 11 Only**
   */
  Mica = 'mica',
  /**
   * **Windows 7/10/11(22H1) Only**
   *
   * #### Notes
   *
   * This effect has bad performance when resizing/dragging the window on Windows 11 build 22621.
   */
  Blur = 'blur',
  /**
   * **Windows 10/11**
   *
   * #### Notes
   *
   * This effect has bad performance when resizing/dragging the window on Windows 10 v1903+ and Windows 11 build 22000.
   */
  Acrylic = 'acrylic',
  /**
   * Tabbed effect that matches the system dark preference **Windows 11 Only**
   */
  Tabbed = 'tabbed',
  /**
   * Tabbed effect with dark mode but only if dark mode is enabled on the system **Windows 11 Only**
   */
  TabbedDark = 'tabbedDark',
  /**
   * Tabbed effect with light mode **Windows 11 Only**
   */
  TabbedLight = 'tabbedLight'
}

/**
 * Window effect state **macOS only**
 *
 * @see https://developer.apple.com/documentation/appkit/nsvisualeffectview/state
 *
 * @since 2.0.0
 */
enum EffectState {
  /**
   *  Make window effect state follow the window's active state **macOS only**
   */
  FollowsWindowActiveState = 'followsWindowActiveState',
  /**
   *  Make window effect state always active **macOS only**
   */
  Active = 'active',
  /**
   *  Make window effect state always inactive **macOS only**
   */
  Inactive = 'inactive'
}

/** The window effects configuration object
 *
 * @since 2.0.0
 */
interface Effects {
  /**
   *  List of Window effects to apply to the Window.
   * Conflicting effects will apply the first one and ignore the rest.
   */
  effects: Effect[]
  /**
   * Window effect state **macOS Only**
   */
  state?: EffectState
  /**
   * Window effect corner radius **macOS Only**
   */
  radius?: number
  /**
   *  Window effect color. Affects {@link Effect.Blur} and {@link Effect.Acrylic} only
   * on Windows 10 v1903+. Doesn't have any effect on Windows 7 or Windows 11.
   */
  color?: Color
}

/**
 * An extra margin kept free around a window when
 * {@link WindowOptions.preventOverflow} limits it to the working area.
 *
 * Values are in logical pixels.
 *
 * @since 2.5.0
 */
interface PreventOverflowMargin {
  /** The horizontal margin, in logical pixels. */
  width: number
  /** The vertical margin, in logical pixels. */
  height: number
}

/**
 * Configuration for the window to create.
 *
 * @since 1.0.0
 */
interface WindowOptions {
  /** Show window in the center of the screen.. */
  center?: boolean
  /**
   * The initial horizontal position in logical pixels, measured from the left edge
   * of the screen. Only applies if `y` is also set.
   */
  x?: number
  /**
   * The initial vertical position in logical pixels, measured from the top edge of
   * the screen. Only applies if `x` is also set.
   */
  y?: number
  /** The initial width in logical pixels. */
  width?: number
  /** The initial height in logical pixels. */
  height?: number
  /** The minimum width in logical pixels. Only applies if `minHeight` is also set. */
  minWidth?: number
  /** The minimum height in logical pixels. Only applies if `minWidth` is also set. */
  minHeight?: number
  /** The maximum width in logical pixels. Only applies if `maxHeight` is also set. */
  maxWidth?: number
  /** The maximum height in logical pixels. Only applies if `maxWidth` is also set. */
  maxHeight?: number
  /**
   * Prevent the window from overflowing the working area (e.g. monitor size - taskbar size)
   * on creation, which means the window size will be limited to `monitor size - taskbar size`
   *
   * Can either be set to `true` or to a {@link PreventOverflowMargin} object to set an additional margin
   * that should be considered to determine the working area
   * (in this case the window size will be limited to `monitor size - taskbar size - margin`)
   *
   * **NOTE**: The overflow check is only performed on window creation, resizes can still overflow
   *
   * #### Platform-specific
   *
   * - **iOS / Android:** Unsupported.
   */
  preventOverflow?: boolean | PreventOverflowMargin
  /** Whether the window is resizable or not. */
  resizable?: boolean
  /** Window title. */
  title?: string
  /** Whether the window is in fullscreen mode or not. */
  fullscreen?: boolean
  /** Whether the window will be initially focused or not. */
  focus?: boolean
  /** Whether the window can be focused or not. */
  focusable?: boolean
  /**
   * Whether the window is transparent or not.
   * Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri.conf.json > app > macOSPrivateApi`.
   * WARNING: Using private APIs on `macOS` prevents your application from being accepted to the `App Store`.
   *
   * On Windows, using `noRedirectionBitmap` can help avoid a white flash when creating a transparent window.
   */
  transparent?: boolean
  /** Whether the window should be maximized upon creation or not. */
  maximized?: boolean
  /** Whether the window should be immediately visible upon creation or not. */
  visible?: boolean
  /** Whether the window should have borders and bars or not. */
  decorations?: boolean
  /** Whether the window should always be on top of other windows or not. */
  alwaysOnTop?: boolean
  /** Whether the window should always be below other windows. */
  alwaysOnBottom?: boolean
  /** Prevents the window contents from being captured by other apps. */
  contentProtected?: boolean
  /** Whether or not the window icon should be added to the taskbar. */
  skipTaskbar?: boolean
  /**
   * This sets `WS_EX_NOREDIRECTIONBITMAP`.
   *
   * This can avoid the white flash that may appear before the webview content is rendered
   * when using a transparent window. **Windows only**.
   */
  noRedirectionBitmap?: boolean
  /**
   *  Whether or not the window has shadow.
   *
   * #### Platform-specific
   *
   * - **Windows:**
   *   - `false` has no effect on decorated window, shadows are always ON.
   *   - `true` will make undecorated window have a 1px white border,
   * and on Windows 11, it will have a rounded corners.
   * - **Linux:** Unsupported.
   *
   * @since 2.0.0
   */
  shadow?: boolean
  /**
   * The initial window theme. Defaults to the system theme.
   *
   * Only implemented on Windows and macOS 10.14+.
   */
  theme?: Theme
  /**
   * The style of the macOS title bar.
   */
  titleBarStyle?: TitleBarStyle
  /**
   * The position of the window controls on macOS.
   *
   * Requires `titleBarStyle: 'overlay'` and `decorations: true`.
   *
   * @since 2.4.0
   */
  trafficLightPosition?: LogicalPosition
  /**
   * If `true`, sets the window title to be hidden on macOS.
   */
  hiddenTitle?: boolean
  /**
   * Defines the window [tabbing identifier](https://developer.apple.com/documentation/appkit/nswindow/1644704-tabbingidentifier) on macOS.
   *
   * Windows with the same tabbing identifier will be grouped together.
   * If the tabbing identifier is not set, automatic tabbing will be disabled.
   */
  tabbingIdentifier?: string
  /**
   * Whether the window's native maximize button is enabled or not. Defaults to `true`.
   */
  maximizable?: boolean
  /**
   * Whether the window's native minimize button is enabled or not. Defaults to `true`.
   */
  minimizable?: boolean
  /**
   * Whether the window's native close button is enabled or not. Defaults to `true`.
   */
  closable?: boolean
  /**
   * Sets a parent to the window to be created. Can be either a {@linkcode Window} or a label of the window.
   *
   * #### Platform-specific
   *
   * - **Windows**: This sets the passed parent as an owner window to the window to be created.
   *   From [MSDN owned windows docs](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows):
   *     - An owned window is always above its owner in the z-order.
   *     - The system automatically destroys an owned window when its owner is destroyed.
   *     - An owned window is hidden when its owner is minimized.
   * - **Linux**: This makes the new window transient for parent, see <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
   * - **macOS**: This adds the window as a child of parent, see <https://developer.apple.com/documentation/appkit/nswindow/1419152-addchildwindow?language=objc>
   */
  parent?: Window | WebviewWindow | string
  /** Whether the window should be visible on all workspaces or virtual desktops.
   *
   * #### Platform-specific
   *
   * - **Windows / iOS / Android:** Unsupported.
   *
   * @since 2.0.0
   */
  visibleOnAllWorkspaces?: boolean
  /**
   * Window effects.
   *
   * Requires the window to be transparent.
   *
   * #### Platform-specific:
   *
   * - **Windows**: If using decorations or shadows, you may want to try this workaround <https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891>
   * - **Linux**: Unsupported
   */
  windowEffects?: Effects
  /**
   * Set the window background color.
   *
   * #### Platform-specific:
   *
   * - **Android / iOS:** Unsupported.
   * - **Windows**: alpha channel is ignored.
   *
   * @since 2.1.0
   */
  backgroundColor?: Color

  /** Change the default background throttling behaviour.
   *
   * ## Platform-specific
   *
   * - **Linux / Windows / Android**: Unsupported. Workarounds like a pending WebLock transaction might suffice.
   * - **iOS**: Supported since version 17.0+.
   * - **macOS**: Supported since version 14.0+.
   *
   * see https://github.com/tauri-apps/tauri/issues/5250#issuecomment-2569380578
   *
   * @since 2.3.0
   */
  backgroundThrottling?: BackgroundThrottlingPolicy
  /**
   * Whether we should disable JavaScript code execution on the webview or not.
   */
  javascriptDisabled?: boolean
  /**
   * on macOS and iOS there is a link preview on long pressing links, this is enabled by default.
   * see https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKWebView.html#method.allowsLinkPreview
   */
  allowLinkPreview?: boolean
  /**
   * Allows disabling the input accessory view on iOS.
   *
   * The accessory view is the view that appears above the keyboard when a text input element is focused.
   * It usually displays a view with "Done", "Next" buttons.
   */
  disableInputAccessoryView?: boolean
  /**
   * Specifies the native scrollbar style to use with the webview.
   * CSS styles that modify the scrollbar are applied on top of the native appearance configured here.
   *
   * Defaults to `default`, which is the browser default.
   *
   * ## Platform-specific
   *
   * - **Windows**:
   *   - `fluentOverlay` requires WebView2 Runtime version 125.0.2535.41 or higher, and does nothing
   *     on older versions.
   *   - This option must be given the same value for all webviews.
   * - **Linux / Android / iOS / macOS**: Unsupported. Only supports `Default` and performs no operation.
   */
  scrollBarStyle?: ScrollBarStyle
  /**
   * The name of the Android activity to create for this window.
   */
  activityName?: string
  /**
   * The name of the Android activity that is creating this webview window.
   *
   * This is important to determine which stack the activity will belong to.
   */
  createdByActivityName?: string
  /**
   * Sets the identifier of the UIScene that is requesting the creation of this new scene,
   * establishing a relationship between the two scenes.
   *
   * By default the system uses the foreground scene.
   */
  requestedBySceneIdentifier?: string
}

function mapMonitor(m: Monitor | null): Monitor | null {
  return m === null
    ? null
    : {
        name: m.name,
        scaleFactor: m.scaleFactor,
        position: new PhysicalPosition(m.position),
        size: new PhysicalSize(m.size),
        workArea: {
          position: new PhysicalPosition(m.workArea.position),
          size: new PhysicalSize(m.workArea.size)
        }
      }
}

/**
 * Returns the monitor on which the window currently resides.
 * Returns `null` if current monitor can't be detected.
 * @example
 * ```typescript
 * import { currentMonitor } from '@tauri-apps/api/window';
 * const monitor = await currentMonitor();
 * ```
 *
 * @since 1.0.0
 */
async function currentMonitor(): Promise<Monitor | null> {
  return invoke<Monitor | null>('plugin:window|current_monitor').then(
    mapMonitor
  )
}

/**
 * Returns the primary monitor of the system.
 * Returns `null` if it can't identify any monitor as a primary one.
 * @example
 * ```typescript
 * import { primaryMonitor } from '@tauri-apps/api/window';
 * const monitor = await primaryMonitor();
 * ```
 *
 * @since 1.0.0
 */
async function primaryMonitor(): Promise<Monitor | null> {
  return invoke<Monitor | null>('plugin:window|primary_monitor').then(
    mapMonitor
  )
}

/**
 * Returns the monitor that contains the given point. Returns `null` if can't find any.
 * @example
 * ```typescript
 * import { monitorFromPoint } from '@tauri-apps/api/window';
 * const monitor = await monitorFromPoint(100.0, 200.0);
 * ```
 *
 * @since 1.0.0
 */
async function monitorFromPoint(x: number, y: number): Promise<Monitor | null> {
  return invoke<Monitor | null>('plugin:window|monitor_from_point', {
    x,
    y
  }).then(mapMonitor)
}

/**
 * Returns the list of all the monitors available on the system.
 * @example
 * ```typescript
 * import { availableMonitors } from '@tauri-apps/api/window';
 * const monitors = await availableMonitors();
 * ```
 *
 * @since 1.0.0
 */
async function availableMonitors(): Promise<Monitor[]> {
  return invoke<Monitor[]>('plugin:window|available_monitors').then(
    (ms) => ms.map(mapMonitor) as Monitor[]
  )
}

/**
 * Get the cursor position relative to the top-left hand corner of the desktop.
 *
 * Note that the top-left hand corner of the desktop is not necessarily the same as the screen.
 * If the user uses a desktop with multiple monitors,
 * the top-left hand corner of the desktop is the top-left hand corner of the main monitor on Windows and macOS
 * or the top-left of the leftmost monitor on X11.
 *
 * The coordinates can be negative if the top-left hand corner of the window is outside of the visible screen region.
 *
 * @example
 * ```typescript
 * import { cursorPosition } from '@tauri-apps/api/window';
 * const position = await cursorPosition();
 * console.log(position.x, position.y);
 * ```
 *
 * @returns The cursor position, in physical pixels.
 *
 * @remarks Uses the `core:window:allow-cursor-position` permission, which is part
 * of `core:window:default`.
 *
 * @since 2.0.0
 */
async function cursorPosition(): Promise<PhysicalPosition> {
  return invoke<PhysicalPosition>('plugin:window|cursor_position').then(
    (v) => new PhysicalPosition(v)
  )
}

export {
  Window,
  CloseRequestedEvent,
  getCurrentWindow,
  getAllWindows,
  LogicalSize,
  PhysicalSize,
  LogicalPosition,
  PhysicalPosition,
  UserAttentionType,
  Effect,
  EffectState,
  currentMonitor,
  monitorFromPoint,
  primaryMonitor,
  availableMonitors,
  cursorPosition
}

export type {
  Effects,
  Theme,
  TitleBarStyle,
  ScaleFactorChanged,
  WindowOptions,
  PreventOverflowMargin,
  Color,
  BackgroundThrottlingPolicy,
  DragDropEvent,
  ScrollBarStyle
}
