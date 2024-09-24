// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The event system allows you to emit events to the backend and listen to events from it.
 *
 * This package is also accessible with `window.__TAURI__.event` when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

import { invoke, transformCallback } from './core'

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

type UnlistenFn = () => void

type EventName = `${TauriEvent}` | (string & Record<never, never>)

interface Options {
  /**
   * The event target to listen to, defaults to `{ kind: 'Any' }`, see {@link EventTarget}.
   *
   * If a string is provided, {@link EventTarget.AnyLabel} is used.
   */
  target?: string | EventTarget
}

/**
 * @since 1.1.0
 */
enum TauriEvent {
  WINDOW_RESIZED = 'tauri://resize',
  WINDOW_MOVED = 'tauri://move',
  WINDOW_CLOSE_REQUESTED = 'tauri://close-requested',
  WINDOW_DESTROYED = 'tauri://destroyed',
  WINDOW_FOCUS = 'tauri://focus',
  WINDOW_BLUR = 'tauri://blur',
  WINDOW_SCALE_FACTOR_CHANGED = 'tauri://scale-change',
  WINDOW_THEME_CHANGED = 'tauri://theme-changed',
  WINDOW_CREATED = 'tauri://window-created',
  WEBVIEW_CREATED = 'tauri://webview-created',
  DRAG_ENTER = 'tauri://drag-enter',
  DRAG_OVER = 'tauri://drag-over',
  DRAG_DROP = 'tauri://drag-drop',
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
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @param options Event listening options.
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
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
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @param options Event listening options.
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
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
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      _unlisten(event, eventData.id)
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
async function emit(event: string, payload?: unknown): Promise<void> {
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
async function emitTo(
  target: EventTarget | string,
  event: string,
  payload?: unknown
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
