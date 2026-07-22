#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Example launcher: stages the local tauri-ffi build into this binding's
// `_native/` and then runs the in-repo `cargo-tauri` against the example,
// translating the `TAURI_RUNTIME` env var into the `app > runtime` the CLI reads
// at startup.
//
// Staging happens HERE — synchronously, before `cargo-tauri` is spawned — rather
// than in the example's `beforeDevCommand`. That hook also starts the long-running
// frontend dev server, so the CLI has to run it non-blocking, which lets its
// `stage-dev.mjs` race the CLI's own cdylib probe (`tauri dev` resolves the
// per-runtime `libtauri_<runtime>` by asking the bindings package where it is, and
// that fails until staging has linked it into `_native/`). Doing it up front makes
// the library present before the CLI ever looks for it.
//
// The CLI captures `app.runtime` in `BindingsInterface::new` — before
// `beforeDevCommand` runs — so the runtime can't come from that hook; it has to
// be a `--config` merge on the command line, which is what this wrapper injects.
//
// `TAURI_RUNTIME` is the single switch for an example's webview: this wrapper
// stages the matching `libtauri_<runtime>` and forwards the runtime as `--config`
// (deciding which library the CLI loads), so the staged and loaded libraries
// always agree. This lives with the examples, not the CLI — the CLI has no
// `TAURI_RUNTIME` awareness of its own.
//
// Usage (from an example dir, e.g. bindings/node/examples/api):
//   TAURI_RUNTIME=cef node ../../../scripts/dev.mjs [dev|build] [extra cargo-tauri args]
//
// With `TAURI_RUNTIME` unset the wry library is staged and no `--config` is added,
// so the example runs on its own `app.runtime`. Override the CLI binary with
// `TAURI_CLI`.

import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(scriptDir, '../..')

function fail(message) {
  console.error(`error: ${message}`)
  process.exit(1)
}

// Validate up front (mirrors stage-dev.mjs) so a typo fails before we spawn.
const runtime = process.env.TAURI_RUNTIME || 'wry'
if (!['wry', 'cef'].includes(runtime)) {
  fail(`unknown TAURI_RUNTIME '${runtime}' (known: wry, cef)`)
}

// The binding language is the `<lang>` in `bindings/<lang>/examples/<name>` — the
// directory this launcher is always invoked from (see the package.json scripts).
const rel = path.relative(repoRoot, process.cwd()).split(path.sep)
const lang = rel[0] === 'bindings' ? rel[1] : undefined
if (!['node', 'deno', 'python'].includes(lang)) {
  fail(
    `could not determine the binding language from ${process.cwd()} — run this from a bindings/<node|deno|python>/examples/<name> directory`
  )
}

// Python has no node_modules/import-map equivalent: `tauri_ffi` is resolved off
// sys.path. The example's `main.py` puts the package there for itself, but the
// CLI's cdylib probe imports `tauri_ffi` directly (`python -c`, without running
// main.py), so expose the package on PYTHONPATH for the CLI to inherit — the
// probe and the spawned app runner both pick it up. Prepended so a repo checkout
// resolves its own package ahead of any installed one.
if (lang === 'python') {
  const pkgParent = path.join(repoRoot, 'bindings', 'python')
  process.env.PYTHONPATH = process.env.PYTHONPATH
    ? `${pkgParent}${path.delimiter}${process.env.PYTHONPATH}`
    : pkgParent
}

// Stage the matching library (and, for cef, its CEF distribution + helper) into
// this binding's `_native/` before spawning the CLI, so the CLI's cdylib probe and
// the app itself both resolve it. Synchronous, so nothing races the probe;
// stage-dev.mjs is a fast no-op build + relink when nothing changed.
const stage = spawnSync(
  'node',
  [path.join(scriptDir, 'stage-dev.mjs'), '--lang', lang, '--runtime', runtime],
  { stdio: 'inherit', cwd: repoRoot }
)
if (stage.error) fail(stage.error.message)
if (stage.status !== 0) process.exit(stage.status ?? 1)

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
const configArgs = process.env.TAURI_RUNTIME ? ['--config', JSON.stringify({ app: { runtime } })] : []

const args = [...passthrough, ...configArgs]
console.log(
  `  $ ${path.relative(process.cwd(), cli)} ${args.join(' ')}${process.env.TAURI_RUNTIME ? `   (TAURI_RUNTIME=${runtime})` : ''}`
)

// Inherit stdio.
const { status, error } = spawnSync(cli, args, { stdio: 'inherit' })
if (error) fail(error.message)
process.exit(status ?? 0)
