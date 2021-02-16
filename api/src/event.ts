import { invoke, transformCallback } from './tauri'

export interface Event<T> {
  type: string
  payload: T
}

export type EventCallback<T> = (event: Event<T>) => void

/**
 * listen to an event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 */
async function listen<T>(
  event: string,
  handler: EventCallback<T>,
  once = false
): Promise<void> {
  await await invoke({
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
 * emits an event to the backend
 *
 * @param event the event name
 * @param [payload] the event payload
 */
async function emit(event: string, payload?: string): Promise<void> {
  await invoke({
    __tauriModule: 'Event',
    message: {
      cmd: 'emit',
      event,
      payload
    }
  })
}

export { listen, emit }
