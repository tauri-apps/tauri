// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './tauri'
import { transformCallback } from '../tauri'

export interface Event<T> {
  /// event name.
  event: string
  /// event identifier used to unlisten.
  id: number
  /// event payload.
  payload: T
}

export type EventCallback<T> = (event: Event<T>) => void

export type UnlistenFn = () => void

async function _listen<T>(
  event: string,
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
 * Unregister the event listener associated with the given id.
 *
 * @param {number} eventId the event identifier
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
 * listen to an event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 * @return {Promise<UnlistenFn>} a promise resolving to a function to unlisten to the event.
 */
async function listen<T>(
  event: string,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return _listen(event, handler)
}

/**
 * listen to an one-off event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 */
async function once<T>(
  event: string,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return _listen<T>(event, (eventData) => {
    handler(eventData)
    _unlisten(eventData.id).catch(() => {})
  })
}

/**
 * emits an event to the backend
 *
 * @param event the event name
 * @param [payload] the event payload
 */
async function emit(
  event: string,
  windowLabel?: string,
  payload?: string
): Promise<void> {
  await invokeTauriCommand({
    __tauriModule: 'Event',
    message: {
      cmd: 'emit',
      event,
      windowLabel,
      payload
    }
  })
}

export { listen, once, emit }
