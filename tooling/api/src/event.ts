// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

export interface Event<T> {
  /** Event name */
  event: string
  /** Event identifier used to unlisten */
  id: number
  /** Event payload */
  payload: T
}

export type EventCallback<T> = (event: Event<T>) => void

export type UnlistenFn = () => void

/**
 * Unregister the event listener associated with the given id
 *
 * @param {number} eventId Event identifier
 * @returns {Promise<void>}
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
 * Listen to an event from the backend
 *
 * @param {string} event Event name
 * @param {EventCallback} handler Event handler callback
 * @return {Promise<UnlistenFn>} A promise resolving to a function to unlisten to the event
 */
async function listen<T>(
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
 * Listen to an one-off event from the backend
 *
 * @param {string} event Event name
 * @param {EventCallback<T>} handler Event handler callback
 * @returns {Promise<UnlistenFn>} A promise resolving to a function to unlisten to the event
 */
async function once<T>(
  event: string,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return listen<T>(event, (eventData) => {
    handler(eventData)
    _unlisten(eventData.id).catch(() => {})
  })
}

/**
 * Emits an event to the backend
 *
 * @param {string} event Event name
 * @param {string} [windowLabel] Label of the window to which the event is sent, if null/undefined the event will be sent to all windows
 * @param {string} [payload] Event payload
 * @returns {Promise<void>}
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
