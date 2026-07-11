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
import { writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import path from 'node:path'
import { open } from './ffi.js'
import { bundledResourceDir, isBundled, resolveAssets, resolveCapabilities, resolveConfig } from './config.js'

/**
 * Builds the app, spawns `appEntry` as a worker, then blocks this thread in
 * the Tauri event loop until the app exits. Never returns — the process
 * exits with the app's exit code.
 *
 * @param {URL | string} appEntry module run inside the worker; use
 *   `import { app } from '@tauri-apps/node/worker'` there.
 * @param {object} [options]
 * @param {object} [options.config] tauri.conf.json-shaped configuration;
 *   when omitted, a `tauri.conf.json` next to `appEntry` (or in the working
 *   directory) is used. The `TAURI_CONFIG` environment variable (set by the
 *   Tauri CLI) is deep-merged on top in either case.
 * @param {URL | string} [options.assetsDir] directory serving app:// assets;
 *   defaults to the config's `build.frontendDist` when it is a directory
 * @param {URL | string} [options.assetsArchive] assets archive packed by
 *   `tauri build`; defaults to a `.assets` `build.frontendDist`
 * @param {boolean} [options.dev] dev mode — `WebviewUrl::App` windows load
 *   from `build.devUrl`; defaults to the `TAURI_DEV` environment variable
 *   (set by `tauri dev`)
 * @param {string[]} [options.commands] command names handled by the worker
 * @param {(object | string)[]} [options.capabilities] capability definitions;
 *   merged with any files found in a `capabilities/` directory next to the
 *   config (mirroring a Rust app's compile-time capability discovery). When
 *   none are supplied, core:default is granted to all windows
 * @param {import('./plugin.js').Plugin[]} [options.plugins] plugins created with
 *   `definePlugin()`; pass the same objects to `app.plugin()` in the worker so
 *   their handlers run there. See '@tauri-apps/node/plugin'.
 */
export function launch(appEntry, options = {}) {
  const { commands = [], capabilities = [], plugins = [] } = options
  const entry = appEntry instanceof URL ? fileURLToPath(appEntry) : appEntry

  // `tauri build` (Node SEA) runs this entry once in trace mode to learn
  // which module launch() would spawn as the worker, so it can bundle it.
  if (!isBundled() && process.env.TAURI_SEA_TRACE) {
    writeFileSync(process.env.TAURI_SEA_TRACE, JSON.stringify({ entry: path.resolve(entry) }))
    process.exit(0)
  }

  // a compiled bundle ignores the TAURI_DEV env override (hermetic in production)
  const dev = options.dev ?? (!isBundled() && process.env.TAURI_DEV === 'true')

  const entryDir = path.dirname(path.resolve(entry))
  const { config, configDir } = resolveConfig(entryDir, options.config)
  const assets = resolveAssets(
    {
      assetsDir: options.assetsDir instanceof URL ? fileURLToPath(options.assetsDir) : options.assetsDir,
      assetsArchive:
        options.assetsArchive instanceof URL
          ? fileURLToPath(options.assetsArchive)
          : options.assetsArchive
    },
    config,
    configDir
  )

  const { api, check, libPath } = open()

  const outBuilder = [0]
  check(api.appBuilderNew(JSON.stringify(config), outBuilder), 'builder_new')
  const builder = outBuilder[0]

  if (dev) {
    check(api.appBuilderSetDev(builder, true), 'set_dev')
  }
  if (assets.archive) {
    check(api.appBuilderSetAssetsArchive(builder, assets.archive), 'set_assets_archive')
  } else if (assets.dir) {
    check(api.appBuilderSetAssetsDir(builder, assets.dir), 'set_assets_dir')
  }
  for (const name of commands) {
    check(api.appBuilderRegisterCommand(builder, name), `register_command(${name})`)
  }
  // Inline capabilities plus any discovered in a `capabilities/` directory
  // next to the config (compile-time capability files, read at launch).
  const allCapabilities = [...capabilities, ...resolveCapabilities([bundledResourceDir(), configDir, entryDir])]
  for (const capability of allCapabilities) {
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

  // In a compiled bundle (Node SEA) the worker module can't be loaded from
  // disk — `tauri build` embedded it (bundled with the app code) as the
  // `worker.js` SEA asset, so run it from source instead.
  const workerData = { app: String(app), libPath }
  const worker = isBundled()
    ? new Worker(createRequire(import.meta.url)('node:sea').getAsset('worker.js', 'utf8'), {
        eval: true,
        workerData
      })
    : new Worker(entry, { workerData })
  // Note: while this thread is blocked in run(), worker events (including
  // 'error') are not dispatched here — the worker logs its own failures.
  worker.unref()

  const outCode = [0]
  check(api.appRun(app, outCode), 'app_run') // blocks until the app exits
  process.exit(outCode[0])
}
