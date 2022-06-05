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
 *         "create": true, // enable window creation
 *         "center": true,
 *         "requestUserAttention": true,
 *         "setResizable": true,
 *         "setTitle": true,
 *         "maximize": true,
 *         "unmaximize": true,
 *         "minimize": true,
 *         "unminimize": true,
 *         "show": true,
 *         "hide": true,
 *         "close": true,
 *         "setDecorations": true,
 *         "setAlwaysOnTop": true,
 *         "setSize": true,
 *         "setMinSize": true,
 *         "setMaxSize": true,
 *         "setPosition": true,
 *         "setFullscreen": true,
 *         "setFocus": true,
 *         "setIcon": true,
 *         "setSkipTaskbar": true,
 *         "setCursorGrab": true,
 *         "setCursorVisible": true,
 *         "setCursorIcon": true,
 *         "setCursorPosition": true,
 *         "startDragging": true,
 *         "print": true
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
 * If a listener is registered for this event, Tauri won't close the window so you must call `appWindow.close()` manually.
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
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import type { EventName, EventCallback, UnlistenFn } from './event'
import { emit, listen, once } from './helpers/event'

type Theme = 'light' | 'dark'

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
    __TAURI_METADATA__: {
      __windows: WindowDef[]
      __currentWindow: WindowDef
    }
  }
}

/** Attention type to request on a window. */
enum UserAttentionType {
  /**
   * #### Platform-specific
   *  - **macOS:** Bounces the dock icon until the application is in focus.
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
 * Get an instance of `WebviewWindow` for the current webview window.
 *
 * @return The current WebviewWindow.
 */
function getCurrent(): WebviewWindow {
  return new WebviewWindow(window.__TAURI_METADATA__.__currentWindow.label, {
    // @ts-expect-error
    skip: true
  })
}

/**
 * Gets an instance of `WebviewWindow` for all available webview windows.
 *
 * @return The list of WebviewWindow.
 */
function getAll(): WebviewWindow[] {
  return window.__TAURI_METADATA__.__windows.map(
    (w) =>
      new WebviewWindow(w.label, {
        // @ts-expect-error
        skip: true
      })
  )
}

/** @ignore */
// events that are emitted right here instead of by the created webview
const localTauriEvents = ['tauri://created', 'tauri://error']
/** @ignore */
export type WindowLabel = string
/**
 * A webview window handle allows emitting and listening to events from the backend that are tied to the window.
 */
class WebviewWindowHandle {
  /** The window label. It is a unique identifier for the window, can be used to reference it later. */
  label: WindowLabel
  /** Local event listeners. */
  listeners: { [key: string]: Array<EventCallback<any>> }

  constructor(label: WindowLabel) {
    this.label = label
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.listeners = Object.create(null)
  }

  /**
   * Listen to an event emitted by the backend that is tied to the webview window.
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   */
  async listen<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return Promise.resolve(() => {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, security/detect-object-injection
        const listeners = this.listeners[event]
        listeners.splice(listeners.indexOf(handler), 1)
      })
    }
    return listen(event, this.label, handler)
  }

  /**
   * Listen to an one-off event emitted by the backend that is tied to the webview window.
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
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
    return once(event, this.label, handler)
  }

  /**
   * Emits an event to the backend, tied to the webview window.
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param payload Event payload.
   */
  async emit(event: string, payload?: unknown): Promise<void> {
    if (localTauriEvents.includes(event)) {
      // eslint-disable-next-line
      for (const handler of this.listeners[event] || []) {
        handler({ event, id: -1, windowLabel: this.label, payload })
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
    return invokeTauriCommand<{ x: number; y: number }>({
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
    }).then(({ x, y }) => new PhysicalPosition(x, y))
  }

  /** The position of the top-left hand corner of the window relative to the top-left hand corner of the desktop. */
  async outerPosition(): Promise<PhysicalPosition> {
    return invokeTauriCommand<{ x: number; y: number }>({
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
    }).then(({ x, y }) => new PhysicalPosition(x, y))
  }

  /**
   * The physical size of the window's client area.
   * The client area is the content of the window, excluding the title bar and borders.
   */
  async innerSize(): Promise<PhysicalSize> {
    return invokeTauriCommand<{ width: number; height: number }>({
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
    }).then(({ width, height }) => new PhysicalSize(width, height))
  }

  /**
   * The physical size of the entire window.
   * These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
   */
  async outerSize(): Promise<PhysicalSize> {
    return invokeTauriCommand<{ width: number; height: number }>({
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
    }).then(({ width, height }) => new PhysicalSize(width, height))
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

  /** Gets the window's current visible state. */
  async theme(): Promise<Theme | null> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'theme'
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
   * #### Platform-specific
   *
   * - **macOS:** `null` has no effect.
   * - **Linux:** Urgency levels have the same effect.
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
   * Toggles the window maximized state.
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async toggleMaximize(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'toggleMaximize'
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
   * Resizes the window with a new inner size.
   * @example
   * ```typescript
   * import { appWindow, LogicalSize } from '@tauri-apps/api/window'
   * await appWindow.setSize(new LogicalSize(600, 500))
   * ```
   *
   * @param size The logical or physical inner size.
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
   * Sets the window minimum inner size. If the `size` argument is not provided, the constraint is unset.
   * @example
   * ```typescript
   * import { appWindow, PhysicalSize } from '@tauri-apps/api/window'
   * await appWindow.setMinSize(new PhysicalSize(600, 500))
   * ```
   *
   * @param size The logical or physical inner size, or `null` to unset the constraint.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMinSize(
    size: LogicalSize | PhysicalSize | null | undefined
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
   * Sets the window maximum inner size. If the `size` argument is undefined, the constraint is unset.
   * @example
   * ```typescript
   * import { appWindow, LogicalSize } from '@tauri-apps/api/window'
   * await appWindow.setMaxSize(new LogicalSize(600, 500))
   * ```
   *
   * @param size The logical or physical inner size, or `null` to unset the constraint.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setMaxSize(
    size: LogicalSize | PhysicalSize | null | undefined
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
   * Sets the window outer position.
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
   * Note that you need the `icon-ico` or `icon-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "icon-png"] }
   * ```
   *
   * @param icon Icon bytes or path to the icon file.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setIcon(icon: string | Uint8Array): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setIcon',
            payload: {
              // correctly serialize Uint8Arrays
              icon: typeof icon === 'string' ? icon : Array.from(icon)
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
   * Grabs the cursor, preventing it from leaving the window.
   *
   * There's no guarantee that the cursor will be hidden. You should
   * hide it by yourself if you want so.
   *
   * #### Platform-specific
   *
   * - **Linux:** Unsupported.
   * - **macOS:** This locks the cursor in a fixed location, which looks visually awkward.
   *
   * @param grab `true` to grab the cursor icon, `false` to release it.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setCursorGrab(grab: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setCursorGrab',
            payload: grab
          }
        }
      }
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
   *
   * @param visible If `false`, this will hide the cursor. If `true`, this will show the cursor.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setCursorVisible(visible: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setCursorVisible',
            payload: visible
          }
        }
      }
    })
  }

  /**
   * Modifies the cursor icon of the window.
   *
   * @param icon The new cursor icon.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setCursorIcon(icon: CursorIcon): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setCursorIcon',
            payload: icon
          }
        }
      }
    })
  }

  /**
   * Changes the position of the cursor in window coordinates.
   *
   * @param position The new cursor position.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setCursorPosition(
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
            type: 'setCursorPosition',
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
 *
 * Windows are identified by a *label*  a unique identifier that can be used to reference it later.
 * It may only contain alphanumeric characters `a-zA-Z` plus the following special characters `-`, `/`, `:` and `_`.
 *
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
  /**
   * Creates a new WebviewWindow.
   * * @param label The unique webview window label. Must be alphanumeric: `a-zA-Z-/:_`.
   * @returns The WebviewWindow instance to communicate with the webview.
   */
  constructor(label: WindowLabel, options: WindowOptions = {}) {
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
        .catch(async (e: string) => this.emit('tauri://error', e))
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
let appWindow: WebviewWindow
if ('__TAURI_METADATA__' in window) {
  appWindow = new WebviewWindow(
    window.__TAURI_METADATA__.__currentWindow.label,
    {
      // @ts-expect-error
      skip: true
    }
  )
} else {
  console.warn(
    `Could not find "window.__TAURI_METADATA__". The "appWindow" value will reference the "main" window label.\nNote that this is not an issue if running this frontend on a browser instead of a Tauri window.`
  )
  appWindow = new WebviewWindow('main', {
    // @ts-expect-error
    skip: true
  })
}

/** Configuration for the window to create. */
interface WindowOptions {
  /**
   * Remote URL or local file path to open.
   *
   * - URL such as `https://github.com/tauri-apps` is opened directly on a Tauri window.
   * - data: URL such as `data:text/html,<html>...` is only supported with the `window-data-url` Cargo feature for the `tauri` dependency.
   * - local file path or route such as `/path/to/page.html` or `/users` is appended to the application URL (the devServer URL on development, or `tauri://localhost/` and `https://tauri.localhost/` on production).
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
  /**
   * Whether the window is transparent or not.
   * Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri.conf.json > tauri > macOSPrivateApi`.
   * WARNING: Using private APIs on `macOS` prevents your application from being accepted to the `App Store`.
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
  /** Whether or not the window icon should be added to the taskbar. */
  skipTaskbar?: boolean
  /**
   * Whether the file drop is enabled or not on the webview. By default it is enabled.
   *
   * Disabling it is required to use drag and drop on the frontend on Windows.
   */
  fileDropEnabled?: boolean
  /**
   *  The initial window theme. Defaults to the system theme.
   *
   * Only implemented on Windows.
   */
  theme?: Theme
}

/**
 * Returns the monitor on which the window currently resides.
 * Returns `null` if current monitor can't be detected.
 */
async function currentMonitor(): Promise<Monitor | null> {
  return invokeTauriCommand({
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'currentMonitor'
        }
      }
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
      cmd: 'manage',
      data: {
        cmd: {
          type: 'primaryMonitor'
        }
      }
    }
  })
}

/** Returns the list of all the monitors available on the system. */
async function availableMonitors(): Promise<Monitor[]> {
  return invokeTauriCommand({
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'availableMonitors'
        }
      }
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

export type { Theme, Monitor, WindowOptions }
