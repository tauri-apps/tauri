// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
import { TauriEvent, emit, listen, once } from './event'
import { invoke } from './core'
import { Window, getCurrent as getCurrentWindow } from './window'
import type { WindowOptions } from './window'

interface FileDropPayload {
  paths: string[]
  position: PhysicalPosition
}

/** The file drop event types. */
type FileDropEvent =
  | ({ type: 'hover' } & FileDropPayload)
  | ({ type: 'drop' } & FileDropPayload)
  | { type: 'cancel' }

/**
 * Get an instance of `Webview` for the current webview.
 *
 * @since 1.0.0
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
 * @since 1.0.0
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
 * // loading embedded asset:
 * const appWindow = new Window('uniqueLabel')
 *
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
 * await webview.emit("some event", "data");
 * // listen to an event from the backend
 * const unlisten = await webview.listen("event name", e => {});
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
   * const webview = new Window(appWindow, 'my-label', {
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
   * Listen to an event emitted by the backend that is tied to the webview.
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
      target: { kind: 'webview', label: this.label }
    })
  }

  /**
   * Listen to an one-off event emitted by the backend that is tied to the webview.
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
      target: { kind: 'webview', label: this.label }
    })
  }

  /**
   * Emits an event to the backend, tied to the webview.
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
          source: { kind: 'webview', label: this.label },
          payload
        })
      }
      return Promise.resolve()
    }
    return emit(event, payload, {
      target: { kind: 'webview', label: this.label }
    })
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
   * @returns The webview's inner position.
   */
  async position(): Promise<PhysicalPosition> {
    return invoke<{ x: number; y: number }>('plugin:webview|position', {
      label: this.label
    }).then(({ x, y }) => new PhysicalPosition(x, y))
  }

  /**
   * The physical size of the webview's client area.
   * The client area is the content of the webview, excluding the title bar and borders.
   * @example
   * ```typescript
   * import { getCurrent } from '@tauri-apps/api/webview';
   * const size = await getCurrent().innerSize();
   * ```
   *
   * @returns The webview's inner size.
   */
  async size(): Promise<PhysicalSize> {
    return invoke<{ width: number; height: number }>('plugin:webview|size', {
      label: this.label
    }).then(({ width, height }) => new PhysicalSize(width, height))
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
   * Resizes the webview with a new inner size.
   * @example
   * ```typescript
   * import { getCurrent, LogicalSize } from '@tauri-apps/api/webview';
   * await getCurrent().setSize(new LogicalSize(600, 500));
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

    return invoke('plugin:webview|set_size', {
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
   * Sets the webview outer position.
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

    return invoke('plugin:webview|set_position', {
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
    return invoke('plugin:webview|set_focus', {
      label: this.label
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
   * const unlisten = await getCurrent().onFileDropEvent((event) => {
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
  async onFileDropEvent(
    handler: EventCallback<FileDropEvent>
  ): Promise<UnlistenFn> {
    const unlistenFileDrop = await this.listen<FileDropPayload>(
      TauriEvent.WEBVIEW_FILE_DROP,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'drop',
            paths: event.payload.paths,
            position: mapPhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenFileHover = await this.listen<FileDropPayload>(
      TauriEvent.WEBVIEW_FILE_DROP_HOVER,
      (event) => {
        handler({
          ...event,
          payload: {
            type: 'hover',
            paths: event.payload.paths,
            position: mapPhysicalPosition(event.payload.position)
          }
        })
      }
    )

    const unlistenCancel = await this.listen<null>(
      TauriEvent.WEBVIEW_FILE_DROP_CANCELLED,
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
}

function mapPhysicalPosition(m: PhysicalPosition): PhysicalPosition {
  return new PhysicalPosition(m.x, m.y)
}

// eslint-disable-next-line @typescript-eslint/no-unsafe-declaration-merging
interface WebviewWindow extends Webview, Window {}

// eslint-disable-next-line @typescript-eslint/no-unsafe-declaration-merging
class WebviewWindow {
  label: string
  /** Local event listeners. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  listeners: Record<string, Array<EventCallback<any>>>

  /**
   * Creates a new {@link Window} hosting a {@link Webview}.
   * @example
   * ```typescript
   * import { WebviewWindow } from '@tauri-apps/api/webview'
   * const webview = new WebviewWindow('my-label', {
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
   * @param label The unique webview label. Must be alphanumeric: `a-zA-Z-/:_`.
   * @returns The {@link WebviewWindow} instance to communicate with the window and webview.
   */
  constructor(
    label: WebviewLabel,
    options: Omit<WebviewOptions, 'x' | 'y' | 'width' | 'height'> &
      WindowOptions = {}
  ) {
    this.label = label
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.listeners = Object.create(null)

    // @ts-expect-error `skip` is not a public API so it is not defined in WebviewOptions
    if (!options?.skip) {
      invoke('plugin:webview|create_webview_window', {
        options: {
          ...options,
          label
        }
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
  static getByLabel(label: string): WebviewWindow | null {
    const webview = getAll().find((w) => w.label === label) ?? null
    if (webview) {
      // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
      return new WebviewWindow(webview.label, { skip: true })
    }
    return null
  }

  /**
   * Get an instance of `Webview` for the current webview.
   */
  static getCurrent(): WebviewWindow {
    const webview = getCurrent()
    // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
    return new WebviewWindow(webview.label, { skip: true })
  }

  /**
   * Gets a list of instances of `Webview` for all available webviews.
   */
  static getAll(): WebviewWindow[] {
    // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
    return getAll().map((w) => new WebviewWindow(w.label, { skip: true }))
  }
}

// order matters, we use window APIs by default
applyMixins(WebviewWindow, [Webview, Window])

/** Extends a base class by other specifed classes */
function applyMixins(
  baseClass: { prototype: unknown },
  extendedClasses: unknown
): void {
  ;(Array.isArray(extendedClasses)
    ? extendedClasses
    : [extendedClasses]
  ).forEach((extendedClass: { prototype: unknown }) => {
    Object.getOwnPropertyNames(extendedClass.prototype).forEach((name) => {
      Object.defineProperty(
        baseClass.prototype,
        name,
        // eslint-disable-next-line
        Object.getOwnPropertyDescriptor(extendedClass.prototype, name) ??
          Object.create(null)
      )
    })
  })
}
/**
 * Configuration for the webview to create.
 *
 * @since 1.0.0
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
   * Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri.conf.json > tauri > macOSPrivateApi`.
   * WARNING: Using private APIs on `macOS` prevents your application from being accepted to the `App Store`.
   */
  transparent?: boolean
  /**
   * Whether the file drop is enabled or not on the webview. By default it is enabled.
   *
   * Disabling it is required to use drag and drop on the frontend on Windows.
   */
  fileDropEnabled?: boolean
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
}

export { Webview, WebviewWindow, getCurrent, getAll }

export type { FileDropEvent, WebviewOptions }
