// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Runtime layer over the generated Deno.dlopen symbols (./symbols.ts, from
// crates/tauri-ffi/api-manifest.json). Only hand-written sugar lives here:
// library discovery, string encoding, error checking and the event-queue poll.

import { bundledResourceDir, isBundled } from './config.ts'
import { ABI_VERSION, CODES, SYMBOLS } from './symbols.ts'
import denoConfig from './deno.json' with { type: 'json' }

export { ABI_VERSION, CODES }

const encoder = new TextEncoder()

/** Null-terminated UTF-8 buffer for 'buffer' string params. */
export function cstr(text: string): Uint8Array {
  return encoder.encode(text + '\0')
}

// The runtime this package loads a library for (wry, cef, …), overridable via
// TAURI_FFI_RUNTIME.
const DEFAULT_RUNTIME = 'wry'

function ffiRuntime(): string {
  return Deno.env.get('TAURI_FFI_RUNTIME') || DEFAULT_RUNTIME
}

// Every runtime's library shares the same tauri_ffi C ABI, so it differs only by
// file name: `tauri_<base>` (libtauri_wry.so, tauri_cef.dll, …). `ffi` is the
// crate's own output name, used for the local dev build (cargo emits it
// regardless of the selected runtime feature); the runtime name is what the
// bundle, prebuilt download/cache and installed packages use.
function platformLibraryName(base: string): string {
  const spec = { darwin: ['lib', '.dylib'], linux: ['lib', '.so'], windows: ['', '.dll'] }[
    Deno.build.os as string
  ]
  if (!spec) throw new Error(`unsupported platform: ${Deno.build.os}`)
  return `${spec[0]}tauri_${base}${spec[1]}`
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

function cachedLibraryPath(runtime: string = ffiRuntime()): string {
  const override = Deno.env.get('TAURI_FFI_CACHE')
  const base =
    override ??
    {
      darwin: `${Deno.env.get('HOME')}/Library/Caches`,
      linux: Deno.env.get('XDG_CACHE_HOME') ?? `${Deno.env.get('HOME')}/.cache`,
      windows: Deno.env.get('LOCALAPPDATA')
    }[Deno.build.os as string]
  if (!base) throw new Error('cannot determine a cache directory — set TAURI_FFI_CACHE')
  return `${base}/tauri-ffi/${denoConfig.version}/${platformLibraryName(runtime)}`
}

export function libraryPath(runtime: string = ffiRuntime()): string {
  // a compiled bundle ignores the TAURI_FFI_LIB env override — it must load
  // only its own bundled cdylib, never an arbitrary library named by the env
  const env = isBundled() ? undefined : Deno.env.get('TAURI_FFI_LIB')
  if (env) return env
  const distName = platformLibraryName(runtime)
  // Bundled next to the executable (deno compile binary in a Tauri bundle): the
  // CLI staged the per-runtime library selected from `app.runtime`.
  const resourceDir = bundledResourceDir()
  if (resourceDir) {
    const bundled = `${resourceDir}/${distName}`
    try {
      Deno.statSync(bundled)
      return bundled
    } catch {
      // not a bundle — keep looking
    }
  }
  // Development fallback: `cargo build -p tauri-ffi` output (crate's own name).
  const crateName = platformLibraryName('ffi')
  for (const profile of ['debug', 'release']) {
    const url = new URL(`../../target/${profile}/${crateName}`, import.meta.url)
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
  const cached = cachedLibraryPath(runtime)
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
export async function ensureLibrary(runtime: string = ffiRuntime()): Promise<string> {
  try {
    return libraryPath(runtime)
  } catch {
    // not present locally — fetch the prebuilt
  }
  const version = denoConfig.version
  if (version === '0.0.0') {
    throw new Error(
      'tauri_ffi library not found and this is an unpublished checkout — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB'
    )
  }
  const distName = platformLibraryName(runtime)
  const asset = `tauri_${runtime}-${targetTriple()}${distName.slice(distName.lastIndexOf('.'))}`
  const url = `https://github.com/tauri-apps/tauri/releases/download/tauri-ffi-v${version}/${asset}`
  const destination = cachedLibraryPath(runtime)
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
