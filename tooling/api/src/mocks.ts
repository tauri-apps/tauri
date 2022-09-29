interface IPCMessage {
  cmd: string
  callback: number
  error: number
  [key: string]: unknown
}

/**
 * Intercepts all IPC requests with the given mock handler.
 *
 * This function can be used when testing tauri frontend applications or when running the frontend in a Node.js context during static site generation.
 *
 * # Examples
 *
 * Testing setup using vitest:
 * ```js
 * import { mockIPC, clearMocks } from "@tauri-apps/api/mocks"
 * import { invoke } from "@tauri-apps/api/tauri"
 *
 * afterEach(() => {
 *    clearMocks()
 * })
 *
 * test("mocked command", () => {
 *  mockIPC((cmd, args) => {
 *   switch (cmd) {
 *     case "add":
 *       return (args.a as number) + (args.b as number);
 *     default:
 *       break;
 *     }
 *  });
 *
 *  expect(invoke('add', { a: 12, b: 15 })).resolves.toBe(27);
 * })
 * ```
 *
 * The callback function can also return a Promise:
 * ```js
 * import { mockIPC, clearMocks } from "@tauri-apps/api/mocks"
 * import { invoke } from "@tauri-apps/api/tauri"
 *
 * afterEach(() => {
 *    clearMocks()
 * })
 *
 * test("mocked command", () => {
 *  mockIPC((cmd, args) => {
 *   if(cmd === "get_data") {
 *    return fetch("https://example.com/data.json")
 *      .then((response) => response.json())
 *   }
 *  });
 *
 *  expect(invoke('get_data')).resolves.toBe({ foo: 'bar' });
 * })
 * ```
 *
 * @since 1.0.0
 */
export function mockIPC(
  cb: (cmd: string, args: Record<string, unknown>) => any | Promise<any>
): void {
  // eslint-disable-next-line @typescript-eslint/no-misused-promises
  window.__TAURI_IPC__ = async ({
    cmd,
    callback,
    error,
    ...args
  }: IPCMessage) => {
    try {
      // @ts-expect-error The function key is dynamic and therefore not typed
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      window[`_${callback}`](await cb(cmd, args))
    } catch (err) {
      // @ts-expect-error The function key is dynamic and therefore not typed
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      window[`_${error}`](err)
    }
  }
}

/**
 * Mocks one or many window labels.
 * In non-tauri context it is required to call this function *before* using the `@tauri-apps/api/window` module.
 *
 * This function only mocks the *presence* of windows,
 * window properties (e.g. width and height) can be mocked like regular IPC calls using the `mockIPC` function.
 *
 * # Examples
 *
 * ```js
 * import { mockWindows } from "@tauri-apps/api/mocks";
 * import { getCurrent } from "@tauri-apps/api/window";
 *
 * mockWindows("main", "second", "third");
 *
 * const win = getCurrent();
 *
 * win.label // "main"
 * ```
 *
 * ```js
 * import { mockWindows } from "@tauri-apps/api/mocks";
 *
 * mockWindows("main", "second", "third");
 *
 * mockIPC((cmd, args) => {
 *  if (cmd === "tauri") {
 *    if (
 *      args?.__tauriModule === "Window" &&
 *      args?.message?.cmd === "manage" &&
 *      args?.message?.data?.cmd?.type === "close"
 *    ) {
 *      console.log('closing window!');
 *    }
 *  }
 * });
 *
 * const { getCurrent } = await import("@tauri-apps/api/window");
 *
 * const win = getCurrent();
 * await win.close(); // this will cause the mocked IPC handler to log to the console.
 * ```
 *
 * @param current Label of window this JavaScript context is running in.
 * @param additionalWindows Label of additional windows the app has.
 *
 * @since 1.0.0
 */
export function mockWindows(
  current: string,
  ...additionalWindows: string[]
): void {
  window.__TAURI_METADATA__ = {
    __windows: [current, ...additionalWindows].map((label) => ({ label })),
    __currentWindow: { label: current }
  }
}

/**
 * Clears mocked functions/data injected by the other functions in this module.
 * When using a test runner that doesn't provide a fresh window object for each test, calling this function will reset tauri specific properties.
 *
 * # Example
 *
 * ```js
 * import { mockWindows, clearMocks } from "@tauri-apps/api/mocks"
 *
 * afterEach(() => {
 *    clearMocks()
 * })
 *
 * test("mocked windows", () => {
 *    mockWindows("main", "second", "third");
 *
 *    expect(window).toHaveProperty("__TAURI_METADATA__")
 * })
 *
 * test("no mocked windows", () => {
 *    expect(window).not.toHaveProperty("__TAURI_METADATA__")
 * })
 * ```
 *
 * @since 1.0.0
 */
export function clearMocks(): void {
  // @ts-expect-error
  delete window.__TAURI_IPC__
  // @ts-expect-error
  delete window.__TAURI_METADATA__
}
