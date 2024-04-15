// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides APIs to create webviews, communicate with other webviews and manipulate the current webview.
 *
 * ## Webview events
 *
 * Events can be listened to using {@link Webview.listen}:
 * ```typescript
 * import { getCurrent } from "@tauri-apps/api/webview";
 * getCurrent().listen("my-webview-event", ({ event, payload }) => { });
 * ```
 *
 * @module
 */

import { PhysicalPosition, PhysicalSize } from './dpi'
import type { LogicalPosition, LogicalSize } from './dpi'
import type { EventName, EventCallback, UnlistenFn } from './event'
import {
  TauriEvent,
  // imported for documentation purposes
  // eslint-disable-next-line
  type EventTarget,
  emit,
  emitTo,
  listen,
  once
} from './event'
import { invoke } from './core'
import { Window, getCurrent as getCurrentWindow } from './window'
import { WebviewWindow } from './webviewWindow'

interface DragDropPayload {
  paths: string[]
  position: PhysicalPosition
}

interface DragOverPayload {
  position: PhysicalPosition
}

/** The drag and drop event types. */
type DragDropEvent =
  | ({ type: 'dragged' } & DragDropPayload)
  | ({ type: 'dragOver' } & DragOverPayload)
  | ({ type: 'dropped' } & DragDropPayload)
  | { type: 'cancelled' }

/**
 * Get an instance of `Webview` for the current webview.
 *
 * @since 2.0.0
 */
function getCurrent(): Webview {
  return new Webview(
    getCurrentWindow(),
    window.__TAURI_INTERNALS__.metadata.currentWebview.label,
    {
      // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
      skip: true
    }
  )
}

/**
 * Gets a list of instances of `Webview` for all available webviews.
 *
 * @since 2.0.0
 */
function getAll(): Webview[] {
  return window.__TAURI_INTERNALS__.metadata.webviews.map(
    (w) =>
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      new Webview(Window.getByLabel(w.windowLabel)!, w.label, {
        // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
        skip: true
      })
  )
}

/** @ignore */
// events that are emitted right here instead of by the created webview
const localTauriEvents = ['tauri://created', 'tauri://error']
/** @ignore */
export type WebviewLabel = string

/**
 * Create new webview or get a handle to an existing one.
 *
 * Webviews are identified by a *label*  a unique identifier that can be used to reference it later.
 * It may only contain alphanumeric characters `a-zA-Z` plus the following special characters `-`, `/`, `:` and `_`.
 *
 * @example
 * ```typescript
 * import { Window } from "@tauri-apps/api/window"
 * import { Webview } from "@tauri-apps/api/webview"
 *
 * const appWindow = new Window('uniqueLabel');
 *
 * // loading embedded asset:
 * const webview = new Webview(appWindow, 'theUniqueLabel', {
 *   url: 'path/to/page.html'
 * });
 * // alternatively, load a remote URL:
 * const webview = new Webview(appWindow, 'theUniqueLabel', {
 *   url: 'https://github.com/tauri-apps/tauri'
 * });
 *
 * webview.once('tauri://created', function () {
 *  // webview successfully created
 * });
 * webview.once('tauri://error', function (e) {
 *  // an error happened creating the webview
 * });
 *
 * // emit an event to the backend
 * await webview.emit("some-event", "data");
 * // listen to an event from the backend
 * const unlisten = await webview.listen("event-name", e => {});
 * unlisten();
 * ```
 *
 * @since 2.0.0
 */
class Webview {
  /** The webview label. It is a unique identifier for the webview, can be used to reference it later. */
  label: WebviewLabel
  /** The window hosting this webview. */
  window: Window
  /** Local event listeners. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  listeners: Record<string, Array<EventCallback<any>>>

  /**
   * Creates a new Webview.
   * @example
   * ```typescript
   * import { Window } from '@tauri-apps/api/window'
   * import { Webview } from '@tauri-apps/api/webview'
   * const appWindow = new Window('my-label')
   * const webview = new Webview(appWindow, 'my-label', {
   *   url: 'https://github.com/tauri-apps/tauri'
   * });
   * webview.once('tauri://created', function () {
   *  // webview successfully created
   * });
   * webview.once('tauri://error', function (e) {
   *  // an error happened creating the webview
   * });
   * ```
   *
   * @param window the window to add this webview to.
   * @param label The unique webview label. Must be alphanumeric: `a-zA-Z-/:_`.
   * @returns The {@link Webview} instance to communicate with the webview.
   */
  constructor(window: Window, label: WebviewLabel, options: WebviewOptions) {
    this.window = window
    this.label = label
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.listeners = Object.create(null)

    // @ts-expect-error `skip` is not a public API so it is not defined in WebviewOptions
    if (!options?.skip) {
      invoke('plugin:webview|create_webview', {
        windowLabel: window.label,
        label,
        options
      })
        .then(async () => this.emit('tauri://created'))
        .catch(async (e: string) => this.emit('tauri://error', e))
    }
  }

  /**
   * Gets the Webview for the webview associated with the given label.
   * @example
   * ```typescript
   * import { Webview } from '@tauri-apps/api/webview';
   * const mainWebview = Webview.getByLabel('main');
   * ```
   *
   * @param label The webview label.
   * @returns The Webview instance to communicate with the webview or null if the webview doesn't exist.
   */
  static getByLabel(label: string): Webview | null {
    return getAll().find((w) => w.label === label) ?? null
  }

  /**
   * Get an instance of `Webview` for the current webview.
   */
  static getCurrent(): Webview {
    return getCurrent()
  }

  /**
   * Gets a list of instances of `Webview` for all available webviews.
   */
  static getAll(): Webview[] {
    return getAll()
  }

  /**
   * Listen to an emitted event on this webview.
   *
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * const unlisten = await getCurrent().listen<string>('state-changed', (event) => {
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
    return listen(event, handler, {
      target: { kind: 'Webview', label: this.label }
    })
  }

  /**
   * Listen to an emitted event on this webview only once.
   *
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * const unlisten = await getCurrent().once<null>('initialized', (event) => {
   *   console.log(`Webview initialized!`);
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
    return once(event, handler, {
      target: { kind: 'Webview', label: this.label }
    })
  }

  /**
   * Emits an event to all {@link EventTarget|targets}.
   *
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * await getCurrent().emit('webview-loaded', { loggedIn: true, token: 'authToken' });
   * ```
   *
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param payload Event payload.
   */
  async emit(event: string, payload?: unknown): Promise<void> {
    if (localTauriEvents.includes(event)) {
      // eslint-disable-next-line
      for (const handler of this.listeners[event] || []) {
        handler({
          event,
          id: -1,
          payload
        })
      }
      return Promise.resolve()
    }
    return emit(event, payload)
  }

  /**
   * Emits an event to all {@link EventTarget|targets} matching the given target.
   *
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * await getCurrent().emitTo('main', 'webview-loaded', { loggedIn: true, token: 'authToken' });
   * ```
   *
   * @param target Label of the target Window/Webview/WebviewWindow or raw {@link EventTarget} object.
   * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
   * @param payload Event payload.
   */
  async emitTo(
    target: string | EventTarget,
    event: string,
    payload?: unknown
  ): Promise<void> {
    if (localTauriEvents.includes(event)) {
      // eslint-disable-next-line
      for (const handler of this.listeners[event] || []) {
        handler({
          event,
          id: -1,
          payload
        })
      }
      return Promise.resolve()
    }
    return emitTo(target, event, payload)
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
   * The position of the top-left hand corner of the webview's client area relative to the top-left hand corner of the desktop.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * const position = await getCurrent().position();
   * ```
   *
   * @returns The webview's position.
   */
  async position(): Promise<PhysicalPosition> {
    return invoke<{ x: number; y: number }>('plugin:webview|webview_position', {
      label: this.label
    }).then(({ x, y }) => new PhysicalPosition(x, y))
  }

  /**
   * The physical size of the webview's client area.
   * The client area is the content of the webview, excluding the title bar and borders.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * const size = await getCurrent().size();
   * ```
   *
   * @returns The webview's size.
   */
  async size(): Promise<PhysicalSize> {
    return invoke<{ width: number; height: number }>(
      'plugin:webview|webview_size',
      {
        label: this.label
      }
    ).then(({ width, height }) => new PhysicalSize(width, height))
  }

  // Setters

  /**
   * Closes the webview.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * await getCurrent().close();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async close(): Promise<void> {
    return invoke('plugin:webview|close', {
      label: this.label
    })
  }

  /**
   * Resizes the webview.
   * @example
   * ```typescript
   * import { getCurrent, LogicalSize } from '@tauri-apps/api/webview';
   * await getCurrent().setSize(new LogicalSize(600, 500));
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

    return invoke('plugin:webview|set_webview_size', {
      label: this.label,
      value: {
        type: size.type,
        data: {
          width: size.width,
          height: size.height
        }
      }
    })
  }

  /**
   * Sets the webview position.
   * @example
   * ```typescript
   * import { getCurrent, LogicalPosition } from '@tauri-apps/api/webview';
   * await getCurrent().setPosition(new LogicalPosition(600, 500));
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

    return invoke('plugin:webview|set_webview_position', {
      label: this.label,
      value: {
        type: position.type,
        data: {
          x: position.x,
          y: position.y
        }
      }
    })
  }

  /**
   * Bring the webview to front and focus.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * await getCurrent().setFocus();
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async setFocus(): Promise<void> {
    return invoke('plugin:webview|set_webview_focus', {
      label: this.label
    })
  }

  /**
   * Set webview zoom level.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * await getCurrent().setZoom(1.5);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async setZoom(scaleFactor: number): Promise<void> {
    return invoke('plugin:webview|set_webview_zoom', {
      label: this.label,
      value: scaleFactor
    })
  }

  /**
   * Moves this webview to the given label.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * await getCurrent().reparent('other-window');
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async reparent(window: Window | WebviewWindow | string): Promise<void> {
    return invoke('plugin:webview|set_webview_focus', {
      label: this.label,
      window: typeof window === 'string' ? window : window.label
    })
  }

  // Listeners

  /**
   * Listen to a file drop event.
   * The listener is triggered when the user hovers the selected files on the webview,
   * drops the files or cancels the operation.
   *
   * @example
   * ```typescript
   * import { getCurrent } from "@tauri-apps/api/webview";
   * const unlisten = await getCurrent().onDragDropEvent((event) => {
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
   */
  async onDragDropEvent(
    handler: EventCallback<DragDropEvent>
  ): Promise<UnlistenFn> {
    const unlistenDrag = await this.listen<DragDropPayload>(
      TauriEvent.DRAG,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'dragged',
            paths: event.payload.paths,
            position: mapPhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenDrop = await this.listen<DragDropPayload>(
      TauriEvent.DROP,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'dropped',
            paths: event.payload.paths,
            position: mapPhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenDragOver = await this.listen<DragDropPayload>(
      TauriEvent.DROP_CANCELLED,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'dragOver',
            position: mapPhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenCancel = await this.listen<null>(
      TauriEvent.DROP_CANCELLED,
      (event) => {
        handler({ ...event, payload: { type: 'cancelled' } })
      }
    )

    return () => {
      unlistenDrag()
      unlistenDrop()
      unlistenDragOver()
      unlistenCancel()
    }
  }
}

function mapPhysicalPosition(m: PhysicalPosition): PhysicalPosition {
  return new PhysicalPosition(m.x, m.y)
}

/**
 * Configuration for the webview to create.
 *
 * @since 2.0.0
 */
interface WebviewOptions {
  /**
   * Remote URL or local file path to open.
   *
   * - URL such as `https://github.com/tauri-apps` is opened directly on a Tauri webview.
   * - data: URL such as `data:text/html,<html>...` is only supported with the `webview-data-url` Cargo feature for the `tauri` dependency.
   * - local file path or route such as `/path/to/page.html` or `/users` is appended to the application URL (the devServer URL on development, or `tauri://localhost/` and `https://tauri.localhost/` on production).
   */
  url?: string
  /** The initial vertical position. */
  x: number
  /** The initial horizontal position. */
  y: number
  /** The initial width. */
  width: number
  /** The initial height. */
  height: number
  /**
   * Whether the webview is transparent or not.
   * Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri.conf.json > app > macOSPrivateApi`.
   * WARNING: Using private APIs on `macOS` prevents your application from being accepted to the `App Store`.
   */
  transparent?: boolean
  /**
   * Whether the drag and drop is enabled or not on the webview. By default it is enabled.
   *
   * Disabling it is required to use HTML5 drag and drop on the frontend on Windows.
   */
  dragDropEnabled?: boolean
  /**
   * Whether clicking an inactive webview also clicks through to the webview on macOS.
   */
  acceptFirstMouse?: boolean
  /**
   * The user agent for the webview.
   */
  userAgent?: string
  /**
   * Whether or not the webview should be launched in incognito mode.
   *
   * #### Platform-specific
   *
   * - **Android:** Unsupported.
   */
  incognito?: boolean
  /**
   * The proxy URL for the WebView for all network requests.
   *
   * Must be either a `http://` or a `socks5://` URL.
   *
   * #### Platform-specific
   *
   * - **macOS**: Requires the `macos-proxy` feature flag and only compiles for macOS 14+.
   * */
  proxyUrl?: string
  /**
   * Whether page zooming by hotkeys is enabled
   *
   * ## Platform-specific:
   *
   * - **Windows**: Controls WebView2's [`IsZoomControlEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings?view=webview2-winrt-1.0.2420.47#iszoomcontrolenabled) setting.
   * - **MacOS / Linux**: Injects a polyfill that zooms in and out with `ctrl/command` + `-/=`,
   * 20% in each step, ranging from 20% to 1000%. Requires `webview:allow-set-webview-zoom` permission
   *
   * - **Android / iOS**: Unsupported.
   */
  zoomHotkeysEnabled?: boolean
}

export { Webview, getCurrent, getAll }

export type { DragDropEvent, DragDropPayload, WebviewOptions }
