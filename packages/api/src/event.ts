// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The event system allows you to emit events to the backend and listen to events from it.
 *
 * This package is also accessible with `window.__TAURI__.event` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * @remarks All commands used by this module (`core:event:allow-listen`,
 * `allow-unlisten`, `allow-emit` and `allow-emit-to`) are part of the
 * `core:event:default` permission set, which is enabled by default, so no extra
 * capability configuration is needed.
 *
 * @module
 */

import { invoke, transformCallback } from './core'

declare global {
  interface Window {
    __TAURI_EVENT_PLUGIN_INTERNALS__: {
      unregisterListener: (event: string, eventId: number) => void
    }
  }
}

/**
 * The target of an event, used to filter which listeners receive it and which
 * listeners a given emit reaches.
 *
 * - `Any`: matches every target (the default).
 * - `AnyLabel`: matches any window, webview or webview window with the given label.
 * - `App`: the application itself, i.e. listeners registered with `app.listen` on the Rust side.
 * - `Window` / `Webview` / `WebviewWindow`: the specific target with that label.
 *
 * @since 2.0.0
 */
type EventTarget =
  | { kind: 'Any' }
  | { kind: 'AnyLabel'; label: string }
  | { kind: 'App' }
  | { kind: 'Window'; label: string }
  | { kind: 'Webview'; label: string }
  | { kind: 'WebviewWindow'; label: string }

interface Event<T> {
  /** Event name */
  event: EventName
  /** Event identifier used to unlisten */
  id: number
  /** Event payload */
  payload: T
}

type EventCallback<T> = (event: Event<T>) => void

// TODO(v3): mark this as Promise<void>
type UnlistenFn = () => void

type EventName = `${TauriEvent}` | (string & Record<never, never>)

interface Options {
  /**
   * The event target to listen to, defaults to `{ kind: 'Any' }`, see {@link EventTarget}.
   *
   * If a string is provided, it is used as the label of an `AnyLabel` target,
   * i.e. `{ kind: 'AnyLabel', label: <the string> }`.
   */
  target?: string | EventTarget
}

/**
 * The built-in event names emitted by Tauri itself.
 *
 * These are the raw names behind the `on*` helpers of the `Window` and `Webview`
 * classes (e.g. `Window.onResized` listens to {@linkcode TauriEvent.WINDOW_RESIZED}).
 * Prefer those helpers when one exists, since they also decode the payload into
 * the matching class (`PhysicalSize`, `PhysicalPosition`, ...).
 *
 * @example
 * ```typescript
 * import { listen, TauriEvent } from '@tauri-apps/api/event';
 * const unlisten = await listen(TauriEvent.WINDOW_DESTROYED, (event) => {
 *   console.log('a window was destroyed', event.payload);
 * });
 * ```
 *
 * @since 1.1.0
 */
enum TauriEvent {
  /** A window was resized. Payload: the new inner size, in physical pixels. See `Window.onResized`. */
  WINDOW_RESIZED = 'tauri://resize',
  /** A window was moved. Payload: the new outer position, in physical pixels. See `Window.onMoved`. */
  WINDOW_MOVED = 'tauri://move',
  /**
   * The user requested a window to be closed (e.g. clicked the close button).
   * See `Window.onCloseRequested`, which also handles preventing the close.
   */
  WINDOW_CLOSE_REQUESTED = 'tauri://close-requested',
  /** A window was destroyed, i.e. it is gone and its label can be reused. */
  WINDOW_DESTROYED = 'tauri://destroyed',
  /** A window gained focus. See `Window.onFocusChanged`. */
  WINDOW_FOCUS = 'tauri://focus',
  /** A window lost focus. See `Window.onFocusChanged`. */
  WINDOW_BLUR = 'tauri://blur',
  /**
   * The scale factor of the monitor a window is on changed, or the window moved to
   * a monitor with a different scale factor. See `Window.onScaleChanged`.
   */
  WINDOW_SCALE_FACTOR_CHANGED = 'tauri://scale-change',
  /** The system or window theme changed. See `Window.onThemeChanged`. */
  WINDOW_THEME_CHANGED = 'tauri://theme-changed',
  /** A new window was created. */
  WINDOW_CREATED = 'tauri://window-created',
  /**
   * The window's event loop was suspended.
   *
   * #### Platform-specific
   *
   * - **Android:** emitted when the activity is paused.
   * - **Other platforms:** never emitted.
   */
  WINDOW_SUSPENDED = 'tauri://suspended',
  /**
   * The window's event loop was resumed after being suspended.
   *
   * #### Platform-specific
   *
   * - **Android:** emitted when the activity is resumed.
   * - **Other platforms:** never emitted.
   */
  WINDOW_RESUMED = 'tauri://resumed',
  /** A new webview was created. */
  WEBVIEW_CREATED = 'tauri://webview-created',
  /** The user dragged files onto a webview. See `Webview.onDragDropEvent`. */
  DRAG_ENTER = 'tauri://drag-enter',
  /** The user is moving dragged files over a webview. See `Webview.onDragDropEvent`. */
  DRAG_OVER = 'tauri://drag-over',
  /** The user dropped files onto a webview. See `Webview.onDragDropEvent`. */
  DRAG_DROP = 'tauri://drag-drop',
  /** The drag operation left the webview or was cancelled. See `Webview.onDragDropEvent`. */
  DRAG_LEAVE = 'tauri://drag-leave'
}

/**
 * Unregister the event listener associated with the given name and id.
 *
 * @ignore
 * @param event The event name
 * @param eventId Event identifier
 * @returns
 */
async function _unlisten(event: string, eventId: number): Promise<void> {
  window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(event, eventId)
  await invoke('plugin:event|unlisten', {
    event,
    eventId
  })
}

/**
 * Listen to an emitted event to any {@link EventTarget|target}.
 *
 * @example
 * ```typescript
 * import { listen } from '@tauri-apps/api/event';
 * const unlisten = await listen<string>('error', (event) => {
 *   console.log(`Got error, payload: ${event.payload}`);
 * });
 *
 * // call unlisten when your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @param options Event listening options.
 * @returns A promise resolving to a function to unlisten to the event.
 *
 * @remarks Listeners bound to a window or webview are removed automatically when
 * that window or webview is destroyed, so you do not need to unlisten just to
 * avoid leaking across a window close. You should still call the returned
 * function when the listener's own scope ends — for example on page navigation
 * or when a component unmounts — otherwise the handler keeps running for the
 * lifetime of the webview.
 *
 * @since 1.0.0
 */
async function listen<T>(
  event: EventName,
  handler: EventCallback<T>,
  options?: Options
): Promise<UnlistenFn> {
  const target: EventTarget =
    typeof options?.target === 'string'
      ? { kind: 'AnyLabel', label: options.target }
      : (options?.target ?? { kind: 'Any' })
  return invoke<number>('plugin:event|listen', {
    event,
    target,
    handler: transformCallback(handler)
  }).then((eventId) => {
    return async () => _unlisten(event, eventId)
  })
}

/**
 * Listens once to an emitted event to any {@link EventTarget|target}.
 *
 * @example
 * ```typescript
 * import { once } from '@tauri-apps/api/event';
 * interface LoadedPayload {
 *   loggedIn: boolean,
 *   token: string
 * }
 * const unlisten = await once<LoadedPayload>('loaded', (event) => {
 *   console.log(`App is loaded, loggedIn: ${event.payload.loggedIn}, token: ${event.payload.token}`);
 * });
 *
 * // call unlisten when your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @param options Event listening options.
 * @returns A promise resolving to a function to unlisten to the event.
 *
 * @remarks The listener removes itself after the first event, and listeners bound
 * to a window or webview are also removed automatically when that target is
 * destroyed. Still call the returned function when the listener's own scope ends
 * before the event arrives — for example on page navigation or component unmount.
 *
 * @since 1.0.0
 */
async function once<T>(
  event: EventName,
  handler: EventCallback<T>,
  options?: Options
): Promise<UnlistenFn> {
  return listen<T>(
    event,
    (eventData) => {
      void _unlisten(event, eventData.id)
      handler(eventData)
    },
    options
  )
}

/**
 * Emits an event to all {@link EventTarget|targets}.
 *
 * @example
 * ```typescript
 * import { emit } from '@tauri-apps/api/event';
 * await emit('frontend-loaded', { loggedIn: true, token: 'authToken' });
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param payload Event payload.
 *
 * @since 1.0.0
 */
async function emit<T>(event: string, payload?: T): Promise<void> {
  await invoke('plugin:event|emit', {
    event,
    payload
  })
}

/**
 * Emits an event to all {@link EventTarget|targets} matching the given target.
 *
 * @example
 * ```typescript
 * import { emitTo } from '@tauri-apps/api/event';
 * await emitTo('main', 'frontend-loaded', { loggedIn: true, token: 'authToken' });
 * ```
 *
 * @param target Label of the target Window/Webview/WebviewWindow or raw {@link EventTarget} object.
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param payload Event payload.
 *
 * @since 2.0.0
 */
async function emitTo<T>(
  target: EventTarget | string,
  event: string,
  payload?: T
): Promise<void> {
  const eventTarget: EventTarget =
    typeof target === 'string' ? { kind: 'AnyLabel', label: target } : target
  await invoke('plugin:event|emit_to', {
    target: eventTarget,
    event,
    payload
  })
}

export type {
  Event,
  EventTarget,
  EventCallback,
  UnlistenFn,
  EventName,
  Options
}

export { listen, once, emit, emitTo, TauriEvent }
