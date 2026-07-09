// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// A packageable plugin wrapper, mirroring `tauri::plugin::Builder`. A plugin
// bundles a name, an optional JavaScript init script (injected into every
// webview) and a set of commands with their handlers. Ship one from its own
// module/package: this file has no dependencies, so a plugin need not pull in
// the FFI layer.
//
//   import { definePlugin } from '@tauri-apps/deno/plugin'
//   export default definePlugin('greet')
//     .initScript("window.greet = { hello: (n) => window.__TAURI__.core.invoke('plugin:greet|hello', { name: n }) }")
//     .command('hello', ({ name }) => `Hello ${name}!`)
//
// The app passes it to `launch({ plugins: [greet] })` on the main thread and
// applies it with `app.plugin(greet)` in the worker. The `plugin:<name>|<cmd>`
// wire format never leaks into app code.

// deno-lint-ignore no-explicit-any
export type Json = any
export type CommandHandler = (
  payload: Json,
  context: { webview: string }
) => Json | Promise<Json>

/**
 * A Tauri plugin definition. Create with {@linkcode definePlugin}; the methods
 * are chainable. Consumed by `launch()` (metadata) and `app.plugin()` (handlers).
 */
export class Plugin {
  readonly name: string
  script: string | null = null
  readonly handlers: Map<string, CommandHandler> = new Map()

  constructor(name: string) {
    if (!name) throw new TypeError('a plugin needs a name')
    this.name = name
  }

  /** JavaScript injected into every webview before page scripts run. Assign
   * globals to `window`; commonly used to install the plugin's frontend API. */
  initScript(js: string): this {
    this.script = js
    return this
  }

  /** Registers a command: `handler(payload, { webview })`; the return value (or
   * thrown error) resolves/rejects the frontend invoke. */
  command(name: string, handler: CommandHandler): this {
    this.handlers.set(name, handler)
    return this
  }

  /** Command names, for native registration. */
  get commandNames(): string[] {
    return [...this.handlers.keys()]
  }
}

/** Creates a {@linkcode Plugin}. */
export function definePlugin(name: string): Plugin {
  return new Plugin(name)
}
