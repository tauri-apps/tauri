// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { getCurrent as getCurrentWebview, Webview } from './webview'
import type { WindowOptions, WindowSizeConstraints } from './window'
import { Window } from './window'
import { listen, once } from './event'
import type { EventName, EventCallback, UnlistenFn } from './event'
import { invoke } from './core'
import type {
  DragDropEvent,
  DragDropPayload,
  WebviewOptions,
  WebviewLabel
} from './webview'

/**
 * Get an instance of `Webview` for the current webview window.
 *
 * @since 2.0.0
 */
function getCurrent(): WebviewWindow {
  const webview = getCurrentWebview()
  // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
  return new WebviewWindow(webview.label, { skip: true })
}

/**
 * Gets a list of instances of `Webview` for all available webview windows.
 *
 * @since 2.0.0
 */
function getAll(): WebviewWindow[] {
  return window.__TAURI_INTERNALS__.metadata.webviews.map(
    (w) =>
      new WebviewWindow(w.label, {
        // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
        skip: true
      })
  )
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
   * import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
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
   * Gets the Webview for the webview associated with the given label.
   * @example
   * ```typescript
   * import { Webview } from '@tauri-apps/api/webviewWindow';
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
    return getCurrent()
  }

  /**
   * Gets a list of instances of `Webview` for all available webviews.
   */
  static getAll(): WebviewWindow[] {
    // @ts-expect-error `skip` is not defined in the public API but it is handled by the constructor
    return getAll().map((w) => new WebviewWindow(w.label, { skip: true }))
  }

  /**
   * Listen to an emitted event on this webivew window.
   *
   * @example
   * ```typescript
   * import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
   * const unlisten = await WebviewWindow.getCurrent().listen<string>('state-changed', (event) => {
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
      return () => {
        // eslint-disable-next-line security/detect-object-injection
        const listeners = this.listeners[event]
        listeners.splice(listeners.indexOf(handler), 1)
      }
    }
    return listen(event, handler, {
      target: { kind: 'WebviewWindow', label: this.label }
    })
  }

  /**
   * Listen to an emitted event on this webview window only once.
   *
   * @example
   * ```typescript
   * import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
   * const unlisten = await WebviewWindow.getCurrent().once<null>('initialized', (event) => {
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
      target: { kind: 'WebviewWindow', label: this.label }
    })
  }
}

// Order matters, we use window APIs by default
applyMixins(WebviewWindow, [Window, Webview])

/** Extends a base class by other specified classes, without overriding existing properties */
function applyMixins(
  baseClass: { prototype: unknown },
  extendedClasses: unknown
): void {
  ;(Array.isArray(extendedClasses)
    ? extendedClasses
    : [extendedClasses]
  ).forEach((extendedClass: { prototype: unknown }) => {
    Object.getOwnPropertyNames(extendedClass.prototype).forEach((name) => {
      if (
        typeof baseClass.prototype === 'object' &&
        baseClass.prototype &&
        name in baseClass.prototype
      )
        return
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

export { WebviewWindow, getCurrent, getAll }
export type {
  DragDropEvent,
  DragDropPayload,
  WindowSizeConstraints,
  WindowOptions,
  WebviewOptions,
  WebviewLabel
}
