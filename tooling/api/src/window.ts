// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides APIs to create windows, communicate with other windows and manipulate the current window.
 *
 * This package is also accessible with `window.__TAURI__.window` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.window`](https://tauri.app/v1/api/config/#allowlistconfig.window) in `tauri.conf.json`:
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
 *         "setIgnoreCursorEvents": true,
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
 * import { appWindow } from "@tauri-apps/api/window";
 * appWindow.listen("my-window-event", ({ event, payload }) => { });
 * ```
 *
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import type { EventName, EventCallback, UnlistenFn } from './event'
import { emit, Event, listen, once } from './helpers/event'
import { TauriEvent } from './event'

type Theme = 'light' | 'dark'
type TitleBarStyle = 'visible' | 'transparent' | 'overlay'

/**
 * Allows you to retrieve information about a given monitor.
 *
 * @since 1.0.0
 */
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

/** The file drop event types. */
type FileDropEvent =
  | { type: 'hover'; paths: string[] }
  | { type: 'drop'; paths: string[] }
  | { type: 'cancel' }

/**
 * A size represented in logical pixels.
 *
 * @since 1.0.0
 */
class LogicalSize {
  type = 'Logical'
  width: number
  height: number

  constructor(width: number, height: number) {
    this.width = width
    this.height = height
  }
}

/**
 * A size represented in physical pixels.
 *
 * @since 1.0.0
 */
class PhysicalSize {
  type = 'Physical'
  width: number
  height: number

  constructor(width: number, height: number) {
    this.width = width
    this.height = height
  }

  /**
   * Converts the physical size to a logical one.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const factor = await appWindow.scaleFactor();
   * const size = await appWindow.innerSize();
   * const logical = size.toLogical(factor);
   * ```
   *  */
  toLogical(scaleFactor: number): LogicalSize {
    return new LogicalSize(this.width / scaleFactor, this.height / scaleFactor)
  }
}

/**
 *  A position represented in logical pixels.
 *
 * @since 1.0.0
 */
class LogicalPosition {
  type = 'Logical'
  x: number
  y: number

  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }
}

/**
 *  A position represented in physical pixels.
 *
 * @since 1.0.0
 */
class PhysicalPosition {
  type = 'Physical'
  x: number
  y: number

  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }

  /**
   * Converts the physical position to a logical one.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const factor = await appWindow.scaleFactor();
   * const position = await appWindow.innerPosition();
   * const logical = position.toLogical(factor);
   * ```
   * */
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
 * @since 1.0.0
 */
function getCurrent(): WebviewWindow {
  return new WebviewWindow(window.__TAURI_METADATA__.__currentWindow.label, {
    // @ts-expect-error
    skip: true
  })
}

/**
 * Gets a list of instances of `WebviewWindow` for all available webview windows.
 *
 * @since 1.0.0
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
 *
 * @since 1.0.0
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const unlisten = await appWindow.listen<string>('state-changed', (event) => {
   *   console.log(`Got error: ${payload}`);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const unlisten = await appWindow.once<null>('initialized', (event) => {
   *   console.log(`Window initialized!`);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param handler Event handler.
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.emit('window-loaded', { loggedIn: true, token: 'authToken' });
   * ```
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
 *
 * @since 1.0.0
 */
class WindowManager extends WebviewWindowHandle {
  // Getters
  /**
   * The scale factor that can be used to map physical pixels to logical pixels.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const factor = await appWindow.scaleFactor();
   * ```
   *
   * @returns The window's monitor scale factor.
   * */
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

  /**
   * The position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const position = await appWindow.innerPosition();
   * ```
   *
   * @returns The window's inner position.
   *  */
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

  /**
   * The position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const position = await appWindow.outerPosition();
   * ```
   *
   * @returns The window's outer position.
   *  */
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const size = await appWindow.innerSize();
   * ```
   *
   * @returns The window's inner size.
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const size = await appWindow.outerSize();
   * ```
   *
   * @returns The window's outer size.
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

  /**
   * Gets the window's current fullscreen state.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const fullscreen = await appWindow.isFullscreen();
   * ```
   *
   * @returns Whether the window is in fullscreen mode or not.
   *  */
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

  /**
   * Gets the window's current maximized state.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const maximized = await appWindow.isMaximized();
   * ```
   *
   * @returns Whether the window is maximized or not.
   * */
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

  /**
   * Gets the window's current decorated state.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const decorated = await appWindow.isDecorated();
   * ```
   *
   * @returns Whether the window is decorated or not.
   *  */
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

  /**
   * Gets the window's current resizable state.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const resizable = await appWindow.isResizable();
   * ```
   *
   * @returns Whether the window is resizable or not.
   *  */
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

  /**
   * Gets the window's current visible state.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const visible = await appWindow.isVisible();
   * ```
   *
   * @returns Whether the window is visible or not.
   *  */
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

  /**
   * Gets the window's current theme.
   *
   * #### Platform-specific
   *
   * - **macOS:** Theme was introduced on macOS 10.14. Returns `light` on macOS 10.13 and below.
   *
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * const theme = await appWindow.theme();
   * ```
   *
   * @returns The window theme.
   * */
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.center();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.requestUserAttention();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setResizable(false);
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setTitle('Tauri');
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.maximize();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.unmaximize();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.toggleMaximize();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.minimize();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.unminimize();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.show();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.hide();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.close();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setDecorations(false);
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setAlwaysOnTop(true);
   * ```
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
   * import { appWindow, LogicalSize } from '@tauri-apps/api/window';
   * await appWindow.setSize(new LogicalSize(600, 500));
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
   * import { appWindow, PhysicalSize } from '@tauri-apps/api/window';
   * await appWindow.setMinSize(new PhysicalSize(600, 500));
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
   * import { appWindow, LogicalSize } from '@tauri-apps/api/window';
   * await appWindow.setMaxSize(new LogicalSize(600, 500));
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
   * import { appWindow, LogicalPosition } from '@tauri-apps/api/window';
   * await appWindow.setPosition(new LogicalPosition(600, 500));
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setFullscreen(true);
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setFocus();
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setIcon('/tauri/awesome.png');
   * ```
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
   * Whether the window icon should be hidden from the taskbar or not.
   *
   * #### Platform-specific
   *
   * - **macOS:** Unsupported.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setSkipTaskbar(true);
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setCursorGrab(true);
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setCursorVisible(false);
   * ```
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
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setCursorIcon('help');
   * ```
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
   * @example
   * ```typescript
   * import { appWindow, LogicalPosition } from '@tauri-apps/api/window';
   * await appWindow.setCursorPosition(new LogicalPosition(600, 300));
   * ```
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
   * Changes the cursor events behavior.
   *
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.setIgnoreCursorEvents(true);
   * ```
   *
   * @param ignore `true` to ignore the cursor events; `false` to process them as usual.
   * @returns A promise indicating the success or failure of the operation.
   */
  async setIgnoreCursorEvents(ignore: boolean): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Window',
      message: {
        cmd: 'manage',
        data: {
          label: this.label,
          cmd: {
            type: 'setIgnoreCursorEvents',
            payload: ignore
          }
        }
      }
    })
  }

  /**
   * Starts dragging the window.
   * @example
   * ```typescript
   * import { appWindow } from '@tauri-apps/api/window';
   * await appWindow.startDragging();
   * ```
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

  // Listeners

  /**
   * Listen to window resize.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onResized(({ payload: size }) => {
   *  console.log('Window resized', size);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
   */
  async onResized(handler: EventCallback<PhysicalSize>): Promise<UnlistenFn> {
    return this.listen<PhysicalSize>(TauriEvent.WINDOW_RESIZED, handler)
  }

  /**
   * Listen to window move.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onMoved(({ payload: position }) => {
   *  console.log('Window moved', position);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
   */
  async onMoved(handler: EventCallback<PhysicalPosition>): Promise<UnlistenFn> {
    return this.listen<PhysicalPosition>(TauriEvent.WINDOW_MOVED, handler)
  }

  /**
   * Listen to window close requested. Emitted when the user requests to closes the window.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * import { confirm } from '@tauri-apps/api/dialog';
   * const unlisten = await appWindow.onCloseRequested(async (event) => {
   *   const confirmed = await confirm('Are you sure?');
   *   if (!confirmed) {
   *     // user did not confirm closing the window; let's prevent it
   *     event.preventDefault();
   *   }
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
   */
  async onCloseRequested(
    handler: (event: CloseRequestedEvent) => void
  ): Promise<UnlistenFn> {
    return this.listen<null>(TauriEvent.WINDOW_CLOSE_REQUESTED, (event) => {
      const evt = new CloseRequestedEvent(event)
      void Promise.resolve(handler(evt)).then(() => {
        if (!evt.isPreventDefault()) {
          return this.close()
        }
      })
    })
  }

  /**
   * Listen to window focus change.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onFocusChanged(({ payload: focused }) => {
   *  console.log('Focus changed, window is focused? ' + focused);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
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
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onScaleChanged(({ payload }) => {
   *  console.log('Scale changed', payload.scaleFactor, payload.size);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
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
   * Listen to the window menu item click. The payload is the item id.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onMenuClicked(({ payload: menuId }) => {
   *  console.log('Menu clicked: ' + menuId);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
   */
  async onMenuClicked(handler: EventCallback<string>): Promise<UnlistenFn> {
    return this.listen<string>(TauriEvent.MENU, handler)
  }

  /**
   * Listen to a file drop event.
   * The listener is triggered when the user hovers the selected files on the window,
   * drops the files or cancels the operation.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onFileDropEvent((event) => {
   *  if (event.payload.type === 'hover') {
   *    console.log('User hovering', event.payload.paths);
   *  } else if (event.payload.type === 'drop') {
   *    console.log('User dropped', event.payload.paths);
   *  } else {
   *    console.log('File drop cancelled');
   *  }
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
   */
  async onFileDropEvent(
    handler: EventCallback<FileDropEvent>
  ): Promise<UnlistenFn> {
    const unlistenFileDrop = await this.listen<string[]>(
      TauriEvent.WINDOW_FILE_DROP,
      (event) => {
        handler({ ...event, payload: { type: 'drop', paths: event.payload } })
      }
    )

    const unlistenFileHover = await this.listen<string[]>(
      TauriEvent.WINDOW_FILE_DROP_HOVER,
      (event) => {
        handler({ ...event, payload: { type: 'hover', paths: event.payload } })
      }
    )

    const unlistenCancel = await this.listen<null>(
      TauriEvent.WINDOW_FILE_DROP_CANCELLED,
      (event) => {
        handler({ ...event, payload: { type: 'cancel' } })
      }
    )

    return () => {
      unlistenFileDrop()
      unlistenFileHover()
      unlistenCancel()
    }
  }

  /**
   * Listen to the system theme change.
   *
   * @example
   * ```typescript
   * import { appWindow } from "@tauri-apps/api/window";
   * const unlisten = await appWindow.onThemeChanged(({ payload: theme }) => {
   *  console.log('New theme: ' + theme);
   * });
   *
   * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
   * unlisten();
   * ```
   *
   * @returns A promise resolving to a function to unlisten to the event.
   * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
   *
   * @since 1.0.2
   */
  async onThemeChanged(handler: EventCallback<Theme>): Promise<UnlistenFn> {
    return this.listen<Theme>(TauriEvent.WINDOW_THEME_CHANGED, handler)
  }
}

/**
 * @since 1.0.2
 */
class CloseRequestedEvent {
  /** Event name */
  event: EventName
  /** The label of the window that emitted this event. */
  windowLabel: string
  /** Event identifier used to unlisten */
  id: number
  private _preventDefault = false

  constructor(event: Event<null>) {
    this.event = event.event
    this.windowLabel = event.windowLabel
    this.id = event.id
  }

  preventDefault(): void {
    this._preventDefault = true
  }

  isPreventDefault(): boolean {
    return this._preventDefault
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
 * });
 * // alternatively, load a remote URL:
 * const webview = new WebviewWindow('theUniqueLabel', {
 *   url: 'https://github.com/tauri-apps/tauri'
 * });
 *
 * webview.once('tauri://created', function () {
 *  // webview window successfully created
 * });
 * webview.once('tauri://error', function (e) {
 *  // an error happened creating the webview window
 * });
 *
 * // emit an event to the backend
 * await webview.emit("some event", "data");
 * // listen to an event from the backend
 * const unlisten = await webview.listen("event name", e => {});
 * unlisten();
 * ```
 *
 * @since 1.0.2
 */
class WebviewWindow extends WindowManager {
  /**
   * Creates a new WebviewWindow.
   * @example
   * ```typescript
   * import { WebviewWindow } from '@tauri-apps/api/window';
   * const webview = new WebviewWindow('my-label', {
   *   url: 'https://github.com/tauri-apps/tauri'
   * });
   * webview.once('tauri://created', function () {
   *  // webview window successfully created
   * });
   * webview.once('tauri://error', function (e) {
   *  // an error happened creating the webview window
   * });
   * ```
   *
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
   * @example
   * ```typescript
   * import { WebviewWindow } from '@tauri-apps/api/window';
   * const mainWindow = WebviewWindow.getByLabel('main');
   * ```
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

/**
 * Configuration for the window to create.
 *
 * @since 1.0.0
 */
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
  /** Whether the window will be initially focused or not. */
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
   * If `true`, sets the window title to be hidden on macOS.
   */
  hiddenTitle?: boolean
  /**
   * Whether clicking an inactive window also clicks through to the webview on macOS.
   */
  acceptFirstMouse?: boolean
  /**
   * Defines the window [tabbing identifier](https://developer.apple.com/documentation/appkit/nswindow/1644704-tabbingidentifier) on macOS.
   *
   * Windows with the same tabbing identifier will be grouped together.
   * If the tabbing identifier is not set, automatic tabbing will be disabled.
   */
  tabbingIdentifier?: string
  /**
   * The user agent for the webview.
   */
  userAgent?: string
}

function mapMonitor(m: Monitor | null): Monitor | null {
  return m === null
    ? null
    : {
        name: m.name,
        scaleFactor: m.scaleFactor,
        position: new PhysicalPosition(m.position.x, m.position.y),
        size: new PhysicalSize(m.size.width, m.size.height)
      }
}

/**
 * Returns the monitor on which the window currently resides.
 * Returns `null` if current monitor can't be detected.
 * @example
 * ```typescript
 * import { currentMonitor } from '@tauri-apps/api/window';
 * const monitor = currentMonitor();
 * ```
 *
 * @since 1.0.0
 */
async function currentMonitor(): Promise<Monitor | null> {
  return invokeTauriCommand<Monitor | null>({
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'currentMonitor'
        }
      }
    }
  }).then(mapMonitor)
}

/**
 * Returns the primary monitor of the system.
 * Returns `null` if it can't identify any monitor as a primary one.
 * @example
 * ```typescript
 * import { primaryMonitor } from '@tauri-apps/api/window';
 * const monitor = primaryMonitor();
 * ```
 *
 * @since 1.0.0
 */
async function primaryMonitor(): Promise<Monitor | null> {
  return invokeTauriCommand<Monitor | null>({
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'primaryMonitor'
        }
      }
    }
  }).then(mapMonitor)
}

/**
 * Returns the list of all the monitors available on the system.
 * @example
 * ```typescript
 * import { availableMonitors } from '@tauri-apps/api/window';
 * const monitors = availableMonitors();
 * ```
 *
 * @since 1.0.0
 */
async function availableMonitors(): Promise<Monitor[]> {
  return invokeTauriCommand<Monitor[]>({
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'availableMonitors'
        }
      }
    }
  }).then((ms) => ms.map(mapMonitor) as Monitor[])
}

export {
  WebviewWindow,
  WebviewWindowHandle,
  WindowManager,
  CloseRequestedEvent,
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

export type {
  Theme,
  TitleBarStyle,
  Monitor,
  ScaleFactorChanged,
  FileDropEvent,
  WindowOptions
}
