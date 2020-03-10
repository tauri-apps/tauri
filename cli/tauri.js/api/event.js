import tauri from './tauri'

/**
 * The event handler callback
 * @callback eventCallback
 * @param {object} event
 * @param {string} event.type
 * @param {any} [event.payload]
 */

/**
 * listen to an event from the backend
 * 
 * @param {string} event the event name
 * @param {eventCallback} handler the event handler callback
 */
function listen (event, handler) {
  tauri.listen(event, handler)
}

/**
 * emits an event to the backend
 *
 * @param {string} event the event name
 * @param {string} [payload] the event payload
 */
function emit (event, payload) {
  tauri.emit(event, payload)
}

export {
  listen,
  emit
}
