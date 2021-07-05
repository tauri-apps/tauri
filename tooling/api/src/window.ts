// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides APIs to create windows, communicate with other windows and manipulate the current window.
 *
 * This package is also accessible with `window.__TAURI__.window` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "window": {
 *         "all": true, // enable all window APIs
 *         "create": true // enable window creation
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * # Window events
 *
 * Events can be listened using `appWindow.listen`:
 * ```typescript
 * import { appWindow } from '@tauri-apps/api/window'
 * appWindow.listen('tauri://move', ({ event, payload }) => {
 *   const { x, y } = payload // payload here is a `PhysicalPosition`
 * })
 * ```
 *
 * Window-specific events emitted by the backend:
 *
 * #### 'tauri://resize'
 * Emitted when the size of the window has changed.
 * *EventPayload*:
 * ```typescript
 * type ResizePayload = PhysicalSize
 * ```
 *
 * #### 'tauri://move'
 * Emitted when the position of the window has changed.
 * *EventPayload*:
 * ```typescript
 * type MovePayload = PhysicalPosition
 * ```
 *
 * #### 'tauri://close-requested'
 * Emitted when the user requests the window to be closed.
 *
 * #### 'tauri://destroyed'
 * Emitted after the window is closed.
 *
 * #### 'tauri://focus'
 * Emitted when the window gains focus.
 *
 * #### 'tauri://blur'
 * Emitted when the window loses focus.
 *
 * #### 'tauri://scale-change'
 * Emitted when the window's scale factor has changed.
 * The following user actions can cause DPI changes:
 * - Changing the display's resolution.
 * - Changing the display's scale factor (e.g. in Control Panel on Windows).
 * - Moving the window to a display with a different scale factor.
 * *Event payload*:
 * ```typescript
 * interface ScaleFactorChanged {
 *   scaleFactor: number
 *   size: PhysicalSize
 * }
 * ```
 *
 * #### 'tauri://menu'
 * Emitted when a menu item is clicked.
 * *EventPayload*:
 * ```typescript
 * type MenuClicked = string
 * ```
 *
 * @packageDocumentation
 */

import { invokeTauriCommand } from './helpers/tauri'
import { EventName, EventCallback, UnlistenFn, listen, once } from './event'
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

/** Attention type to request on a window. */
enum UserAttentionType {
  /**
   * ## Platform-specific
   *  - **macOS:** Bounces the dock icon until the application is in focus.
   * - **Windows:** Flashes both the window and the taskbar button until the application is in focus.
   */
  Critical = 1,
  /**
   * ## Platform-specific
   * - **macOS:** Bounces the dock icon once.
   * - **Windows:** Flashes the taskbar button until the application is in focus.
   */
  Informational
}

/**
 * Get an instance of `WebviewWindow` for the current webview window.
 *
 * @return The current WebviewWindow.
 */
function getCurrent(): WebviewWindow {
  // @ts-expect-error
  return new WebviewWindow(window.__TAURI__.__currentWindow.label, {
    skip: true
  })
}

/**
 * Gets an instance of `WebviewWindow` for all available webview windows.
 *
 * @return The list of WebviewWindow.
 */
function getAll(): WebviewWindow[] {
  // @ts-expect-error
  return window.__TAURI__.__windows.map(
    (w) => new WebviewWindow(w, { skip: true })
  )
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
    event: EventName,
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
 * Manage the current window object.
 */
class WindowManager extends WebviewWindowHandle {
  // Getters
  /** The scale factor that can be used to map physical pixels to logical pixels. */
  async scaleFactor(): Promise<number> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'scaleFactor'
          }
        }
      }
    })
  }

  /** The position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop. */
  async innerPosition(): Promise<PhysicalPosition> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'innerPosition'
          }
        }
      }
    })
  }

  /** The position of the top-left hand corner of the window relative to the top-left hand corner of the desktop. */
  async outerPosition(): Promise<PhysicalPosition> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'outerPosition'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'innerSize'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'outerSize'
          }
        }
      }
    })
  }

  /** Gets the window's current fullscreen state. */
  async isFullscreen(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'isFullscreen'
          }
        }
      }
    })
  }

  /** Gets the window's current maximized state. */
  async isMaximized(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'isMaximized'
          }
        }
      }
    })
  }

  /** Gets the window's current decorated state. */
  async isDecorated(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'isDecorated'
          }
        }
      }
    })
  }

  /** Gets the window's current resizable state. */
  async isResizable(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'isResizable'
          }
        }
      }
    })
  }

  /** Gets the window's current visible state. */
  async isVisible(): Promise<boolean> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'isVisible'
          }
        }
      }
    })
  }

  // Setters

  /**
   * Centers the window.
   *
   * @param resizable
   * @returns A promise indicating the success or failure of the operation.
   */
  async center(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'center'
          }
        }
      }
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
   * ## Platform-specific
   *
   * - **macOS:** `null` has no effect.
   *
   * @param resizable
   * @returns A promise indicating the success or failure of the operation.
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
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'requestUserAttention',
            payload: requestType_
          }
        }
      }
    })
  }

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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setResizable',
            payload: resizable
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setTitle',
            payload: title
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'maximize'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'unmaximize'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'minimize'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'unminimize'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'show'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'hide'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'close'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setDecorations',
            payload: decorations
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setAlwaysOnTop',
            payload: alwaysOnTop
          }
        }
      }
    })
  }

  /**
   * Resizes the window.
   * @example
   * ```typescript
   * import { appWindow, LogicalSize } from '@tauri-apps/api/window'
   * await appWindow.setSize(new LogicalSize(600, 500))
   * ```
   *
   * @param size The logical or physical size.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setSize(size: LogicalSize | PhysicalSize): Promise<void> {
    if (!size || (size.type !== 'Logical' && size.type !== 'Physical')) {
      throw new Error(
        'the `size` argument must be either a LogicalSize or a PhysicalSize instance'
      )
    }
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setSize',
            payload: {
              type: size.type,
              data: {
                width: size.width,
                height: size.height
              }
            }
          }
        }
      }
    })
  }

  /**
   * Sets the window min size. If the `size` argument is not provided, the min size is unset.
   * @example
   * ```typescript
   * import { appWindow, PhysicalSize } from '@tauri-apps/api/window'
   * await appWindow.setMinSize(new PhysicalSize(600, 500))
   * ```
   *
   * @param size The logical or physical size.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMinSize(
    size: LogicalSize | PhysicalSize | undefined
  ): Promise<void> {
    if (size && size.type !== 'Logical' && size.type !== 'Physical') {
      throw new Error(
        'the `size` argument must be either a LogicalSize or a PhysicalSize instance'
      )
    }
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setMinSize',
            payload: size
              ? {
                  type: size.type,
                  data: {
                    width: size.width,
                    height: size.height
                  }
                }
              : null
          }
        }
      }
    })
  }

  /**
   * Sets the window max size. If the `size` argument is undefined, the max size is unset.
   * @example
   * ```typescript
   * import { appWindow, LogicalSize } from '@tauri-apps/api/window'
   * await appWindow.setMaxSize(new LogicalSize(600, 500))
   * ```
   *
   * @param size The logical or physical size.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMaxSize(
    size: LogicalSize | PhysicalSize | undefined
  ): Promise<void> {
    if (size && size.type !== 'Logical' && size.type !== 'Physical') {
      throw new Error(
        'the `size` argument must be either a LogicalSize or a PhysicalSize instance'
      )
    }
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setMaxSize',
            payload: size
              ? {
                  type: size.type,
                  data: {
                    width: size.width,
                    height: size.height
                  }
                }
              : null
          }
        }
      }
    })
  }

  /**
   * Sets the window position.
   * @example
   * ```typescript
   * import { appWindow, LogicalPosition } from '@tauri-apps/api/window'
   * await appWindow.setPosition(new LogicalPosition(600, 500))
   * ```
   *
   * @param position The new position, in logical or physical pixels.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setPosition(
    position: LogicalPosition | PhysicalPosition
  ): Promise<void> {
    if (
      !position ||
      (position.type !== 'Logical' && position.type !== 'Physical')
    ) {
      throw new Error(
        'the `position` argument must be either a LogicalPosition or a PhysicalPosition instance'
      )
    }
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setPosition',
            payload: {
              type: position.type,
              data: {
                x: position.x,
                y: position.y
              }
            }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setFullscreen',
            payload: fullscreen
          }
        }
      }
    })
  }

  /**
   * Bring the window to front and focus.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async setFocus(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setFocus'
          }
        }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setIcon',
            payload: {
              icon
            }
          }
        }
      }
    })
  }

  /**
   * Whether to show the window icon in the task bar or not.
   *
   * @param skip true to hide window icon, false to show it.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setSkipTaskbar(skip: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setSkipTaskbar',
            payload: skip
          }
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
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'startDragging'
          }
        }
      }
    })
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
class WebviewWindow extends WindowManager {
  constructor(label: string, options: WindowOptions = {}) {
    super(label)
    // @ts-expect-error
    if (!options?.skip) {
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
  }

  /**
   * Gets the WebviewWindow for the webview associated with the given label.
   *
   * @param label The webview window label.
   * @returns The WebviewWindow instance to communicate with the webview or null if the webview doesn't exist.
   */
  static getByLabel(label: string): WebviewWindow | null {
    if (getAll().some((w) => w.label === label)) {
      // @ts-expect-error
      return new WebviewWindow(label, { skip: true })
    }
    return null
  }
}

/** The WebviewWindow for the current window. */
// @ts-expect-error
const appWindow = new WebviewWindow()

/** Configuration for the window to create. */
interface WindowOptions {
  /**
   * Remote URL or local file path to open, e.g. `https://github.com/tauri-apps` or `path/to/page.html`.
   */
  url?: string
  /** Show window in the center of the screen.. */
  center?: boolean
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
  /** Whether the window will be initially hidden or focused. */
  focus?: boolean
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
  /** Whether or not the window icon should be added to the taskbar. */
  skipTaskbar?: boolean
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
  WindowManager,
  getCurrent,
  getAll,
  appWindow,
  LogicalSize,
  PhysicalSize,
  LogicalPosition,
  PhysicalPosition,
  UserAttentionType,
  currentMonitor,
  primaryMonitor,
  availableMonitors
}

export type { Monitor, WindowOptions }
