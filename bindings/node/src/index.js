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
import { readFileSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import path from 'node:path'
import { libraryPath, open } from './ffi.js'
import {
  isBundled,
  readEmbedded,
  resolveAssets,
  resolveCapabilities,
  resolveConfig
} from './config.js'

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
 * @param {string[]} [options.protocols] custom URI schemes served by the
 *   worker; handle each with `app.protocol(scheme, handler)` there
 * @param {boolean} [options.pageLoadEvents] deliver 'page-load' events to the
 *   worker (off by default — navigations are frequent)
 * @param {boolean} [options.webviewEvents] deliver 'webview-event' messages
 *   (drag & drop, web content process termination) to the worker
 * @param {boolean} [options.singleInstance] refuse to launch a second instance;
 *   the running app gets a 'single-instance' event with the new argv and cwd
 * @param {number} [options.localhost] serve the frontend over
 *   `http://localhost:<port>` instead of the custom protocol
 * @param {string | {rgba: Uint8Array, width: number, height: number}} [options.windowIcon]
 *   default window icon — a PNG/ICO path, or raw RGBA pixels
 * @param {string | Uint8Array} [options.appIcon] app icon (PNG/ICO path or
 *   encoded bytes) presented by plugins such as notification
 * @param {string | string[]} [options.invokeInitializationScript] JavaScript
 *   appended to the IPC init script, injected into every webview
 * @param {'always' | 'unfocused' | 'never'} [options.deviceEventFilter]
 * @param {boolean} [options.macosDefaultMenu] whether macOS gets the default
 *   application menu when the app declares none (default true)
 */
export function launch(appEntry, options = {}) {
  const {
    commands = [],
    capabilities = [],
    plugins = [],
    protocols = []
  } = options
  const entry = appEntry instanceof URL ? fileURLToPath(appEntry) : appEntry

  // `tauri build` (Node SEA) runs this entry once in trace mode to learn
  // which module launch() would spawn as the worker, so it can bundle it.
  if (!isBundled() && process.env.TAURI_SEA_TRACE) {
    writeFileSync(
      process.env.TAURI_SEA_TRACE,
      JSON.stringify({ entry: path.resolve(entry) })
    )
    process.exit(0)
  }

  // a compiled bundle ignores the TAURI_DEV env override (hermetic in production)
  const dev = options.dev ?? (!isBundled() && process.env.TAURI_DEV === 'true')

  const entryDir = path.dirname(path.resolve(entry))

  // When bundled (SEA), assets/config/capabilities are embedded *inside* the
  // binary as SEA assets; in dev they come from disk. Explicit options win.
  const embeddedConfig = options.config
    ? null
    : readEmbedded('config.json', 'utf8')
  const { config, configDir } = embeddedConfig
    ? { config: JSON.parse(embeddedConfig), configDir: null }
    : resolveConfig(entryDir, options.config)

  const embeddedAssets =
    !options.assetsDir && !options.assetsArchive
      ? readEmbedded('app.assets')
      : null
  const assets = embeddedAssets
    ? {}
    : resolveAssets(
        {
          assetsDir:
            options.assetsDir instanceof URL
              ? fileURLToPath(options.assetsDir)
              : options.assetsDir,
          assetsArchive:
            options.assetsArchive instanceof URL
              ? fileURLToPath(options.assetsArchive)
              : options.assetsArchive
        },
        config,
        configDir
      )

  // `app.runtime` selects which prebuilt tauri-ffi library to load (wry/cef),
  // and which platform package's run-loop addon runApp() loads.
  const { api, check, runApp, libPath } = open(
    libraryPath(config?.app?.runtime),
    config?.app?.runtime
  )

  const outBuilder = [0]
  check(api.appBuilderNew(JSON.stringify(config), outBuilder), 'builder_new')
  const builder = outBuilder[0]

  if (dev) {
    check(api.appBuilderSetDev(builder, true), 'set_dev')
  }
  if (embeddedAssets) {
    check(
      api.appBuilderSetAssetsArchiveBytes(
        builder,
        embeddedAssets,
        embeddedAssets.length
      ),
      'set_assets_archive_bytes'
    )
  } else if (assets.archive) {
    check(
      api.appBuilderSetAssetsArchive(builder, assets.archive),
      'set_assets_archive'
    )
  } else if (assets.dir) {
    check(api.appBuilderSetAssetsDir(builder, assets.dir), 'set_assets_dir')
  }
  for (const name of commands) {
    check(
      api.appBuilderRegisterCommand(builder, name),
      `register_command(${name})`
    )
  }
  for (const scheme of protocols) {
    check(
      api.appBuilderRegisterUriSchemeProtocol(builder, scheme),
      `register_protocol(${scheme})`
    )
  }
  if (options.pageLoadEvents) {
    check(
      api.appBuilderSetPageLoadEvents(builder, true),
      'set_page_load_events'
    )
  }
  if (options.webviewEvents) {
    check(api.appBuilderSetWebviewEvents(builder, true), 'set_webview_events')
  }
  if (options.singleInstance) {
    check(api.appBuilderEnableSingleInstance(builder), 'enable_single_instance')
  }
  if (options.localhost !== undefined) {
    check(
      api.appBuilderEnableLocalhost(builder, options.localhost),
      'enable_localhost'
    )
  }
  if (options.windowIcon !== undefined) {
    const icon = options.windowIcon
    check(
      typeof icon === 'string'
        ? api.appBuilderSetDefaultWindowIconPath(builder, icon)
        : api.appBuilderSetDefaultWindowIcon(
            builder,
            icon.rgba,
            icon.width,
            icon.height
          ),
      'set_default_window_icon'
    )
  }
  if (options.appIcon !== undefined) {
    const bytes =
      typeof options.appIcon === 'string'
        ? readFileSync(options.appIcon)
        : options.appIcon
    check(
      api.appBuilderSetAppIcon(builder, bytes, bytes.length),
      'set_app_icon'
    )
  }
  for (const script of [options.invokeInitializationScript ?? []].flat()) {
    check(
      api.appBuilderAppendInvokeInitializationScript(builder, script),
      'append_invoke_init_script'
    )
  }
  if (options.deviceEventFilter !== undefined) {
    check(
      api.appBuilderSetDeviceEventFilter(builder, options.deviceEventFilter),
      'set_device_event_filter'
    )
  }
  if (options.macosDefaultMenu !== undefined) {
    check(
      api.appBuilderSetMacosDefaultMenu(builder, options.macosDefaultMenu),
      'set_macos_default_menu'
    )
  }
  // Inline capabilities plus those embedded in the binary (SEA) or discovered
  // in a `capabilities/` directory next to the config (dev) — compile-time
  // capability files, read at launch.
  const embeddedCapabilities = readEmbedded('capabilities.json', 'utf8')
  const discoveredCapabilities = embeddedCapabilities
    ? JSON.parse(embeddedCapabilities)
    : resolveCapabilities([configDir, entryDir])
  const allCapabilities = [...capabilities, ...discoveredCapabilities]
  for (const capability of allCapabilities) {
    const json =
      typeof capability === 'string' ? capability : JSON.stringify(capability)
    check(api.appBuilderAddCapability(builder, json), 'add_capability')
  }
  for (const plugin of plugins) {
    const outPlugin = [0]
    check(api.pluginNew(plugin.name, outPlugin), `plugin_new(${plugin.name})`)
    const handle = outPlugin[0]
    if (plugin.script) {
      check(
        api.pluginSetInitScript(handle, plugin.script),
        `plugin_set_init_script(${plugin.name})`
      )
    }
    for (const command of plugin.commandNames) {
      check(
        api.pluginRegisterCommand(handle, command),
        `plugin_register_command(${plugin.name}|${command})`
      )
    }
    check(
      api.appBuilderAddPlugin(builder, handle),
      `add_plugin(${plugin.name})`
    )
  }

  const outApp = [0]
  check(api.appBuild(builder, outApp), 'app_build')
  const app = outApp[0]

  // In a compiled bundle (Node SEA) the worker module can't be loaded from
  // disk — `tauri build` embedded it (bundled with the app code) as the
  // `worker.js` SEA asset, so run it from source instead.
  const workerData = { app: String(app), libPath }
  const worker = isBundled()
    ? new Worker(
        createRequire(import.meta.url)('node:sea').getAsset(
          'worker.js',
          'utf8'
        ),
        {
          eval: true,
          workerData
        }
      )
    : new Worker(entry, { workerData })
  // Note: while this thread is blocked in run(), worker events (including
  // 'error') are not dispatched here — the worker logs its own failures.
  worker.unref()

  process.exit(runApp(app)) // blocks until the app exits
}
