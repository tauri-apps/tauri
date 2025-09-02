// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
  getCurrentWebview,
  Webview,
  WebviewLabel,
  WebviewOptions
} from './webview'
import type { WindowOptions } from './window'
import { Window } from './window'
import { listen, once } from './event'
import type { EventName, EventCallback, UnlistenFn } from './event'
import { invoke } from './core'
import type { Color, DragDropEvent } from './webview'

// Internal-only type to avoid @ts-expect-error everywhere
interface InternalWebviewOptions
  extends Omit<WebviewOptions, 'x' | 'y' | 'width' | 'height'> {
  skip?: boolean
  parent?: Window | WebviewWindow | string
}

/**
 * Get an instance of `Webview` for the current webview window.
 *
 * @since 2.0.0
 */
function getCurrentWebviewWindow(): WebviewWindow {
  const webview = getCurrentWebview()
  return new WebviewWindow(webview.label, { skip: true } as any)
}

/**
 * Gets a list of instances of `Webview` for all available webview windows.
 *
 * @since 2.0.0
 */
async function getAllWebviewWindows(): Promise<WebviewWindow[]> {
  return invoke<string[]>('plugin:window|get_all_windows').then((windows) =>
    windows.map(
      (w) =>
        new WebviewWindow(w, {
          skip: true
        } as any)
    )
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

  constructor(
    label: WebviewLabel,
    options: Omit<WebviewOptions, 'x' | 'y' | 'width' | 'height'>
      & WindowOptions & { skip?: boolean } = {}
  ) {
    this.label = label
    this.listeners = Object.create(null)

    const internalOptions = options as InternalWebviewOptions

    if (!internalOptions?.skip) {
      invoke('plugin:webview|create_webview_window', {
        options: {
          ...internalOptions,
          parent:
            typeof internalOptions.parent === 'string'
              ? internalOptions.parent
              : internalOptions.parent?.label,
          label
        }
      })
        .then(async () => this.emit('tauri://created'))
        .catch(async (e: string) => this.emit('tauri://error', e))
    }
  }

  static async getByLabel(label: string): Promise<WebviewWindow | null> {
    const webview =
      (await getAllWebviewWindows()).find((w) => w.label === label) ?? null
    if (webview) {
      return new WebviewWindow(webview.label, { skip: true } as any)
    }
    return null
  }

  static getCurrent(): WebviewWindow {
    return getCurrentWebviewWindow()
  }

  static async getAll(): Promise<WebviewWindow[]> {
    return getAllWebviewWindows()
  }

  async listen<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return () => {
        const listeners = this.listeners[event]
        const idx = listeners.indexOf(handler)
        if (idx >= 0) listeners.splice(idx, 1)
      }
    }
    return listen(event, handler, {
      target: { kind: 'WebviewWindow', label: this.label }
    })
  }

  async once<T>(
    event: EventName,
    handler: EventCallback<T>
  ): Promise<UnlistenFn> {
    if (this._handleTauriEvent(event, handler)) {
      return () => {
        const listeners = this.listeners[event]
        const idx = listeners.indexOf(handler)
        if (idx >= 0) listeners.splice(idx, 1)
      }
    }
    return once(event, handler, {
      target: { kind: 'WebviewWindow', label: this.label }
    })
  }

  /**
   * Set the window and webview background color.
   *
   * @since 2.1.0
   */
  async setBackgroundColor(color: Color): Promise<void> {
    try {
      await invoke('plugin:window|set_background_color', { color })
      await invoke('plugin:webview|set_webview_background_color', { color })
    } catch (err) {
      throw new Error(`Failed to set background color: ${String(err)}`)
    }
  }
}

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
        typeof baseClass.prototype === 'object'
        && baseClass.prototype
        && name in baseClass.prototype
      ) {
        return
      }
      Object.defineProperty(
        baseClass.prototype,
        name,
        Object.getOwnPropertyDescriptor(extendedClass.prototype, name)
          ?? Object.create(null)
      )
    })
  })
}

export { WebviewWindow, getCurrentWebviewWindow, getAllWebviewWindows }
export type { DragDropEvent, Color }
