// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Invoke your custom commands.
 *
 * This package is also accessible with `window.__TAURI__.tauri` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

/** @ignore */
declare global {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface Window {
    __TAURI_IPC__: (message: any) => void
    ipc: {
      postMessage: (args: string) => void
    }
  }
}

/** @ignore */
function uid(): number {
  return window.crypto.getRandomValues(new Uint32Array(1))[0]
}

/**
 * Transforms a callback function to a string identifier that can be passed to the backend.
 * The backend uses the identifier to `eval()` the callback.
 *
 * @return A unique identifier associated with the callback function.
 *
 * @since 1.0.0
 */
function transformCallback(
  callback?: (response: any) => void,
  once = false
): number {
  const identifier = uid()
  const prop = `_${identifier}`

  Object.defineProperty(window, prop, {
    value: (result: any) => {
      if (once) {
        Reflect.deleteProperty(window, prop)
      }

      return callback?.(result)
    },
    writable: false,
    configurable: true
  })

  return identifier
}

/**
 * Command arguments.
 *
 * @since 1.0.0
 */
interface InvokeArgs {
  [key: string]: unknown
}

/**
 * Sends a message to the backend.
 * @example
 * ```typescript
 * import { invoke } from '@tauri-apps/api/tauri';
 * await invoke('login', { user: 'tauri', password: 'poiwe3h4r5ip3yrhtew9ty' });
 * ```
 *
 * @param cmd The command name.
 * @param args The optional arguments to pass to the command.
 * @return A promise resolving or rejecting to the backend response.
 *
 * @since 1.0.0
 */
async function invoke<T>(cmd: string, args: InvokeArgs = {}): Promise<T> {
  return new Promise((resolve, reject) => {
    const callback = transformCallback((e: T) => {
      resolve(e)
      Reflect.deleteProperty(window, `_${error}`)
    }, true)
    const error = transformCallback((e) => {
      reject(e)
      Reflect.deleteProperty(window, `_${callback}`)
    }, true)

    window.__TAURI_IPC__({
      cmd,
      callback,
      error,
      ...args
    })
  })
}

/**
 * Convert a device file path to an URL that can be loaded by the webview.
 * Note that `asset:` and `https://asset.localhost` must be added to [`tauri.security.csp`](https://tauri.app/v1/api/config/#securityconfig.csp) in `tauri.conf.json`.
 * Example CSP value: `"csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost"` to use the asset protocol on image sources.
 *
 * Additionally, `asset` must be added to [`tauri.allowlist.protocol`](https://tauri.app/v1/api/config/#allowlistconfig.protocol)
 * in `tauri.conf.json` and its access scope must be defined on the `assetScope` array on the same `protocol` object.
 *
 * @param  filePath The file path.
 * @param  protocol The protocol to use. Defaults to `asset`. You only need to set this when using a custom protocol.
 * @example
 * ```typescript
 * import { appDataDir, join } from '@tauri-apps/api/path';
 * import { convertFileSrc } from '@tauri-apps/api/tauri';
 * const appDataDirPath = await appDataDir();
 * const filePath = await join(appDataDirPath, 'assets/video.mp4');
 * const assetUrl = convertFileSrc(filePath);
 *
 * const video = document.getElementById('my-video');
 * const source = document.createElement('source');
 * source.type = 'video/mp4';
 * source.src = assetUrl;
 * video.appendChild(source);
 * video.load();
 * ```
 *
 * @return the URL that can be used as source on the webview.
 *
 * @since 1.0.0
 */
function convertFileSrc(filePath: string, protocol = 'asset'): string {
  const path = encodeURIComponent(filePath)
  return navigator.userAgent.includes('Windows')
    ? `https://${protocol}.localhost/${path}`
    : `${protocol}://localhost/${path}`
}

export type { InvokeArgs }

export { transformCallback, invoke, convertFileSrc }
