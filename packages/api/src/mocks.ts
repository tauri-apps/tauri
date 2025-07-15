// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { InvokeArgs, InvokeOptions } from './core'
import { EventName } from './event'

function mockInternals() {
  window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ ?? {}
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ =
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ ?? {}
}

/**
 * Options for `mockIPC`.
 *
 * # Options
 * `shouldMockEvents`: If true, the `listen` and `emit` functions will be mocked, allowing you to test event handling without a real backend.
 * **This will consume any events emitted with the `plugin:event` prefix.**
 *
 * @since 2.7.0
 */
export interface MockIPCOptions {
  shouldMockEvents?: boolean
}

/**
 * Intercepts all IPC requests with the given mock handler.
 *
 * This function can be used when testing tauri frontend applications or when running the frontend in a Node.js context during static site generation.
 *
 * # Examples
 *
 * Testing setup using Vitest:
 * ```ts
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
 * `listen` can also be mocked with direct calls to the `emit` function. This functionality is opt-in via the `shouldMockEvents` option:
 * ```js
 * import { mockIPC, clearMocks } from "@tauri-apps/api/mocks"
 * import { emit, listen } from "@tauri-apps/api/event"
 *
 * afterEach(() => {
 *    clearMocks()
 * })
 *
 * test("mocked event", () => {
 *  mockIPC(() => {}, { shouldMockEvents: true }); // enable event mocking
 *
 *  const eventHandler = vi.fn();
 *  listen('test-event', eventHandler); // typically in component setup or similar
 *
 *  emit('test-event', { foo: 'bar' });
 *  expect(eventHandler).toHaveBeenCalledWith({
 *    event: 'test-event',
 *    payload: { foo: 'bar' }
 *  });
 * })
 * ```
 * `emitTo` is currently **not** supported by this mock implementation.
 *
 * @since 1.0.0
 */
export function mockIPC(
  cb: (cmd: string, payload?: InvokeArgs) => unknown,
  options?: MockIPCOptions
): void {
  mockInternals()

  function isEventPluginInvoke(cmd: string): boolean {
    return cmd.startsWith('plugin:event|')
  }

  function handleEventPlugin(cmd: string, args?: InvokeArgs): unknown {
    switch (cmd) {
      case 'plugin:event|listen':
        return handleListen(args as { event: EventName; handler: number })
      case 'plugin:event|emit':
        return handleEmit(args as { event: EventName; payload?: unknown })
      case 'plugin:event|unlisten':
        return handleRemoveListener(args as { event: EventName; id: number })
    }
  }

  const listeners = new Map<string, number[]>()
  function handleListen(args: { event: EventName; handler: number }) {
    if (!listeners.has(args.event)) {
      listeners.set(args.event, [])
    }
    listeners.get(args.event)!.push(args.handler)
    return args.handler
  }

  function handleEmit(args: { event: EventName; payload?: unknown }) {
    const eventListeners = listeners.get(args.event) || []
    for (const handler of eventListeners) {
      runCallback(handler, args)
    }
    return null
  }
  function handleRemoveListener(args: { event: EventName; id: number }) {
    const eventListeners = listeners.get(args.event)
    if (eventListeners) {
      const index = eventListeners.indexOf(args.id)
      if (index !== -1) {
        eventListeners.splice(index, 1)
      }
    }
  }

  // eslint-disable-next-line @typescript-eslint/require-await
  async function invoke<T>(
    cmd: string,
    args?: InvokeArgs,
    _options?: InvokeOptions
  ): Promise<T> {
    if (options?.shouldMockEvents && isEventPluginInvoke(cmd)) {
      return handleEventPlugin(cmd, args) as T
    }

    return cb(cmd, args) as T
  }

  const callbacks = new Map<number, (data: unknown) => void>()

  function registerCallback<T = unknown>(
    callback?: (response: T) => void,
    once = false
  ) {
    const identifier = window.crypto.getRandomValues(new Uint32Array(1))[0]
    callbacks.set(identifier, (data) => {
      if (once) {
        unregisterCallback(identifier)
      }
      return callback && callback(data as T)
    })
    return identifier
  }

  function unregisterCallback(id: number) {
    callbacks.delete(id)
  }

  function runCallback(id: number, data: unknown) {
    const callback = callbacks.get(id)
    if (callback) {
      callback(data)
    } else {
      // eslint-disable-next-line no-console
      console.warn(
        `[TAURI] Couldn't find callback id ${id}. This might happen when the app is reloaded while Rust is running an asynchronous operation.`
      )
    }
  }

  function unregisterListener(event: EventName, id: number) {
    unregisterCallback(id)
  }

  window.__TAURI_INTERNALS__.invoke = invoke
  window.__TAURI_INTERNALS__.transformCallback = registerCallback
  window.__TAURI_INTERNALS__.unregisterCallback = unregisterCallback
  window.__TAURI_INTERNALS__.runCallback = runCallback
  window.__TAURI_INTERNALS__.callbacks = callbacks
  window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener =
    unregisterListener
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
 * import { getCurrentWindow } from "@tauri-apps/api/window";
 *
 * mockWindows("main", "second", "third");
 *
 * const win = getCurrentWindow();
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
 *
 * @since 1.0.0
 */
export function mockWindows(
  current: string,
  ..._additionalWindows: string[]
): void {
  mockInternals()
  window.__TAURI_INTERNALS__.metadata = {
    currentWindow: { label: current },
    currentWebview: { windowLabel: current, label: current }
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
  mockInternals()
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

  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.invoke
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.transformCallback
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.unregisterCallback
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.runCallback
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.callbacks
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.convertFileSrc
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_INTERNALS__.metadata

  if (typeof window.__TAURI_EVENT_PLUGIN_INTERNALS__ !== 'object') {
    return
  }
  // @ts-expect-error "The operand of a 'delete' operator must be optional." does not matter in this case
  delete window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener
}
