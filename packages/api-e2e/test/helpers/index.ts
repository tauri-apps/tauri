// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { browser } from '@wdio/globals'
import type * as TauriApi from '@tauri-apps/api'

/** The full `@tauri-apps/api` surface, as exposed on `window.__TAURI__`. */
export type Api = typeof TauriApi

type PageOutcome<T> =
  | { ok: true; value: T }
  | { ok: false; error: string; stack?: string }

/** Thrown when the function passed to {@link tauri} rejects inside the webview. */
export class TauriPageError extends Error {
  pageStack?: string
  constructor(message: string, pageStack?: string) {
    super(message)
    this.name = 'TauriPageError'
    this.pageStack = pageStack
  }
}

/**
 * Runs `fn` inside the app's webview with `window.__TAURI__` as its first argument
 * and resolves with its (JSON-serializable) return value.
 *
 * `fn` is serialized with `Function.prototype.toString`, so it **cannot close over
 * anything in the spec module** — every value it needs must be passed through `args`,
 * and it may only reference `api`, those args, and browser globals (`window`,
 * `document`, `setTimeout`, `Promise`, ...).
 *
 * @example
 * const name = await tauri((api) => api.app.getName())
 * const sum = await tauri((api, a, b) => a + b, 2, 3)
 */
export async function tauri<R, A extends unknown[]>(
  fn: (api: Api, ...args: A) => R,
  ...args: A
): Promise<Awaited<R>> {
  // A string body (rather than passing `fn` directly) keeps this working across
  // both the classic and bidi WebDriver protocols and avoids any in-page eval of
  // our own — the driver injects this script itself, which is exempt from the
  // app's CSP. `executeAsync` is used because promise support in `execute` is not
  // uniform across the platform drivers tauri-driver proxies to.
  const script = `
    var done = arguments[arguments.length - 1];
    var args = Array.prototype.slice.call(arguments, 0, arguments.length - 1);
    var fn = (${fn.toString()});
    Promise.resolve()
      .then(function () { return fn.apply(null, [window.__TAURI__].concat(args)); })
      .then(
        function (value) { done({ ok: true, value: value === undefined ? null : value }); },
        function (error) {
          done({
            ok: false,
            error: error instanceof Error ? error.message : String(error),
            stack: error instanceof Error ? error.stack : undefined
          });
        }
      );
  `
  const outcome = (await browser.executeAsync(script, ...args)) as PageOutcome<
    Awaited<R>
  > | null
  if (!outcome || typeof outcome !== 'object' || !('ok' in outcome)) {
    throw new Error(
      `tauri() bridge returned an unexpected value: ${JSON.stringify(outcome)}`
    )
  }
  if (!outcome.ok) {
    throw new TauriPageError(outcome.error, outcome.stack)
  }
  return outcome.value
}

/**
 * Asserts that the page-side call rejects and returns the rejection message,
 * so specs can assert on it. Throws if the call unexpectedly resolves.
 */
export async function tauriError<A extends unknown[]>(
  fn: (api: Api, ...args: A) => unknown,
  ...args: A
): Promise<string> {
  try {
    await tauri(fn, ...args)
  } catch (error) {
    if (error instanceof TauriPageError) {
      return error.message
    }
    // Some platform drivers (notably the Linux WebKitWebDriver) surface a
    // page-side `invoke` rejection as a WebDriver-level error on the
    // `execute/async` command instead of letting the in-page bridge report it
    // as an `{ ok: false }` outcome. Fall back to that error's message so the
    // backend rejection is still assertable. This is safe for error-path specs:
    // they match the message against an expected pattern, so a genuine driver
    // failure (whose message won't match) still fails the test.
    if (error instanceof Error) {
      return error.message
    }
    throw error
  }
  throw new Error('expected the API call to reject, but it resolved')
}

/**
 * Polls `check` until it returns without throwing or `timeout` elapses.
 * Use for state that a window manager applies asynchronously (focus, minimize, ...).
 */
export async function eventually<T>(
  check: () => T | Promise<T>,
  {
    timeout = 10_000,
    interval = 250
  }: { timeout?: number; interval?: number } = {}
): Promise<T> {
  const deadline = Date.now() + timeout
  let lastError: unknown
  for (;;) {
    try {
      return await check()
    } catch (error) {
      lastError = error
    }
    if (Date.now() > deadline) {
      throw lastError instanceof Error
        ? lastError
        : new Error(String(lastError))
    }
    await new Promise((resolve) => setTimeout(resolve, interval))
  }
}

const skippedModules = (process.env.E2E_SKIP ?? '')
  .split(',')
  .map((entry) => entry.trim())
  .filter(Boolean)

/**
 * `describe` wrapper keyed by API module name. Any module listed in the
 * comma-separated `E2E_SKIP` env var (e.g. `E2E_SKIP=tray,menu`) is skipped.
 */
export function describeApi(module: string, fn: () => void): void {
  const title = `@tauri-apps/api/${module}`
  if (skippedModules.includes(module)) {
    describe.skip(title, fn)
  } else {
    describe(title, fn)
  }
}

/**
 * `it` for assertions that depend on a real window manager (minimize, maximize,
 * focus, ...). Skipped entirely when `E2E_SKIP_WM` is set (e.g. bare headless CI).
 */
export function itWm(title: string, fn: () => void | Promise<void>): void {
  if (process.env.E2E_SKIP_WM) {
    it.skip(title, fn)
  } else {
    it(title, fn)
  }
}
