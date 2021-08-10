// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The event system allows you to emit events to the backend and listen to events from it.
 *
 * This package is also accessible with `window.__TAURI__.event` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import { emit as emitEvent } from './helpers/event'
import { transformCallback } from './tauri'
import { LiteralUnion } from 'type-fest'

interface Event<T> {
  /** Event name */
  event: string
  /** Event identifier used to unlisten */
  id: number
  /** Event payload */
  payload: T
}

type EventName = LiteralUnion<
  | 'tauri://update'
  | 'tauri://update-available'
  | 'tauri://update-install'
  | 'tauri://update-status'
  | 'tauri://resize'
  | 'tauri://move'
  | 'tauri://close-requested'
  | 'tauri://destroyed'
  | 'tauri://focus'
  | 'tauri://blur'
  | 'tauri://scale-change'
  | 'tauri://menu'
  | 'tauri://file-drop'
  | 'tauri://file-drop-hover'
  | 'tauri://file-drop-cancelled',
  string
>

type EventCallback<T> = (event: Event<T>) => void

type UnlistenFn = () => void

/**
 * Unregister the event listener associated with the given id.
 *
 * @ignore
 * @param eventId Event identifier
 * @returns
 */
async function _unlisten(eventId: number): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Event',
    message: {
      cmd: 'unlisten',
      eventId
    }
  })
}

/**
 * Listen to an event from the backend.
 *
 * @param event Event name
 * @param handler Event handler callback
 * @return A promise resolving to a function to unlisten to the event.
 */
async function listen<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return invokeTauriCommand<number>({
    __tauriModule: 'Event',
    message: {
      cmd: 'listen',
      event,
      handler: transformCallback(handler)
    }
  }).then((eventId) => {
    return async () => _unlisten(eventId)
  })
}

/**
 * Listen to an one-off event from the backend.
 *
 * @param event Event name
 * @param handler Event handler callback
 * @returns A promise resolving to a function to unlisten to the event.
 */
async function once<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return listen<T>(event, (eventData) => {
    handler(eventData)
    _unlisten(eventData.id).catch(() => {})
  })
}

/**
 * Emits an event to the backend.
 *
 * @param event Event name
 * @param [payload] Event payload
 * @returns
 */
async function emit(event: string, payload?: string): Promise<void> {
  return emitEvent(event, undefined, payload)
}

export type { Event, EventName, EventCallback, UnlistenFn }

export { listen, once, emit }
