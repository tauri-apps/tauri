export declare type ShortcutHandler = (shortcut: string) => void;
/**
 * Register a global shortcut.
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 * @returns
 */
declare function register(shortcut: string, handler: ShortcutHandler): Promise<void>;
/**
 * Register a collection of global shortcuts.
 *
 * @param shortcuts Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 * @returns
 */
declare function registerAll(shortcuts: string[], handler: ShortcutHandler): Promise<void>;
/**
 * Determines whether the given shortcut is registered by this application or not.
 *
 * @param shortcut Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @returns A promise resolving to the state.
 */
declare function isRegistered(shortcut: string): Promise<boolean>;
/**
 * Unregister a global shortcut.
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @returns
 */
declare function unregister(shortcut: string): Promise<void>;
/**
 * Unregisters all shortcuts registered by the application.
 *
 * @returns
 */
declare function unregisterAll(): Promise<void>;
export { register, registerAll, isRegistered, unregister, unregisterAll };
