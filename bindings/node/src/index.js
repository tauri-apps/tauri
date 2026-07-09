// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Main-thread bootstrap. Tauri must own the OS main thread for the app's
// lifetime (a hard macOS requirement), which is also where Node's event loop
// lives — so `launch()` inverts the layout: your app code runs in a
// worker_thread with a live event loop, while this thread parks inside the
// blocking `tauri_app_run`. See /ffi-bindings-plan.md §3.

import { Worker } from 'node:worker_threads'
import { fileURLToPath } from 'node:url'
import { open } from './ffi.js'

/**
 * Builds the app, spawns `appEntry` as a worker, then blocks this thread in
 * the Tauri event loop until the app exits. Never returns — the process
 * exits with the app's exit code.
 *
 * @param {URL | string} appEntry module run inside the worker; use
 *   `import { app } from '@tauri-apps/node/worker'` there.
 * @param {object} options
 * @param {object} options.config tauri.conf.json-shaped configuration
 * @param {URL | string} [options.assetsDir] directory serving app:// assets
 * @param {string[]} [options.commands] command names handled by the worker
 * @param {(object | string)[]} [options.capabilities] capability definitions;
 *   defaults to granting core:default to all windows
 * @param {import('./plugin.js').Plugin[]} [options.plugins] plugins created with
 *   `definePlugin()`; pass the same objects to `app.plugin()` in the worker so
 *   their handlers run there. See '@tauri-apps/node/plugin'.
 */
export function launch(
  appEntry,
  { config, assetsDir, commands = [], capabilities = [], plugins = [] } = {}
) {
  const { api, check, libPath } = open()

  const outBuilder = [0]
  check(api.appBuilderNew(JSON.stringify(config ?? {}), outBuilder), 'builder_new')
  const builder = outBuilder[0]

  if (assetsDir) {
    const dir = assetsDir instanceof URL ? fileURLToPath(assetsDir) : assetsDir
    check(api.appBuilderSetAssetsDir(builder, dir), 'set_assets_dir')
  }
  for (const name of commands) {
    check(api.appBuilderRegisterCommand(builder, name), `register_command(${name})`)
  }
  for (const capability of capabilities) {
    const json = typeof capability === 'string' ? capability : JSON.stringify(capability)
    check(api.appBuilderAddCapability(builder, json), 'add_capability')
  }
  for (const plugin of plugins) {
    const outPlugin = [0]
    check(api.pluginNew(plugin.name, outPlugin), `plugin_new(${plugin.name})`)
    const handle = outPlugin[0]
    if (plugin.script) {
      check(api.pluginSetInitScript(handle, plugin.script), `plugin_set_init_script(${plugin.name})`)
    }
    for (const command of plugin.commandNames) {
      check(api.pluginRegisterCommand(handle, command), `plugin_register_command(${plugin.name}|${command})`)
    }
    check(api.appBuilderAddPlugin(builder, handle), `add_plugin(${plugin.name})`)
  }

  const outApp = [0]
  check(api.appBuild(builder, outApp), 'app_build')
  const app = outApp[0]

  const entry = appEntry instanceof URL ? fileURLToPath(appEntry) : appEntry
  const worker = new Worker(entry, {
    workerData: { app: String(app), libPath }
  })
  // Note: while this thread is blocked in run(), worker events (including
  // 'error') are not dispatched here — the worker logs its own failures.
  worker.unref()

  const outCode = [0]
  check(api.appRun(app, outCode), 'app_run') // blocks until the app exits
  process.exit(outCode[0])
}
