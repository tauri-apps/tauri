import { invoke, transformCallback } from './tauri'

/**
 * Register a global shortcut
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler shortcut handler callback - takes the triggered shortcut as argument
 */
async function register(
  shortcut: string,
  handler: (shortcut: string) => void
): Promise<void> {
  return invoke('tauri', {
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'register',
      shortcut,
      handler: transformCallback(handler)
    }
  })
}

/**
 * Register a collection of global shortcuts
 * @param shortcuts array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler shortcut handler callback - takes the triggered shortcut as argument
 */
async function registerAll(
  shortcuts: string[],
  handler: (shortcut: string) => void
): Promise<void> {
  return invoke('tauri', {
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'registerAll',
      shortcuts,
      handler: transformCallback(handler)
    }
  })
}

/**
 * Determines whether the given shortcut is registered by this application or not.
 *
 * @param shortcuts array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @return {Promise<boolean>} promise resolving to the state
 */
async function isRegistered(shortcut: string): Promise<boolean> {
  return invoke('tauri', {
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'isRegistered',
      shortcut
    }
  })
}

/**
 * Unregister a global shortcut
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 */
async function unregister(shortcut: string): Promise<void> {
  return invoke('tauri', {
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregister',
      shortcut
    }
  })
}

/**
 * Unregisters all shortcuts registered by the application.
 */
async function unregisterAll(): Promise<void> {
  return invoke('tauri', {
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregisterAll'
    }
  })
}

export { register, registerAll, isRegistered, unregister, unregisterAll }
