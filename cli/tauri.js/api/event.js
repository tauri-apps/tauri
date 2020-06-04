import tauri from './tauri'

/**
 * The event handler callback
 * @callback EventCallback
 * @param {Object} event
 * @param {String} event.type
 * @param {any} [event.payload]
 */

/**
 * listen to an event from the backend
 * 
 * @param {String} event the event name
 * @param {EventCallback} handler the event handler callback
 */
function listen (event, handler) {
  tauri.listen(event, handler)
}

/**
 * emits an event to the backend
 *
 * @param {String} event the event name
 * @param {String} [payload] the event payload
 */
function emit (event, payload) {
  tauri.emit(event, payload)
}

export {
  listen,
  emit
}
