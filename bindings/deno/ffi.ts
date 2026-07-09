// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Runtime layer over the generated Deno.dlopen symbols (./symbols.ts, from
// crates/tauri-ffi/api-manifest.json). Only hand-written sugar lives here:
// library discovery, string encoding, error checking and the event-queue poll.

import { ABI_VERSION, CODES, SYMBOLS } from './symbols.ts'

export { ABI_VERSION, CODES }

const encoder = new TextEncoder()

/** Null-terminated UTF-8 buffer for 'buffer' string params. */
export function cstr(text: string): Uint8Array {
  return encoder.encode(text + '\0')
}

export function libraryPath(): string {
  const env = Deno.env.get('TAURI_FFI_LIB')
  if (env) return env
  const name = {
    darwin: 'libtauri_ffi.dylib',
    linux: 'libtauri_ffi.so',
    windows: 'tauri_ffi.dll'
  }[Deno.build.os as string]
  if (!name) throw new Error(`unsupported platform: ${Deno.build.os}`)
  for (const profile of ['debug', 'release']) {
    const url = new URL(`../../target/${profile}/${name}`, import.meta.url)
    const path = decodeURIComponent(url.pathname)
    try {
      Deno.statSync(path)
      return path
    } catch {
      // keep looking
    }
  }
  throw new Error(
    'tauri_ffi library not found — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB'
  )
}

export function open(libPath = libraryPath()) {
  const lib = Deno.dlopen(libPath, SYMBOLS)
  const sym = lib.symbols

  const abi = sym.tauri_ffi_abi_version()
  if (abi !== ABI_VERSION) {
    throw new Error(`ABI mismatch: library has v${abi}, bindings expect v${ABI_VERSION}`)
  }

  function lastError(): string {
    const pointer = sym.tauri_last_error_message()
    return pointer === null ? 'unknown error' : new Deno.UnsafePointerView(pointer).getCString()
  }

  function check(code: number, what = 'tauri-ffi call'): number {
    if (code !== CODES.OK) throw new Error(`${what} failed (${code}): ${lastError()}`)
    return code
  }

  /** Decodes and releases an owned string written to a char** out slot. */
  function takeString(slot: BigUint64Array): string | null {
    const pointer = Deno.UnsafePointer.create(slot[0])
    if (pointer === null) return null
    const value = new Deno.UnsafePointerView(pointer).getCString()
    sym.tauri_string_free(pointer)
    return value
  }

  // Declared nonblocking in symbols.ts — runs on Deno's blocking pool and
  // resolves a promise, keeping the caller's event loop free.
  async function eventsNext(app: bigint, timeoutMs: number) {
    const slot = new BigUint64Array(1)
    const code = await sym.tauri_events_next(app, timeoutMs, new Uint8Array(slot.buffer))
    if (code !== CODES.OK) return { code, json: null }
    return { code, json: takeString(slot) }
  }

  return { lib, sym, check, lastError, takeString, eventsNext, libPath }
}
