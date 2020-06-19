import { invoke, transformCallback } from './tauri'
import { EventCallback } from './types/event'

/**
 * listen to an event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 */
function listen(event: string, handler: EventCallback, once = false) {
  invoke({
    cmd: 'listen',
    event,
    handler: transformCallback(handler, once),
    once
  })
}

/**
 * emits an event to the backend
 *
 * @param event the event name
 * @param [payload] the event payload
 */
function emit(event: string, payload?: string) {
  invoke({
    cmd: 'emit',
    event,
    payload
  })
}

export {
  listen,
  emit
}
