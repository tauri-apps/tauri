// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The event system allows you to emit events to the backend and listen to events from it.
 *
 * This package is also accessible with `window.__TAURI__.event` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */

import * as eventApi from './helpers/event'
import type {
  EventName,
  EventCallback,
  UnlistenFn,
  Event
} from './helpers/event'

/**
 * Listen to an event from the backend.
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @return A promise resolving to a function to unlisten to the event.
 */
async function listen<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return eventApi.listen(event, null, handler)
}

/**
 * Listen to an one-off event from the backend.
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @returns A promise resolving to a function to unlisten to the event.
 */
async function once<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return eventApi.once(event, null, handler)
}

/**
 * Emits an event to the backend.
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param [payload] Event payload
 * @returns
 */
async function emit(event: string, payload?: unknown): Promise<void> {
  return eventApi.emit(event, undefined, payload)
}

export type { Event, EventName, EventCallback, UnlistenFn }

export { listen, once, emit }
