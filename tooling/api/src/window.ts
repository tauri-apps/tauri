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
  constructor(label: string, options: WindowOptions = {}) {
    super(label)
    invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'createWebview',
        options: {
          label,
          ...options
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
        resizable
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
        title
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
        decorations
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
        alwaysOnTop
      }
    })
  }

  /**
   * Sets the window width.
   *
   * @param width The new window width.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setWidth(width: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setWidth',
        width
      }
    })
  }

  /**
   * Sets the window height.
   *
   * @param height The new window height.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setHeight(height: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setHeight',
        height
      }
    })
  }

  /**
   * Resizes the window.
   *
   * @param width The new window width.
   * @param height The new window height.
   * @returns A promise indicating the success or failure of the operation.
   */
  async resize(width: number, height: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'resize',
        width,
        height
      }
    })
  }

  /**
   * Sets the window min size.
   *
   * @param minWidth The new window min width.
   * @param minHeight The new window min height.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMinSize(minWidth: number, minHeight: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setMinSize',
        minWidth,
        minHeight
      }
    })
  }

  /**
   * Sets the window max size.
   *
   * @param maxWidth The new window max width.
   * @param maxHeight The new window max height.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMaxSize(maxWidth: number, maxHeight: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setMaxSize',
        maxWidth,
        maxHeight
      }
    })
  }

  /**
   * Sets the window x position.
   *
   * @param x The new window x position.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setX(x: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setX',
        x
      }
    })
  }

  /**
   * Sets the window y position.
   *
   * @param y The new window y position.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setY(y: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setY',
        y
      }
    })
  }

  /**
   * Sets the window position.
   *
   * @param x The new window x position.
   * @param y The new window y position.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setPosition(x: number, y: number): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setPosition',
        x,
        y
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
        fullscreen
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
        icon
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
  /** Whether the window should be maximized upon creation or not. */
  maximized?: boolean
  /** Whether the window should be immediately visible upon creation or not. */
  visible?: boolean
  /** Whether the window should have borders and bars or not. */
  decorations?: boolean
  /** Whether the window should always be on top of other windows or not. */
  alwaysOnTop?: boolean
}

export { WebviewWindow, getCurrent, getAll, appWindow }
