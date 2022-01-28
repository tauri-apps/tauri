/**
 * Invoke your custom commands.
 *
 * This package is also accessible with `window.__TAURI__.tauri` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */
/** @ignore */
declare global {
    interface Window {
        __TAURI_POST_MESSAGE__: (command: string, args?: {
            [key: string]: unknown;
        }) => void;
    }
}
/**
 * Transforms a callback function to a string identifier that can be passed to the backend.
 * The backend uses the identifier to `eval()` the callback.
 *
 * @return A unique identifier associated with the callback function.
 */
declare function transformCallback(callback?: (response: any) => void, once?: boolean): string;
/** Command arguments. */
interface InvokeArgs {
    [key: string]: unknown;
}
/**
 * Sends a message to the backend.
 *
 * @param cmd The command name.
 * @param args The optional arguments to pass to the command.
 * @return A promise resolving or rejecting to the backend response.
 */
declare function invoke<T>(cmd: string, args?: InvokeArgs): Promise<T>;
/**
 * Convert a device file path to an URL that can be loaded by the webview.
 * Note that `asset:` must be allowed on the `csp` value configured on `tauri.conf.json`.
 *
 * @param  filePath the file path.
 *
 * @return the URL that can be used as source on the webview
 */
declare function convertFileSrc(filePath: string): string;
export type { InvokeArgs };
export { transformCallback, invoke, convertFileSrc };
