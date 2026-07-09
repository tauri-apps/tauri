// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Main-thread bootstrap. Tauri must own the OS main thread for the app's
// lifetime (a hard macOS requirement), which is also where Deno's event loop
// lives — so `launch()` inverts the layout: your app code runs in a Worker
// with a live event loop, while this thread parks inside the blocking
// `tauri_app_run`. See /ffi-bindings-plan.md §3.

import { cstr, ensureLibrary, open } from './ffi.ts'
import type { Plugin } from './plugin.ts'

export interface LaunchOptions {
  /** tauri.conf.json-shaped configuration. */
  config: unknown
  /** Directory serving app:// assets. */
  assetsDir?: URL | string
  /** Command names handled by the worker. */
  commands?: string[]
  /** Capability definitions; defaults to granting core:default to all windows. */
  capabilities?: (object | string)[]
  /** Plugins created with `definePlugin()`; pass the same objects to
   * `app.plugin()` in the worker so their handlers run there. */
  plugins?: Plugin[]
}

/**
 * Builds the app, spawns `appEntry` as a worker, then blocks this thread in
 * the Tauri event loop until the app exits. Never returns — the process
 * exits with the app's exit code. Use
 * `import { app } from '../../worker.ts'` inside the worker module.
 */
export async function launch(appEntry: URL | string, options: LaunchOptions): Promise<never> {
  const { sym, check, libPath } = open(await ensureLibrary())

  const outBuilder = new BigUint64Array(1)
  check(
    sym.tauri_app_builder_new(
      cstr(JSON.stringify(options.config ?? {})),
      new Uint8Array(outBuilder.buffer)
    ),
    'builder_new'
  )
  const builder = outBuilder[0]

  if (options.assetsDir) {
    const dir =
      options.assetsDir instanceof URL
        ? decodeURIComponent(options.assetsDir.pathname)
        : options.assetsDir
    check(sym.tauri_app_builder_set_assets_dir(builder, cstr(dir)), 'set_assets_dir')
  }
  for (const name of options.commands ?? []) {
    check(sym.tauri_app_builder_register_command(builder, cstr(name)), `register_command(${name})`)
  }
  for (const capability of options.capabilities ?? []) {
    const json = typeof capability === 'string' ? capability : JSON.stringify(capability)
    check(sym.tauri_app_builder_add_capability(builder, cstr(json)), 'add_capability')
  }
  for (const plugin of options.plugins ?? []) {
    const outPlugin = new BigUint64Array(1)
    check(sym.tauri_plugin_new(cstr(plugin.name), new Uint8Array(outPlugin.buffer)), `plugin_new(${plugin.name})`)
    const handle = outPlugin[0]
    if (plugin.script) {
      check(sym.tauri_plugin_set_init_script(handle, cstr(plugin.script)), `plugin_set_init_script(${plugin.name})`)
    }
    for (const command of plugin.commandNames) {
      check(sym.tauri_plugin_register_command(handle, cstr(command)), `plugin_register_command(${plugin.name}|${command})`)
    }
    check(sym.tauri_app_builder_add_plugin(builder, handle), `add_plugin(${plugin.name})`)
  }

  const outApp = new BigUint64Array(1)
  check(sym.tauri_app_build(builder, new Uint8Array(outApp.buffer)), 'app_build')
  const app = outApp[0]

  // Deno workers have no workerData; hand over the context via query params,
  // readable synchronously at module init through self.location.
  const workerUrl = new URL(appEntry instanceof URL ? appEntry.href : appEntry)
  workerUrl.searchParams.set('tauri-app', String(app))
  workerUrl.searchParams.set('tauri-lib', libPath)
  new Worker(workerUrl.href, { type: 'module' })

  const outCode = new Int32Array(1)
  check(sym.tauri_app_run(app, new Uint8Array(outCode.buffer)), 'app_run') // blocks until exit
  Deno.exit(outCode[0])
}
