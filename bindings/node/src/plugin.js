// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// A packageable plugin wrapper, mirroring `tauri::plugin::Builder`. A plugin
// bundles a name, an optional JavaScript init script (injected into every
// webview) and a set of commands with their handlers. Ship one from its own
// npm package: this module has no dependencies, so a plugin package need not
// pull in koffi or the FFI layer.
//
//   // @acme/tauri-plugin-greet
//   import { definePlugin } from '@tauri-apps/node/plugin'
//   export default definePlugin('greet')
//     .initScript("window.greet = { hello: (n) => window.__TAURI__.core.invoke('plugin:greet|hello', { name: n }) }")
//     .command('hello', ({ name }) => `Hello ${name}!`)
//
// The app passes it to `launch({ plugins: [greet] })` on the main thread and
// applies it with `app.plugin(greet)` in the worker. The `plugin:<name>|<cmd>`
// wire format never leaks into app code — the frontend calls the API the init
// script installs, and handlers are dispatched by the plugin object.

/**
 * A Tauri plugin definition. Create with {@link definePlugin}; the methods are
 * chainable. Consumed by `launch()` (metadata) and `app.plugin()` (handlers).
 */
export class Plugin {
  /** @param {string} name plugin name; commands are `plugin:<name>|<command>`. */
  constructor(name) {
    if (!name || typeof name !== 'string') throw new TypeError('a plugin needs a name')
    this.name = name
    /** @type {string | null} */
    this.script = null
    /** @type {Map<string, Function>} command name -> handler */
    this.handlers = new Map()
  }

  /** JavaScript injected into every webview before page scripts run. Assign
   * globals to `window`; commonly used to install the plugin's frontend API. */
  initScript(js) {
    this.script = js
    return this
  }

  /** Registers a command: `handler(payload, { webview })`; the return value (or
   * thrown error) resolves/rejects the frontend invoke. */
  command(name, handler) {
    this.handlers.set(name, handler)
    return this
  }

  /** Command names, for native registration. */
  get commandNames() {
    return [...this.handlers.keys()]
  }
}

/** Creates a {@link Plugin}. */
export function definePlugin(name) {
  return new Plugin(name)
}
