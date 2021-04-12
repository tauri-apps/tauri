// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { EventCallback, UnlistenFn, emit, listen, once } from './event'

interface WindowDef {
  label: string
}

declare global {
  interface Window {
    __TAURI__: {
      __windows: WindowDef[]
      __currentWindow: WindowDef
    }
  }
}

function getCurrent(): WebviewWindowHandle {
  return new WebviewWindowHandle(window.__TAURI__.__currentWindow.label)
}

function getAll(): WindowDef[] {
  return window.__TAURI__.__windows
}

// events that are emitted right here instead of by the created webview
const localTauriEvents = ['tauri://created', 'tauri://error']

class WebviewWindowHandle {
  label: string
  listeners: { [key: string]: Array<EventCallback<any>> }

  constructor(label: string) {
    this.label = label
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.listeners = Object.create(null)
  }

  /**
   * Listen to an event emitted by the webview
   *
   * @param {string} event Event name
   * @param {EventCallback<T>} handler Event handler callback
   * @returns {Promise<UnlistenFn>} A promise resolving to a function to unlisten to the event.
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
   * Listen to an one-off event emitted by the webview
   *
   * @param {string} event Event name
   * @param {EventCallback<T>} handler Event handler callback
   * @returns {Promise<UnlistenFn>} A promise resolving to a function to unlisten to the event.
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
   * Emits an event to the webview
   *
   * @param {string} event Event name
   * @param {string} [payload] Event payload
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
   * Gets the WebviewWindow handle for the webview associated with the given label
   *
   * @param {string} label The webview window label.
   * @returns {WebviewWindowHandle} The handle to communicate with the webview or null if the webview doesn't exist
   */
  static getByLabel(label: string): WebviewWindowHandle | null {
    if (getAll().some((w) => w.label === label)) {
      return new WebviewWindowHandle(label)
    }
    return null
  }
}

class WindowManager {
  /**
   * Updates the window resizable flag
   *
   * @param {boolean} resizable
   * @returns {Promise<void>}
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
   * Sets the window title
   *
   * @param title The new title
   * @returns {Promise<void>}
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
   * Maximizes the window
   *
   * @returns {Promise<void>}
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
   * Unmaximizes the window
   *
   * @returns {Promise<void>}
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
   * Minimizes the window
   *
   * @returns {Promise<void>}
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
   * Unminimizes the window
   *
   * @returns {Promise<void>}
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
   * Sets the window visibility to true
   *
   * @returns {Promise<void>}
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
   * Sets the window visibility to false
   *
   * @returns {Promise<void>}
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
   * Closes the window
   *
   * @returns {Promise<void>}
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
   * Whether the window should have borders and bars
   *
   * @param {boolean} decorations Whether the window should have borders and bars
   * @returns {Promise<void>}
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
   * Whether the window should always be on top of other windows
   *
   * @param {boolean} alwaysOnTop Whether the window should always be on top of other windows or not
   * @returns {Promise<void>}
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
   * Sets the window width
   *
   * @param {number} width The new window width
   * @returns {Promise<void>}
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
   * Sets the window height
   *
   * @param {number} height The new window height
   * @returns {Promise<void>}
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
   * Resizes the window
   *
   * @param {number} width The new window width
   * @param {number} height The new window height
   * @returns {Promise<void>}
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
   * Sets the window min size
   *
   * @param {number} minWidth The new window min width
   * @param {number} minHeight The new window min height
   * @returns {Promise<void>}
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
   * Sets the window max size
   *
   * @param {number} maxWidth The new window max width
   * @param {number} maxHeight The new window max height
   * @returns {Promise<void>}
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
   * Sets the window x position
   *
   * @param {number} x The new window x position
   * @returns {Promise<void>}
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
   * Sets the window y position
   *
   * @param {number} y The new window y position
   * @returns {Promise<void>}
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
   * Sets the window position
   *
   * @param {number} x The new window x position
   * @param {number} y The new window y position
   * @returns {Promise<void>}
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
   * Sets the window fullscreen state
   *
   * @param {boolean} fullscreen Whether the window should go to fullscreen or not
   * @returns {Promise<void>}
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
   * Sets the window icon
   *
   * @param {string | number[]} icon Icon bytes or path to the icon file
   * @returns {Promise<void>}
   */
  async setIcon(icon: 'string' | number[]): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'setIcon',
        icon
      }
    })
  }
}

const appWindow = new WindowManager()

export interface WindowOptions {
  url?: 'app' | string
  x?: number
  y?: number
  width?: number
  height?: number
  minWidth?: number
  minHeight?: number
  maxWidth?: number
  maxHeight?: number
  resizable?: boolean
  title?: string
  fullscreen?: boolean
  maximized?: boolean
  visible?: boolean
  decorations?: boolean
  alwaysOnTop?: boolean
}

export { WebviewWindow, getCurrent, getAll, appWindow }
