// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
  interface Window {
    __TAURI__: {
      path: {
        __sep: string
        __delimiter: string
      }

      convertFileSrc: (src: string, protocol: string) => string
    }

    __TAURI_IPC__: (message: any) => void
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

class Channel<T = unknown> {
  id: number
  // @ts-expect-error field used by the IPC serializer
  private readonly __TAURI_CHANNEL_MARKER__ = true
  #onmessage: (response: T) => void = () => {
    // no-op
  }

  constructor() {
    this.id = transformCallback((response: T) => {
      this.#onmessage(response)
    })
  }

  set onmessage(handler: (response: T) => void) {
    this.#onmessage = handler
  }

  get onmessage(): (response: T) => void {
    return this.#onmessage
  }

  toJSON(): string {
    return `__CHANNEL__:${this.id}`
  }
}

class PluginListener {
  plugin: string
  event: string
  channelId: number

  constructor(plugin: string, event: string, channelId: number) {
    this.plugin = plugin
    this.event = event
    this.channelId = channelId
  }

  async unregister(): Promise<void> {
    return invoke(`plugin:${this.plugin}|remove_listener`, {
      event: this.event,
      channelId: this.channelId
    })
  }
}

/**
 * Adds a listener to a plugin event.
 *
 * @returns The listener object to stop listening to the events.
 *
 * @since 2.0.0
 */
async function addPluginListener<T>(
  plugin: string,
  event: string,
  cb: (payload: T) => void
): Promise<PluginListener> {
  const handler = new Channel<T>()
  handler.onmessage = cb
  return invoke(`plugin:${plugin}|register_listener`, { event, handler }).then(
    () => new PluginListener(plugin, event, handler.id)
  )
}

/**
 * Command arguments.
 *
 * @since 1.0.0
 */
type InvokeArgs = Record<string, unknown> | number[] | ArrayBuffer | Uint8Array

/**
 * @since 2.0.0
 */
interface InvokeOptions {
  headers: Headers | Record<string, string>
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
 * @param options The request options.
 * @return A promise resolving or rejecting to the backend response.
 *
 * @since 1.0.0
 */
async function invoke<T>(
  cmd: string,
  args: InvokeArgs = {},
  options?: InvokeOptions
): Promise<T> {
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
      payload: args,
      options
    })
  })
}

/**
 * Convert a device file path to an URL that can be loaded by the webview.
 * Note that `asset:` and `http://asset.localhost` must be added to [`tauri.security.csp`](https://tauri.app/v1/api/config/#securityconfig.csp) in `tauri.conf.json`.
 * Example CSP value: `"csp": "default-src 'self' ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost"` to use the asset protocol on image sources.
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
  return window.__TAURI__.convertFileSrc(filePath, protocol)
}

export type { InvokeArgs, InvokeOptions }

export {
  transformCallback,
  Channel,
  PluginListener,
  addPluginListener,
  invoke,
  convertFileSrc
}
