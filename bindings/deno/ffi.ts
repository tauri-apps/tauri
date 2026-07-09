// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Runtime layer over the generated Deno.dlopen symbols (./symbols.ts, from
// crates/tauri-ffi/api-manifest.json). Only hand-written sugar lives here:
// library discovery, string encoding, error checking and the event-queue poll.

import { ABI_VERSION, CODES, SYMBOLS } from './symbols.ts'
import denoConfig from './deno.json' with { type: 'json' }

export { ABI_VERSION, CODES }

const encoder = new TextEncoder()

/** Null-terminated UTF-8 buffer for 'buffer' string params. */
export function cstr(text: string): Uint8Array {
  return encoder.encode(text + '\0')
}

function libraryName(): string {
  const name = {
    darwin: 'libtauri_ffi.dylib',
    linux: 'libtauri_ffi.so',
    windows: 'tauri_ffi.dll'
  }[Deno.build.os as string]
  if (!name) throw new Error(`unsupported platform: ${Deno.build.os}`)
  return name
}

function targetTriple(): string {
  const os = {
    darwin: 'apple-darwin',
    linux: 'unknown-linux-gnu',
    windows: 'pc-windows-msvc'
  }[Deno.build.os as string]
  if (!os) throw new Error(`unsupported platform: ${Deno.build.os}`)
  return `${Deno.build.arch}-${os}`
}

function cachedLibraryPath(): string {
  const override = Deno.env.get('TAURI_FFI_CACHE')
  const base =
    override ??
    {
      darwin: `${Deno.env.get('HOME')}/Library/Caches`,
      linux: Deno.env.get('XDG_CACHE_HOME') ?? `${Deno.env.get('HOME')}/.cache`,
      windows: Deno.env.get('LOCALAPPDATA')
    }[Deno.build.os as string]
  if (!base) throw new Error('cannot determine a cache directory — set TAURI_FFI_CACHE')
  return `${base}/tauri-ffi/${denoConfig.version}/${libraryName()}`
}

export function libraryPath(): string {
  const env = Deno.env.get('TAURI_FFI_LIB')
  if (env) return env
  const name = libraryName()
  // Development fallback: cargo build output in the repo.
  for (const profile of ['debug', 'release']) {
    const url = new URL(`../../target/${profile}/${name}`, import.meta.url)
    if (url.protocol === 'file:') {
      const path = decodeURIComponent(url.pathname)
      try {
        Deno.statSync(path)
        return path
      } catch {
        // keep looking
      }
    }
  }
  // Previously downloaded by ensureLibrary().
  const cached = cachedLibraryPath()
  try {
    Deno.statSync(cached)
    return cached
  } catch {
    throw new Error(
      'tauri_ffi library not found — run `cargo build -p tauri-ffi`, set TAURI_FFI_LIB, or use launch() (which downloads a prebuilt library)'
    )
  }
}

/**
 * Resolves the library, downloading the prebuilt cdylib for this platform
 * from the matching GitHub release into the cache on first use.
 */
export async function ensureLibrary(): Promise<string> {
  try {
    return libraryPath()
  } catch {
    // not present locally — fetch the prebuilt
  }
  const version = denoConfig.version
  if (version === '0.0.0') {
    throw new Error(
      'tauri_ffi library not found and this is an unpublished checkout — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB'
    )
  }
  const asset = `tauri_ffi-${targetTriple()}${libraryName().slice(libraryName().lastIndexOf('.'))}`
  const url = `https://github.com/tauri-apps/tauri/releases/download/tauri-ffi-v${version}/${asset}`
  const destination = cachedLibraryPath()
  console.error(`[tauri-ffi] downloading ${url}`)
  const response = await fetch(url)
  if (!response.ok) throw new Error(`failed to download ${url}: HTTP ${response.status}`)
  const bytes = new Uint8Array(await response.arrayBuffer())
  await Deno.mkdir(destination.slice(0, destination.lastIndexOf('/')), { recursive: true })
  const temp = `${destination}.download`
  await Deno.writeFile(temp, bytes)
  await Deno.rename(temp, destination)
  return destination
}

/** Bound library runtime returned by {@linkcode open}. */
export interface FfiLibrary {
  lib: Deno.DynamicLibrary<typeof SYMBOLS>
  sym: Deno.DynamicLibrary<typeof SYMBOLS>['symbols']
  check: (code: number, what?: string) => number
  lastError: () => string
  takeString: (slot: BigUint64Array) => string | null
  eventsNext: (app: bigint, timeoutMs: number) => Promise<{ code: number; json: string | null }>
  libPath: string
}

export function open(libPath: string = libraryPath()): FfiLibrary {
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
