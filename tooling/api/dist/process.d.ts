/**
 * Perform operations on the current process.
 *
 * This package is also accessible with `window.__TAURI__.process` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */
/**
 * Exits immediately with the given `exitCode`.
 *
 * @param exitCode The exit code to use.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function exit(exitCode?: number): Promise<void>;
/**
 * Exits the current instance of the app then relaunches it.
 *
 * @returns A promise indicating the success or failure of the operation.
 */
declare function relaunch(): Promise<void>;
export { exit, relaunch };
