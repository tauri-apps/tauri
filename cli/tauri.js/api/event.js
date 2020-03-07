import tauri from './tauri'

/**
 * listen to an event from the backend
 */
function listen (event, handler) {
  tauri.listen(event, handler)
}

function emit (event, payload) {
  tauri.emit(event, payload)
}

export {
  listen,
  emit
}
