import tauri from './tauri'

/**
 * Shortcut manager
 */
class ShortcutManager {
  constructor() {
    this.handlers = []
  }

  /**
   * register a global shortcut
   * @param {String} shortcut shortcut definition, modifiers and key separated by "+" e.g. Control+Q
   * @param {Function} handler shortcut handler callback
   * @param {Function(String)} onError shortcut error callback
   */
  registerShortcut (shortcut, handler, onError = null) {
    this.handlers.push({
      shortcut,
      handler,
      onError
    })
  }

  /**
   * notifies the backend that you're done registering shortcuts, and it should start listening
   */
  listen () {
    tauri.addShortcuts(this.handlers)
  }
}

export {
  ShortcutManager
}
