// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The distribution target matrix — single source for dist.mjs and anything else staging release artifacts.
// Keep in sync with the build matrix in .github/workflows/publish-ffi.yml.

export const TARGETS = {
  'x86_64-apple-darwin': { platform: 'darwin', arch: 'x64', lib: 'libtauri_ffi.dylib' },
  'aarch64-apple-darwin': { platform: 'darwin', arch: 'arm64', lib: 'libtauri_ffi.dylib' },
  'x86_64-pc-windows-msvc': {
    platform: 'win32',
    arch: 'x64',
    lib: 'tauri_ffi.dll',
    extra: ['tauri_ffi.dll.lib']
  },
  'aarch64-pc-windows-msvc': {
    platform: 'win32',
    arch: 'arm64',
    lib: 'tauri_ffi.dll',
    extra: ['tauri_ffi.dll.lib']
  },
  'x86_64-unknown-linux-gnu': { platform: 'linux', arch: 'x64', lib: 'libtauri_ffi.so' },
  'aarch64-unknown-linux-gnu': { platform: 'linux', arch: 'arm64', lib: 'libtauri_ffi.so' }
}

/** npm platform package for a target: @tauri-apps/node-<platform>-<arch>. */
export function platformPackageName(target) {
  const { platform, arch } = TARGETS[target]
  return `@tauri-apps/node-${platform}-${arch}`
}
