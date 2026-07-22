// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The distribution matrix — single source for dist.mjs, the language loaders'
// naming and anything else staging release artifacts. Two dimensions:
//   - RUNTIMES: each produces a distinct compiled library that implements the
//     *same* tauri_ffi C ABI. The library is distributed per runtime
//     (libtauri_wry, libtauri_cef, …); the header/symbols/env var stay `ffi`.
//   - TARGETS: the platform/arch build matrix.
// Keep both in sync with the build matrix in .github/workflows/publish-ffi.yml.

/** Runtimes we build and distribute a library for. */
export const RUNTIMES = ['wry', 'cef']

/** The default runtime a language package loads when none is selected. */
export const DEFAULT_RUNTIME = 'wry'

export const TARGETS = {
  'x86_64-apple-darwin': { platform: 'darwin', arch: 'x64' },
  'aarch64-apple-darwin': { platform: 'darwin', arch: 'arm64' },
  'x86_64-pc-windows-msvc': { platform: 'win32', arch: 'x64' },
  'aarch64-pc-windows-msvc': { platform: 'win32', arch: 'arm64' },
  'x86_64-unknown-linux-gnu': { platform: 'linux', arch: 'x64' },
  'aarch64-unknown-linux-gnu': { platform: 'linux', arch: 'arm64' }
}

/**
 * The compiled library file(s) a runtime produces for a target. The base name is
 * `tauri_<runtime>` (libtauri_wry.so, tauri_cef.dll, …). `extra` are companion
 * files shipped alongside the library (the Windows import lib).
 */
export function libraryFiles(target, runtime) {
  if (target.includes('windows')) {
    return { lib: `tauri_${runtime}.dll`, extra: [`tauri_${runtime}.dll.lib`] }
  }
  if (target.includes('apple')) {
    return { lib: `libtauri_${runtime}.dylib`, extra: [] }
  }
  return { lib: `libtauri_${runtime}.so`, extra: [] }
}

/**
 * The Node-API addon the Node bindings run the app's event loop through, shipped
 * in every npm platform package (see `bindings/node/native/tauri_node.c` for why
 * it exists — koffi cannot host a webview's event loop).
 *
 * It is Node-specific and runtime-agnostic, so it is deliberately not part of
 * `libraryFiles`: the C/Deno distributions have no use for it, and each runtime's
 * platform package simply carries its own copy.
 */
export const RUN_ADDON_FILE = 'tauri_node.node'

/**
 * The CEF subprocess helper (`tauri-cef-helper`) executable that ships beside the
 * `cef` library. CEF is multi-process and re-executes a helper for its
 * renderer/GPU/utility subprocesses; a bindings host (node/deno/python) cannot
 * act as one, so every language points CEF at this executable staged next to the
 * library (built from the `cef-helper` crate). It is cef-only and
 * language-agnostic, so — like `RUN_ADDON_FILE` — it is not part of
 * `libraryFiles`; `cefHelperFile` returns it only for the `cef` runtime.
 */
export function cefHelperFile(target, runtime) {
  if (runtime !== 'cef') return null
  return target.includes('windows')
    ? 'tauri-cef-helper.exe'
    : 'tauri-cef-helper'
}

/** npm platform package for a (target, runtime): @tauri-apps/node-<runtime>-<platform>-<arch>. */
export function platformPackageName(target, runtime) {
  const { platform, arch } = TARGETS[target]
  return `@tauri-apps/node-${runtime}-${platform}-${arch}`
}

/** CI artifact folder for a (target, runtime) build: lib-<runtime>-<target>. */
export function artifactName(target, runtime) {
  return `lib-${runtime}-${target}`
}
