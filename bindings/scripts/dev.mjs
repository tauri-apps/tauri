#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Example launcher: runs the in-repo `cargo-tauri` against a binding example and
// translates the `TAURI_RUNTIME` env var into the `app > runtime` the CLI reads at
// startup. The CLI captures `app.runtime` in `BindingsInterface::new` — before
// `beforeDevCommand` runs — so the runtime can't come from that hook; it has to
// be a `--config` merge on the command line, which is what this wrapper injects.
//
// `TAURI_RUNTIME` is the single switch for an example's webview: this wrapper forwards
// it as `--config` (deciding which `libtauri_<runtime>` the CLI loads) while the
// inherited env carries it to `stage-dev.mjs` in `beforeDevCommand` (deciding
// which library gets staged), so the two always agree. This lives with the
// examples, not the CLI — the CLI has no `TAURI_RUNTIME` awareness of its own.
//
// Usage (from an example dir, e.g. bindings/node/examples/api):
//   TAURI_RUNTIME=cef node ../../../scripts/dev.mjs [dev|build] [extra cargo-tauri args]
//
// With `TAURI_RUNTIME` unset no `--config` is added and the example runs with its own
// `app.runtime`. Override the CLI binary with `TAURI_CLI`.

import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')

function fail(message) {
  console.error(`error: ${message}`)
  process.exit(1)
}

// Validate up front (mirrors stage-dev.mjs) so a typo fails before we spawn.
const runtime = process.env.TAURI_RUNTIME
if (runtime && !['wry', 'cef'].includes(runtime)) {
  fail(`unknown TAURI_RUNTIME '${runtime}' (known: wry, cef)`)
}

// Locate the in-repo cargo-tauri (built with `cargo build -p tauri-cli`),
// preferring debug over release; `TAURI_CLI` overrides the lookup.
const exe = process.platform === 'win32' ? 'cargo-tauri.exe' : 'cargo-tauri'
const cli =
  process.env.TAURI_CLI ||
  [path.join(repoRoot, 'target/debug', exe), path.join(repoRoot, 'target/release', exe)].find(existsSync)
if (!cli || !existsSync(cli)) {
  fail(`could not find cargo-tauri under ${path.join(repoRoot, 'target')} — build it first: cargo build -p tauri-cli`)
}

// Default to `dev`; forward the subcommand and any extra args straight through.
const passthrough = process.argv.slice(2)
if (passthrough.length === 0) passthrough.push('dev')

// Only fill `app.runtime` when TAURI_RUNTIME is set, leaving an unset run on the
// example's own config. The merge value wins over the config file's field.
const configArgs = runtime ? ['--config', JSON.stringify({ app: { runtime } })] : []

const args = [...passthrough, ...configArgs]
console.log(`  $ ${path.relative(process.cwd(), cli)} ${args.join(' ')}${runtime ? `   (TAURI_RUNTIME=${runtime})` : ''}`)

// Inherit env (so TAURI_RUNTIME reaches stage-dev.mjs in beforeDevCommand) and stdio.
const { status, error } = spawnSync(cli, args, { stdio: 'inherit' })
if (error) fail(error.message)
process.exit(status ?? 0)
