import { invoke, transformCallback } from './tauri'

interface ShortcutHandler {
  shortcut: string
  handler: () => void
  onError?: (error: string) => void
}

/**
 * Shortcut manager
 */
class ShortcutManager {
  handlers: ShortcutHandler[]

  constructor() {
    this.handlers = []
  }

  /**
   * register a global shortcut
   * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. Control+Q
   * @param handler shortcut handler callback
   * @param onError shortcut error callback
   */
  registerShortcut(shortcut: string, handler: () => void, onError?: (error: string) => void) {
    this.handlers.push({
      shortcut,
      handler,
      onError
    })
  }

  /**
   * notifies the backend that you're done registering shortcuts, and it should start listening
   */
  listen() {
    invoke({
      cmd: 'addShortcuts',
      shortcutHandlers: this.handlers.map(handler => {
        return {
          shortcut: handler.shortcut,
          callback: transformCallback(handler.handler),
          error: handler.onError ? transformCallback(handler.onError) : null
        }
      })
    })
  }
}

export {
  ShortcutManager
}
