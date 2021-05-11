// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides APIs to create windows, communicate with other windows and manipulate the current window.
 * @packageDocumentation
 */

import { invokeTauriCommand } from './helpers/tauri'
import { EventCallback, UnlistenFn, listen, once } from './event'
import { emit } from './helpers/event'

/** Allows you to retrieve information about a given monitor. */
interface Monitor {
  /** Human-readable name of the monitor */
  name: string | null
  /** The monitor's resolution. */
  size: PhysicalSize
  /** the Top-left corner position of the monitor relative to the larger full screen area. */
  position: PhysicalPosition
  /** The scale factor that can be used to map physical pixels to logical pixels. */
  scaleFactor: number
}

/** A size represented in logical pixels. */
class LogicalSize {
  type = 'Logical'
  width: number
  height: number

  constructor(width: number, height: number) {
    this.width = width
    this.height = height
  }
}

/** A size represented in physical pixels. */
class PhysicalSize {
  type = 'Physical'
  width: number
  height: number

  constructor(width: number, height: number) {
    this.width = width
    this.height = height
  }

  /** Converts the physical size to a logical one. */
  toLogical(scaleFactor: number): LogicalSize {
    return new LogicalSize(this.width / scaleFactor, this.height / scaleFactor)
  }
}

/** A position represented in logical pixels. */
class LogicalPosition {
  type = 'Logical'
  x: number
  y: number

  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }
}

/** A position represented in physical pixels. */
class PhysicalPosition {
  type = 'Physical'
  x: number
  y: number

  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }

  /** Converts the physical position to a logical one. */
  toLogical(scaleFactor: number): LogicalPosition {
    return new LogicalPosition(this.x / scaleFactor, this.y / scaleFactor)
  }
}

/** @ignore */
interface WindowDef {
  label: string
}

/** @ignore */
declare global {
  interface Window {
    __TAURI__: {
      __windows: WindowDef[]
      __currentWindow: WindowDef
    }
  }
}

/**
 * Get a handle to the current webview window. Allows emitting and listening to events from the backend that are tied to the window.
 *
 * @return The current window handle.
 */
function getCurrent(): WebviewWindowHandle {
  return new WebviewWindowHandle(window.__TAURI__.__currentWindow.label)
}

/**
 * Gets metadata for all available webview windows.
 *
 * @return The list of webview handles.
 */
function getAll(): WindowDef[] {
  return window.__TAURI__.__windows
}

/** @ignore */
// events that are emitted right here instead of by the created webview
const localTauriEvents = ['tauri://created', 'tauri://error']

/**
 * A webview window handle allows emitting and listening to events from the backend that are tied to the window.
 */
class WebviewWindowHandle {
  /** Window label. */
  label: string
  /** Local event listeners. */
  listeners: { [key: string]: Array<EventCallback<any>> }

  constructor(label: string) {
    this.label = label
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.listeners = Object.create(null)
  }

  /**
   * Listen to an event emitted by the backend that is tied to the webview window.
   *
   * @param event Event name.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   */
  async listen<T>(
    event: string,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return Promise.resolve(() => {
        // eslint-disable-next-line security/detect-object-injection
        const listeners = this.listeners[event]
        listeners.splice(listeners.indexOf(handler), 1)
      })
    }
    return listen(event, handler)
  }

  /**
   * Listen to an one-off event emitted by the backend that is tied to the webview window.
   *
   * @param event Event name.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   */
  async once<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return Promise.resolve(() => {
        // eslint-disable-next-line security/detect-object-injection
        const listeners = this.listeners[event]
        listeners.splice(listeners.indexOf(handler), 1)
      })
    }
    return once(event, handler)
  }

  /**
   * Emits an event to the backend, tied to the webview window.
   *
   * @param event Event name.
   * @param payload Event payload.
   */
  async emit(event: string, payload?: string): Promise<void> {
    if (localTauriEvents.includes(event)) {
      // eslint-disable-next-line
      for (const handler of this.listeners[event] || []) {
        handler({ event, id: -1, payload })
      }
      return Promise.resolve()
    }
    return emit(event, this.label, payload)
  }

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
}

/**
 * Create new webview windows and get a handle to existing ones.
 * @example
 * ```typescript
 * // loading embedded asset:
 * const webview = new WebviewWindow('theUniqueLabel', {
 *   url: 'path/to/page.html'
 * })
 * // alternatively, load a remote URL:
 * const webview = new WebviewWindow('theUniqueLabel', {
 *   url: 'https://github.com/tauri-apps/tauri'
 * })
 *
 * webview.once('tauri://created', function () {
 *  // webview window successfully created
 * })
 * webview.once('tauri://error', function (e) {
 *  // an error happened creating the webview window
 * })
 *
 * // emit an event to the backend
 * await webview.emit("some event", "data")
 * // listen to an event from the backend
 * const unlisten = await webview.listen("event name", e => {})
 * unlisten()
 * ```
 */
class WebviewWindow extends WebviewWindowHandle {
  private constructor(label: string, options: WindowOptions = {}) {
    super(label)
    invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'createWebview',
        data: {
          options: {
            label,
            ...options
          }
        }
      }
    })
      .then(async () => this.emit('tauri://created'))
      .catch(async (e) => this.emit('tauri://error', e))
  }

  /**
   * Gets the WebviewWindow handle for the webview associated with the given label.
   *
   * @param label The webview window label.
   * @returns The handle to communicate with the webview or null if the webview doesn't exist.
   */
  static getByLabel(label: string): WebviewWindowHandle | null {
    if (getAll().some((w) => w.label === label)) {
      return new WebviewWindowHandle(label)
    }
    return null
  }
}

/**
 * Manage the current window object.
 */
export class WindowManager {
  // Getters
  /** The scale factor that can be used to map physical pixels to logical pixels. */
  async scaleFactor(): Promise<number> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'scaleFactor'
      }
    })
  }

  /** The position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop. */
  async innerPosition(): Promise<PhysicalPosition> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'innerPosition'
      }
    })
  }

  /** The position of the top-left hand corner of the window relative to the top-left hand corner of the desktop. */
  async outerPosition(): Promise<PhysicalPosition> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'outerPosition'
      }
    })
  }

  /**
   * The physical size of the window's client area.
   * The client area is the content of the window, excluding the title bar and borders.
   */
  async innerSize(): Promise<PhysicalSize> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'innerSize'
      }
    })
  }

  /**
   * The physical size of the entire window.
   * These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
   */
  async outerSize(): Promise<PhysicalSize> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'outerSize'
      }
    })
  }

  /** Gets the window's current fullscreen state. */
  async isFullscreen(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'isFullscreen'
      }
    })
  }

  /** Gets the window's current maximized state. */
  async isMaximized(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'isMaximized'
      }
    })
  }

  // Setters

  /**
   * Updates the window resizable flag.
   *
   * @param resizable
   * @returns A promise indicating the success or failure of the operation.
   */
  async setResizable(resizable: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setResizable',
        data: resizable
      }
    })
  }

  /**
   * Sets the window title.
   *
   * @param title The new title
   * @returns A promise indicating the success or failure of the operation.
   */
  async setTitle(title: string): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setTitle',
        data: title
      }
    })
  }

  /**
   * Maximizes the window.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async maximize(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'maximize'
      }
    })
  }

  /**
   * Unmaximizes the window.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async unmaximize(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'unmaximize'
      }
    })
  }

  /**
   * Minimizes the window.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async minimize(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'minimize'
      }
    })
  }

  /**
   * Unminimizes the window.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async unminimize(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'unminimize'
      }
    })
  }

  /**
   * Sets the window visibility to true.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async show(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'show'
      }
    })
  }

  /**
   * Sets the window visibility to false.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async hide(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'hide'
      }
    })
  }

  /**
   * Closes the window.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async close(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'close'
      }
    })
  }

  /**
   * Whether the window should have borders and bars.
   *
   * @param decorations Whether the window should have borders and bars.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setDecorations(decorations: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setDecorations',
        data: decorations
      }
    })
  }

  /**
   * Whether the window should always be on top of other windows.
   *
   * @param alwaysOnTop Whether the window should always be on top of other windows or not.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setAlwaysOnTop',
        data: alwaysOnTop
      }
    })
  }

  /**
   * Resizes the window.
   *
   * @param size The logical or physical size.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setSize(size: LogicalSize | PhysicalSize): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setSize',
        data: {
          type: size.type,
          data: {
            width: size.width,
            height: size.height
          }
        }
      }
    })
  }

  /**
   * Sets the window min size.
   *
   * @param size The logical or physical size.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMinSize(
    size: LogicalSize | PhysicalSize | undefined
  ): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setMinSize',
        data: size
          ? {
              type: size.type,
              data: {
                width: size.width,
                height: size.height
              }
            }
          : null
      }
    })
  }

  /**
   * Sets the window max size.
   *
   * @param size The logical or physical size.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMaxSize(
    size: LogicalSize | PhysicalSize | undefined
  ): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setMaxSize',
        data: size
          ? {
              type: size.type,
              data: {
                width: size.width,
                height: size.height
              }
            }
          : null
      }
    })
  }

  /**
   * Sets the window position.
   *
   * @param position The new position, in logical or physical pixels.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setPosition(
    position: LogicalPosition | PhysicalPosition
  ): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setPosition',
        data: {
          type: position.type,
          data: {
            x: position.x,
            y: position.y
          }
        }
      }
    })
  }

  /**
   * Sets the window fullscreen state.
   *
   * @param fullscreen Whether the window should go to fullscreen or not.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setFullscreen(fullscreen: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setFullscreen',
        data: fullscreen
      }
    })
  }

  /**
   * Sets the window icon.
   *
   * @param icon Icon bytes or path to the icon file.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setIcon(icon: string | number[]): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setIcon',
        data: {
          icon
        }
      }
    })
  }

  /**
   * Starts dragging the window.
   *
   * @return A promise indicating the success or failure of the operation.
   */
  async startDragging(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'startDragging'
      }
    })
  }
}

/** The manager for the current window. Allows you to manipulate the window object. */
const appWindow = new WindowManager()

/** Configuration for the window to create. */
export interface WindowOptions {
  /**
   * Remote URL or local file path to open, e.g. `https://github.com/tauri-apps` or `path/to/page.html`.
   */
  url?: string
  /** The initial vertical position. Only applies if `y` is also set. */
  x?: number
  /** The initial horizontal position. Only applies if `x` is also set. */
  y?: number
  /** The initial width. */
  width?: number
  /** The initial height. */
  height?: number
  /** The minimum width. Only applies if `minHeight` is also set. */
  minWidth?: number
  /** The minimum height. Only applies if `minWidth` is also set. */
  minHeight?: number
  /** The maximum width. Only applies if `maxHeight` is also set. */
  maxWidth?: number
  /** The maximum height. Only applies if `maxWidth` is also set. */
  maxHeight?: number
  /** Whether the window is resizable or not. */
  resizable?: boolean
  /** Window title. */
  title?: string
  /** Whether the window is in fullscreen mode or not. */
  fullscreen?: boolean
  /** Whether the window is transparent or not. */
  transparent?: boolean
  /** Whether the window should be maximized upon creation or not. */
  maximized?: boolean
  /** Whether the window should be immediately visible upon creation or not. */
  visible?: boolean
  /** Whether the window should have borders and bars or not. */
  decorations?: boolean
  /** Whether the window should always be on top of other windows or not. */
  alwaysOnTop?: boolean
}

/**
 * Returns the monitor on which the window currently resides.
 * Returns `null` if current monitor can't be detected.
 */
async function currentMonitor(): Promise<Monitor | null> {
  return invokeTauriCommand({
    __tauriModule: 'Window',
    message: {
      cmd: 'currentMonitor'
    }
  })
}

/**
 * Returns the primary monitor of the system.
 * Returns `null` if it can't identify any monitor as a primary one.
 */
async function primaryMonitor(): Promise<Monitor | null> {
  return invokeTauriCommand({
    __tauriModule: 'Window',
    message: {
      cmd: 'primaryMonitor'
    }
  })
}

/** Returns the list of all the monitors available on the system. */
async function availableMonitors(): Promise<Monitor[]> {
  return invokeTauriCommand({
    __tauriModule: 'Window',
    message: {
      cmd: 'availableMonitors'
    }
  })
}

export {
  WebviewWindow,
  WebviewWindowHandle,
  getCurrent,
  getAll,
  appWindow,
  Monitor,
  LogicalSize,
  PhysicalSize,
  LogicalPosition,
  PhysicalPosition,
  currentMonitor,
  primaryMonitor,
  availableMonitors
}
