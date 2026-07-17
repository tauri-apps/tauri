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
// bundle and the published npm platform packages use.
function platformLibraryName(base: string): string {
  const spec = { darwin: ['lib', '.dylib'], linux: ['lib', '.so'], windows: ['', '.dll'] }[
    Deno.build.os as string
  ]
  if (!spec) throw new Error(`unsupported platform: ${Deno.build.os}`)
  return `${spec[0]}tauri_${base}${spec[1]}`
}

// The npm platform package that ships this runtime's library for the current
// target: `@tauri-apps/node-<runtime>-<platform>-<arch>`. The suffix follows
// Node's `process.platform`/`process.arch` naming (see bindings/scripts/
// targets.mjs), which is what the packages are published under — so Deno maps
// its own `Deno.build` names onto that.
function platformPackage(runtime: string): string {
  const platform = { darwin: 'darwin', linux: 'linux', windows: 'win32' }[Deno.build.os as string]
  const arch = { x86_64: 'x64', aarch64: 'arm64' }[Deno.build.arch as string]
  if (!platform || !arch) {
    throw new Error(`unsupported platform: ${Deno.build.os}-${Deno.build.arch}`)
  }
  return `@tauri-apps/node-${runtime}-${platform}-${arch}`
}

/** Converts a `file://` URL string to a filesystem path (handles a Windows
 * drive-letter path like `/C:/…`). */
function fromFileUrl(url: string): string {
  const path = decodeURIComponent(new URL(url).pathname)
  return Deno.build.os === 'windows' && /^\/[A-Za-z]:/.test(path) ? path.slice(1) : path
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
  throw new Error(
    'tauri_ffi library not found — run `cargo build -p tauri-ffi`, set TAURI_FFI_LIB, or call launch()/ensureLibrary() (which installs the prebuilt library from npm)'
  )
}

/**
 * Resolves the library, installing the prebuilt cdylib from its npm platform
 * package (`@tauri-apps/node-<runtime>-<platform>-<arch>`) on first use.
 *
 * This is the same package — and the same library file — the Node bindings load:
 * reusing it means the prebuilt is published once, as npm platform packages that
 * bundle the library, and both runtimes share it rather than the Deno bindings
 * fetching a separate copy from a GitHub release. Deno installs an npm package
 * into its module cache the first time it is imported, so importing the package
 * makes `import.meta.resolve` hand back the on-disk path of the bundled library
 * to `dlopen` — no bespoke download, checksum or cache handling of our own.
 */
export async function ensureLibrary(runtime: string = ffiRuntime()): Promise<string> {
  try {
    return libraryPath(runtime)
  } catch {
    // not present locally — install the prebuilt from npm below
  }
  const version = denoConfig.version
  if (version === '0.0.0') {
    throw new Error(
      'tauri_ffi library not found and this is an unpublished checkout — run `cargo build -p tauri-ffi` or set TAURI_FFI_LIB'
    )
  }
  const spec = `npm:${platformPackage(runtime)}@${version}`
  const distName = platformLibraryName(runtime)
  // Importing anything from the package triggers Deno's npm install; the
  // package.json import is otherwise unused — the platform package ships no code,
  // only the library — but it is always present and safe to load as JSON.
  try {
    await import(`${spec}/package.json`, { with: { type: 'json' } })
  } catch (error) {
    throw new Error(
      `failed to install the ${runtime} tauri-ffi library from npm (${spec}) — ${
        error instanceof Error ? error.message : error
      }`
    )
  }
  // Now installed: resolve the bundled library file to a real path. import.meta
  // .resolve only yields a `file:` URL once the package is in the cache (before
  // that it echoes the `npm:` specifier), which is why it follows the import.
  const url = import.meta.resolve(`${spec}/${distName}`)
  if (!url.startsWith('file:')) {
    throw new Error(`could not resolve ${distName} from the npm package ${spec} (resolved to ${url})`)
  }
  return fromFileUrl(url)
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

function cefLibraryName(): string {
  return (
    { darwin: 'libcef.dylib', linux: 'libcef.so', windows: 'libcef.dll' }[Deno.build.os as string] ??
    'libcef.so'
  )
}

function cefHelperName(): string {
  return Deno.build.os === 'windows' ? 'tauri-cef-helper.exe' : 'tauri-cef-helper'
}

/** The directory holding `file`. */
function parentDir(file: string): string {
  const slash = Math.max(file.lastIndexOf('/'), file.lastIndexOf('\\'))
  return slash === -1 ? '.' : file.slice(0, slash)
}

function exists(path: string): boolean {
  try {
    Deno.statSync(path)
    return true
  } catch {
    return false
  }
}

/**
 * Preloads the `libcef` staged next to a cef library so that library's own
 * (rpath-less) dependency on it resolves. The dynamic loader reads its search
 * path once at process start, so it can't be pointed at `dir` from in here;
 * loading libcef by absolute path instead puts it in the process's link map and
 * the subsequent dlopen resolves the dependency to it by name.
 */
function preloadCef(dir: string): boolean {
  const libcef = `${dir}/${cefLibraryName()}`
  if (!exists(libcef)) return false
  Deno.dlopen(libcef, {})
  return true
}

export function open(libPath: string = libraryPath()): FfiLibrary {
  const dir = parentDir(libPath)
  // CEF is multi-process: it launches renderer/GPU/utility subprocesses by
  // executing a helper program. A Deno host can't act as one, so point CEF at
  // the helper staged next to the library. A wry library never reads this.
  // A compiled bundle must point at its own staged helper (hermetic, like
  // TAURI_FFI_LIB); in dev an explicit env value wins.
  const helper = `${dir}/${cefHelperName()}`
  if (exists(helper) && (isBundled() || !Deno.env.get('TAURI_CEF_SUBPROCESS_PATH'))) {
    Deno.env.set('TAURI_CEF_SUBPROCESS_PATH', helper)
  }
  let lib: Deno.DynamicLibrary<typeof SYMBOLS>
  try {
    lib = Deno.dlopen(libPath, SYMBOLS)
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
    lib = Deno.dlopen(libPath, SYMBOLS)
  }
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
