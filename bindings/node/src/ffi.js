// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Runtime layer over the generated koffi declarations (./ffi-decls.js, from
// crates/tauri-ffi/api-manifest.json). Only hand-written sugar lives here:
// library discovery, error checking and the async event-queue poll.

import { existsSync } from 'node:fs'
import { createRequire } from 'node:module'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import koffi from 'koffi'
import { bundledResourceDir, isBundled } from './config.js'
import { CODES, declare } from './ffi-decls.js'

export { CODES }

const require = createRequire(import.meta.url)

// The runtime the app loads a library for, when none is given. Comes from the
// resolved config's `app.runtime` (passed by launch()); this default only
// applies to a bare open() (e.g. the smoke test). Overridable via TAURI_FFI_RUNTIME.
const DEFAULT_RUNTIME = 'wry'

/** The Node-API addon carrying the run loop — see {@link loadRunAddon}. */
const RUN_ADDON_NAME = 'tauri_node.node'

/** Which runtime's prebuilt library to load (wry, cef, …). */
export function ffiRuntime() {
  return process.env.TAURI_FFI_RUNTIME || DEFAULT_RUNTIME
}

// Every runtime's library shares the same tauri_ffi C ABI, so it differs only by
// file name: `tauri_<base>` (libtauri_wry.so, tauri_cef.dll, …). `ffi` is the
// crate's own output name, used for the local dev build (cargo emits it
// regardless of the selected runtime feature).
function platformLibraryName(base) {
  const spec = { darwin: ['lib', '.dylib'], linux: ['lib', '.so'], win32: ['', '.dll'] }[
    process.platform
  ]
  if (!spec) throw new Error(`unsupported platform: ${process.platform}`)
  return `${spec[0]}tauri_${base}${spec[1]}`
}

/**
 * This package's dev staging directory (`_native/`), populated by
 * `bindings/scripts/stage-dev.mjs` with the freshly built cdylib, cef
 * distribution, subprocess helper and run-loop addon. It is the in-repo stand-in
 * for an installed platform package — a checkout resolves its assets from here
 * exactly the way a published app resolves them from the platform package, so no
 * `target/` discovery or wry-vs-cef probing is needed. A published base package
 * carries only `src`, so this directory is absent there and simply skipped.
 */
function baseNativeDir() {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '_native')
}

export function libraryPath(runtime = ffiRuntime()) {
  // a compiled bundle ignores the TAURI_FFI_LIB env override — it must load
  // only its own bundled cdylib, never an arbitrary library named by the env
  if (!isBundled() && process.env.TAURI_FFI_LIB) return process.env.TAURI_FFI_LIB
  const distName = platformLibraryName(runtime)
  // a compiled bundle loads the per-runtime cdylib the CLI staged in its
  // resource dir (selected there from `app.runtime`)
  const resourceDir = bundledResourceDir()
  if (resourceDir) {
    const bundled = path.join(resourceDir, distName)
    if (existsSync(bundled)) return bundled
    throw new Error(`bundled ${distName} not found in ${resourceDir}`)
  }
  // Dev checkout: the library staged into this package's `_native/`.
  const staged = path.join(baseNativeDir(), distName)
  if (existsSync(staged)) return staged
  // Installed platform package (optionalDependencies of the runtime's base package).
  try {
    return require.resolve(`@tauri-apps/node-${runtime}-${process.platform}-${process.arch}/${distName}`)
  } catch {
    // not installed
  }
  throw new Error(
    `tauri_ffi ${runtime} library not found — in the repo run \`node bindings/scripts/stage-dev.mjs --lang node --runtime ${runtime}\`, otherwise install the app's dependencies or set TAURI_FFI_LIB`
  )
}

function cefLibraryName() {
  return { darwin: 'libcef.dylib', linux: 'libcef.so', win32: 'libcef.dll' }[process.platform] ?? 'libcef.so'
}

function cefHelperName() {
  return process.platform === 'win32' ? 'tauri-cef-helper.exe' : 'tauri-cef-helper'
}

/**
 * Path to the Node-API addon that owns the blocking run loop, or `null` when it
 * was neither shipped nor built. See `native/tauri_node.c`: koffi runs every
 * call on a private stack, and WebKit aborts if its stack is not the thread's
 * real one — so the loop cannot go through koffi.
 *
 * Resolved like the library itself: from the bundle's resource dir (where
 * `tauri build` stages it), else this package's dev `_native/`, else the
 * installed platform package. The Tauri CLI calls this to find the addon to stage.
 */
export function runAddonPath(runtime = ffiRuntime()) {
  const resourceDir = bundledResourceDir()
  if (resourceDir) {
    const bundled = path.join(resourceDir, RUN_ADDON_NAME)
    return existsSync(bundled) ? bundled : null
  }
  const staged = path.join(baseNativeDir(), RUN_ADDON_NAME)
  if (existsSync(staged)) return staged
  try {
    const installed = require.resolve(
      `@tauri-apps/node-${runtime}-${process.platform}-${process.arch}/${RUN_ADDON_NAME}`
    )
    if (existsSync(installed)) return installed
  } catch {
    // not installed
  }
  return null
}

/** Loads the run-loop addon, or `null` when there is none. */
function loadRunAddon(runtime) {
  const file = runAddonPath(runtime)
  if (!file) return null
  const mod = { exports: {} }
  process.dlopen(mod, file)
  return mod.exports
}

/**
 * Preloads the `libcef` a cef library links so that library's own (rpath-less)
 * dependency on it resolves. The dynamic loader reads its search path
 * (LD_LIBRARY_PATH/PATH) once at process start, so it can't be pointed at the
 * directory from in here; loading libcef by absolute path instead puts it in the
 * process's link map and the subsequent load resolves the dependency by name.
 *
 * It looks next to the library first (a repo checkout / a bundle stage libcef
 * there), then — in dev only — at the `TAURI_CEF_PATH` the Tauri CLI sets to the
 * shared CEF cache it downloaded, since an installed platform package ships the
 * library without the ~1.4GB CEF runtime. A bundle is hermetic and ignores it.
 */
function preloadCef(dir) {
  const dirs = [dir]
  if (!isBundled() && process.env.TAURI_CEF_PATH) dirs.push(process.env.TAURI_CEF_PATH)
  for (const d of dirs) {
    const libcef = path.join(d, cefLibraryName())
    if (existsSync(libcef)) {
      koffi.load(libcef)
      return true
    }
  }
  return false
}

export function open(libPath = libraryPath(), runtime = ffiRuntime()) {
  const dir = path.dirname(libPath)
  // CEF is multi-process: it launches renderer/GPU/utility subprocesses by
  // executing a helper program. A Node host can't act as one, so point CEF at
  // the helper staged next to the library. A wry library never reads this.
  // A compiled bundle must point at its own staged helper (hermetic, like
  // TAURI_FFI_LIB); in dev an explicit env value wins.
  const helper = path.join(dir, cefHelperName())
  if (existsSync(helper) && (isBundled() || !process.env.TAURI_CEF_SUBPROCESS_PATH)) {
    process.env.TAURI_CEF_SUBPROCESS_PATH = helper
  }
  let lib
  try {
    lib = koffi.load(libPath)
  } catch (error) {
    // A cef library links libcef; preload it and retry. If the preload finds no
    // libcef (a wry library that failed for another reason) or itself throws,
    // report the original load error, not the preload's.
    let preloaded = false
    try {
      preloaded = preloadCef(dir)
    } catch {
      // keep the original error
    }
    if (!preloaded) throw error
    lib = koffi.load(libPath)
  }
  const api = declare(lib)

  function check(code, what = 'tauri-ffi call') {
    if (code !== CODES.OK) {
      throw new Error(`${what} failed (${code}): ${api.lastErrorMessage() ?? 'unknown error'}`)
    }
    return code
  }

  /** Decodes and releases an owned string written to a `_Out_ void **` slot. */
  function takeString(out) {
    if (!out[0]) return null
    const value = koffi.decode(out[0], 'char', -1)
    api.stringFree(out[0])
    return value
  }

  // Async so the worker's event loop stays free while we block on the queue —
  // koffi runs the call on the libuv thread pool and resolves a promise.
  function eventsNext(app, timeoutMs) {
    return new Promise((resolve, reject) => {
      const out = [null]
      api.eventsNext.async(app, timeoutMs, out, (err, code) => {
        if (err) return reject(err)
        if (code !== CODES.OK) return resolve({ code, json: null })
        const json = koffi.decode(out[0], 'char', -1)
        api.stringFree(out[0])
        resolve({ code, json })
      })
    })
  }

  /**
   * Runs the app, blocking this thread until it exits, and returns its exit
   * code.
   *
   * Goes through the Node-API addon rather than koffi: the loop drives every
   * webview, and koffi's private call stack makes WebKit abort (see
   * {@link loadRunAddon} and `native/tauri_node.c`). Falling back to koffi
   * keeps a broken install usable for engines that bring their own JS (cef),
   * which is the only reason this is a warning rather than an error.
   */
  function runApp(app) {
    const addon = loadRunAddon(runtime)
    if (addon) {
      const { status, exitCode } = addon.run(libPath, app)
      check(status, 'app_run')
      return exitCode
    }
    console.warn(
      `[tauri] ${RUN_ADDON_NAME} not found — falling back to a koffi run loop. ` +
        'A webkit-based (wry) app will abort in JavaScriptCore; build the addon with ' +
        '`node native/build.mjs`, or reinstall so the platform package is present.'
    )
    const outCode = [0]
    check(api.appRun(app, outCode), 'app_run')
    return outCode[0]
  }

  return { api, check, takeString, eventsNext, runApp, libPath }
}
