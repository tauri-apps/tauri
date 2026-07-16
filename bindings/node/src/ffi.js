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

export function libraryPath(runtime = ffiRuntime()) {
  // a compiled bundle ignores the TAURI_FFI_LIB env override — it must load
  // only its own bundled cdylib, never an arbitrary library named by the env
  if (!isBundled() && process.env.TAURI_FFI_LIB) return process.env.TAURI_FFI_LIB
  const distName = platformLibraryName(runtime)
  // a compiled bundle loads the per-runtime cdylib the CLI staged in its
  // resource dir (selected there from `app.runtime`)
  const resourceDir = bundledResourceDir()
  if (resourceDir) return path.join(resourceDir, distName)
  // Installed platform package (optionalDependencies of the runtime's base package).
  try {
    return require.resolve(`@tauri-apps/node-${runtime}-${process.platform}-${process.arch}/${distName}`)
  } catch {
    // not installed — fall through to the repo dev build
  }
  // Repo dev build: `cargo build -p tauri-ffi` emits the crate's own name,
  // regardless of the selected runtime feature.
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..')
  const devName = platformLibraryName('ffi')
  for (const profile of ['debug', 'release']) {
    const candidate = path.join(repoRoot, 'target', profile, devName)
    if (existsSync(candidate)) return candidate
  }
  throw new Error(
    'tauri_ffi library not found — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB'
  )
}

export function open(libPath = libraryPath()) {
  const lib = koffi.load(libPath)
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

  return { api, check, takeString, eventsNext, libPath }
}
