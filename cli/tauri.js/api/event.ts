import tauri from './tauri'
import { EventCallback } from './models'

/**
 * listen to an event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 */
function listen (event: string, handler: EventCallback) {
  tauri.listen(event, handler)
}

/**
 * emits an event to the backend
 *
 * @param event the event name
 * @param [payload] the event payload
 */
function emit (event: string, payload?: string) {
  tauri.emit(event, payload)
}

export {
  listen,
  emit
}
