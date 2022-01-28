/**
 * Gets the application version.
 *
 * @returns A promise resolving to the application version.
 */
declare function getVersion(): Promise<string>;
/**
 * Gets the application name.
 *
 * @returns A promise resolving to application name.
 */
declare function getName(): Promise<string>;
/**
 * Gets the tauri version.
 *
 * @returns A promise resolving to tauri version.
 */
declare function getTauriVersion(): Promise<string>;
export { getName, getVersion, getTauriVersion };
