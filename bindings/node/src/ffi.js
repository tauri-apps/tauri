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
import { CODES, declare } from './ffi-decls.js'

export { CODES }

const require = createRequire(import.meta.url)

export function libraryPath() {
  if (process.env.TAURI_FFI_LIB) return process.env.TAURI_FFI_LIB
  const name = { darwin: 'libtauri_ffi.dylib', linux: 'libtauri_ffi.so', win32: 'tauri_ffi.dll' }[
    process.platform
  ]
  if (!name) throw new Error(`unsupported platform: ${process.platform}`)
  // Installed platform package (optionalDependencies of @tauri-apps/node).
  try {
    return require.resolve(`@tauri-apps/node-${process.platform}-${process.arch}/${name}`)
  } catch {
    // not installed — fall through to the repo dev build
  }
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..')
  for (const profile of ['debug', 'release']) {
    const candidate = path.join(repoRoot, 'target', profile, name)
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
