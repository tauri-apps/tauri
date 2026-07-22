#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Stages the local tauri-ffi build into a binding package's `_native/` directory
// so an in-repo `tauri dev`/`tauri build` resolves the cdylib, CEF distribution,
// subprocess helper and run-loop addon exactly the way a published app resolves
// its installed platform package — no `target/` discovery, no wry-vs-cef probing.
// The binding examples run it from `beforeDevCommand`/`beforeBuildCommand`.
//
// Usage:
//   node bindings/scripts/stage-dev.mjs --lang <node|deno|python> [--runtime <wry|cef>] [--profile <debug|release>]
//
// The `TAURI_RUNTIME` env var overrides `--runtime`, so an example can be staged for a
// different webview without touching its config: `TAURI_RUNTIME=cef cargo tauri dev`.
//
// It (re)builds `tauri-ffi` for the runtime, links the artifacts into
// `<binding>/_native/`, and for cef also links the CEF distribution `cef-dll-sys`
// staged next to the build output and builds+copies the `tauri-cef-helper`. Big
// artifacts are symlinked (they track the freshly built `target/` files and cost
// no copy); the helper is copied because CEF resolves its sibling libcef through
// the executable's own `$ORIGIN`, which must be the `_native/` dir. On Windows,
// where symlinks need privileges, everything is copied instead.

import { execFileSync } from 'node:child_process'
import { copyFileSync, existsSync, lstatSync, mkdirSync, rmSync, symlinkSync } from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')

// The CEF distribution files a cef app needs beside libcef, mirroring the
// bundler's list and `stage_cef_runtime` in the CLI. `cef-dll-sys` stages these
// next to the tauri-ffi build output when it is built with `--features runtime-cef`.
const CEF_LINUX_FILES = [
  'libcef.so',
  'icudtl.dat',
  'v8_context_snapshot.bin',
  'chrome_100_percent.pak',
  'chrome_200_percent.pak',
  'resources.pak',
  'libEGL.so',
  'libGLESv2.so',
  'libvk_swiftshader.so',
  'vk_swiftshader_icd.json',
  'libvulkan.so.1',
  'chrome-sandbox'
]
// Only en-US is staged, matching what the bundler ships for a Rust cef app.
const CEF_LOCALES = ['en-US.pak', 'en-US_FEMININE.pak', 'en-US_MASCULINE.pak', 'en-US_NEUTER.pak']

// ---------------------------------------------------------------------------

const argv = process.argv.slice(2)
function flag(name, fallback) {
  const i = argv.indexOf(name)
  return i === -1 ? fallback : argv[i + 1]
}
function fail(message) {
  console.error(`error: ${message}`)
  process.exit(1)
}

const lang = flag('--lang')
// `TAURI_RUNTIME` (env) overrides the `--runtime` flag so an example can be run against
// a different webview without editing its config, e.g. `TAURI_RUNTIME=cef cargo tauri dev`.
// This is example-only: it lives in the staging script the examples call, not the CLI.
const runtime = process.env.TAURI_RUNTIME || flag('--runtime', 'wry')
const profile = flag('--profile', 'debug')

if (!['node', 'deno', 'python'].includes(lang)) {
  fail('usage: stage-dev.mjs --lang <node|deno|python> [--runtime <wry|cef>] [--profile <debug|release>]')
}
if (!['wry', 'cef'].includes(runtime)) fail(`unknown runtime '${runtime}' (known: wry, cef)`)
if (!['debug', 'release'].includes(profile)) fail(`unknown profile '${profile}' (known: debug, release)`)

const isCef = runtime === 'cef'
if (isCef && process.platform !== 'linux') {
  // Windows needs its own CEF file list and macOS needs the framework staged into
  // an .app — neither is wired up (matching the CLI's `stage_cef_runtime`).
  fail(`cef local dev staging is only supported on linux for now (this is ${process.platform})`)
}

// The binding package's `_native/` — the same directory the loaders resolve an
// installed package's library from (Python's is literally the wheel layout).
const nativeDir = {
  node: path.join(repoRoot, 'bindings/node/_native'),
  deno: path.join(repoRoot, 'bindings/deno/_native'),
  python: path.join(repoRoot, 'bindings/python/tauri_ffi/_native')
}[lang]

/** The `tauri_<base>` library file name for this host (libtauri_ffi.so, …). */
function libFileName(base) {
  const spec = { darwin: ['lib', '.dylib'], linux: ['lib', '.so'], win32: ['', '.dll'] }[process.platform]
  if (!spec) fail(`unsupported platform: ${process.platform}`)
  return `${spec[0]}tauri_${base}${spec[1]}`
}

/** `<cache>/tauri-cef`, matching the CLI's `default_cef_path`, so a build here
 * reuses the CEF the CLI/library build already downloaded. */
function defaultCefPath() {
  const home = os.homedir()
  const base =
    process.platform === 'darwin'
      ? path.join(home, 'Library', 'Caches')
      : process.platform === 'win32'
        ? process.env.LOCALAPPDATA || path.join(home, 'AppData', 'Local')
        : process.env.XDG_CACHE_HOME || path.join(home, '.cache')
  return path.join(base, 'tauri-cef')
}

const env = { ...process.env }
if (isCef && !env.CEF_PATH) env.CEF_PATH = defaultCefPath()

function run(cmd, args, opts = {}) {
  console.log(`  $ ${cmd} ${args.join(' ')}`)
  execFileSync(cmd, args, { stdio: 'inherit', cwd: repoRoot, env, ...opts })
}

function rmDest(dest) {
  try {
    if (lstatSync(dest)) rmSync(dest, { recursive: true, force: true })
  } catch {
    // does not exist
  }
}

/** Symlink `src` into `dest` (copy on Windows), for artifacts that can track the
 * `target/` file directly. Fails if `src` is missing — the build should have
 * produced it. */
function link(src, dest) {
  if (!existsSync(src)) fail(`expected ${src} but it does not exist — the build did not produce it`)
  mkdirSync(path.dirname(dest), { recursive: true })
  rmDest(dest)
  if (process.platform === 'win32') copyFileSync(src, dest)
  else symlinkSync(src, dest)
}

/** Copy `src` to `dest` (a real file, not a symlink). Used for the helper, whose
 * `$ORIGIN` libcef lookup must resolve inside `_native/`. */
function copy(src, dest) {
  if (!existsSync(src)) fail(`expected ${src} but it does not exist — the build did not produce it`)
  mkdirSync(path.dirname(dest), { recursive: true })
  rmDest(dest)
  copyFileSync(src, dest)
}

// ---------------------------------------------------------------------------

const profileArgs = profile === 'release' ? ['--release'] : []
const targetDir = path.join(repoRoot, 'target', profile)

console.log(`staging ${runtime} tauri-ffi (${profile}) into ${path.relative(repoRoot, nativeDir)} for ${lang}`)

// 1. build the cdylib (cargo emits the crate's own name for any runtime feature).
run('cargo', ['build', ...profileArgs, '-p', 'tauri-ffi', '--no-default-features', '--features', `runtime-${runtime}`])

// Start from a clean package dir so switching runtimes never leaves a stale
// library/helper behind, mirroring a freshly installed platform package.
rmDest(nativeDir)
mkdirSync(nativeDir, { recursive: true })

// 2. the library, under its distribution name (libtauri_<runtime>).
link(path.join(targetDir, libFileName('ffi')), path.join(nativeDir, libFileName(runtime)))

// 3. cef: the CEF distribution next to the library, plus the subprocess helper.
if (isCef) {
  for (const name of CEF_LINUX_FILES) link(path.join(targetDir, name), path.join(nativeDir, name))
  for (const name of CEF_LOCALES) {
    link(path.join(targetDir, 'locales', name), path.join(nativeDir, 'locales', name))
  }

  // Build the helper out-of-workspace (its own `[workspace]`), redirecting its
  // target dir under the repo's ignored `target/` so the checkout stays clean.
  // The `sandbox` default feature is macOS-only; drop it on linux.
  const helperTargetDir = path.join(repoRoot, 'target', 'cef-helper')
  run(
    'cargo',
    ['build', ...profileArgs, '--manifest-path', 'cef-helper/Cargo.toml', '--no-default-features'],
    { env: { ...env, CARGO_TARGET_DIR: helperTargetDir } }
  )
  copy(path.join(helperTargetDir, profile, 'tauri-cef-helper'), path.join(nativeDir, 'tauri-cef-helper'))
}

// 4. node: the run-loop addon the Node bindings drive the event loop through.
if (lang === 'node') {
  run('node', ['bindings/node/native/build.mjs', '--out', path.join(nativeDir, 'tauri_node.node')])
}

console.log(`staged ${runtime} tauri-ffi into ${path.relative(repoRoot, nativeDir)}`)
