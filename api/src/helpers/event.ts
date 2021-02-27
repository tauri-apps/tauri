import { invoke, transformCallback } from '../tauri'

export interface Event<T> {
  type: string
  payload: T
}

export type EventCallback<T> = (event: Event<T>) => void

async function _listen<T>(
  event: string,
  handler: EventCallback<T>,
  once: boolean
): Promise<void> {
  await invoke({
    __tauriModule: 'Event',
    message: {
      cmd: 'listen',
      event,
      handler: transformCallback(handler, once),
      once
    }
  })
}

/**
 * listen to an event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 */
async function listen<T>(
  event: string,
  handler: EventCallback<T>
): Promise<void> {
  return _listen(event, handler, false)
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
): Promise<void> {
  return _listen(event, handler, true)
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
  await invoke({
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
