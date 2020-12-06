declare global {
    interface Window {
        __TAURI_INVOKE_HANDLER__: (command: string) => void;
    }
}
/**
 * sends a synchronous command to the backend
 *
 * @param args
 */
declare function invoke(args: any): void;
declare function transformCallback(callback?: (response: any) => void, once?: boolean): string;
/**
 * sends an asynchronous command to the backend
 *
 * @param args
 *
 * @return {Promise<T>} Promise resolving or rejecting to the backend response
 */
declare function promisified<T>(args: any): Promise<T>;
export { invoke, transformCallback, promisified };
