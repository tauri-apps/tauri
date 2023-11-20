// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
 * import { invoke } from "@tauri-apps/api/core"
 *
 * afterEach(() => {
 *    clearMocks()
 * })
 *
 * test("mocked command", () => {
 *  mockIPC((cmd, payload) => {
 *   switch (cmd) {
 *     case "add":
 *       return (payload.a as number) + (payload.b as number);
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
 * import { invoke } from "@tauri-apps/api/core"
 *
 * afterEach(() => {
 *    clearMocks()
 * })
 *
 * test("mocked command", () => {
 *  mockIPC((cmd, payload) => {
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
  cb: (cmd: string, payload: Record<string, unknown>) => unknown
): void {
  // eslint-disable-next-line @typescript-eslint/no-misused-promises
  window.__TAURI_INTERNALS__.ipc = async ({
    cmd,
    callback,
    error,
    payload
  }: IPCMessage) => {
    try {
      // @ts-expect-error The function key is dynamic and therefore not typed
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      window[`_${callback}`](await cb(cmd, payload))
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
 *  if (cmd === "plugin:event|emit") {
 *    console.log('emit event', args?.event, args?.payload);
 *  }
 * });
 *
 * const { emit } = await import("@tauri-apps/api/event");
 * await emit('loaded'); // this will cause the mocked IPC handler to log to the console.
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
  window.__TAURI_INTERNALS__.metadata = {
    windows: [current, ...additionalWindows].map((label) => ({ label })),
    currentWindow: { label: current }
  }
}

/**
 * Mock `convertFileSrc` function
 *
 *
 * @example
 * ```js
 * import { mockConvertFileSrc } from "@tauri-apps/api/mocks";
 * import { convertFileSrc } from "@tauri-apps/api/core";
 *
 * mockConvertFileSrc("windows")
 *
 * const url = convertFileSrc("C:\\Users\\user\\file.txt")
 * ```
 *
 * @param osName The operating system to mock, can be one of linux, macos, or windows
 *
 * @since 1.6.0
 */
export function mockConvertFileSrc(osName: string): void {
  window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ ?? {}
  window.__TAURI_INTERNALS__.convertFileSrc = function (
    filePath,
    protocol = 'asset'
  ) {
    const path = encodeURIComponent(filePath)
    return osName === 'windows'
      ? `http://${protocol}.localhost/${path}`
      : `${protocol}://localhost/${path}`
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
 *    expect(window.__TAURI_INTERNALS__).toHaveProperty("metadata")
 * })
 *
 * test("no mocked windows", () => {
 *    expect(window.__TAURI_INTERNALS__).not.toHaveProperty("metadata")
 * })
 * ```
 *
 * @since 1.0.0
 */
export function clearMocks(): void {
  if (typeof window.__TAURI_INTERNALS__ !== 'object') {
    return
  }

  // @ts-expect-error "The operand of a 'delete' operator must be optional' does not matter in this case
  delete window.__TAURI_INTERNALS__.convertFileSrc
  // @ts-expect-error "The operand of a 'delete' operator must be optional' does not matter in this case
  delete window.__TAURI_INTERNALS__.ipc
  // @ts-expect-error "The operand of a 'delete' operator must be optional' does not matter in this case
  delete window.__TAURI_INTERNALS__.metadata
}
