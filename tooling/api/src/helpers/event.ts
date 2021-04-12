import { invokeTauriCommand } from './tauri'

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

export { emit }
