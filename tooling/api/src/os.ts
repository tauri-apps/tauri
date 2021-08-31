// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides operating system-related utility methods and properties.
 *
 * This package is also accessible with `window.__TAURI__.fs` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "os": {
 *         "all": true, // enable all Os APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { isWindows } from './helpers/os-check'
import { invokeTauriCommand } from './helpers/tauri'

/**
 * The operating system-specific end-of-line marker.
 * - `\n` on POSIX
 * - `\r\n` on Windows
 * */
const EOL = isWindows() ? '\r\n' : '\n'


type Platform = 'aix' | 'darwin' | 'freebsd' | 'linux' | 'openbsd' | 'sunos' | 'win32'

/**
 * Returns a string identifying the operating system platform.
 * The value is set at compile time. Possible values are `'aix'`, `'darwin'`, `'freebsd'`, `'linux'`, `'openbsd'`, `'sunos'`, and `'win32'`.
 */
async function platform(): Promise<Platform> {
  return invokeTauriCommand<Platform>({
    __tauriModule: 'Os',
    message: {
      cmd: 'platform'
    }
  })
}

/**
 * Returns a string identifying the kernel version.
 */
async function version(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'version'
    }
  })
}


type OsType = 'Linux' | 'Darwin' | 'Windows_NT'

/**
 * Returns `'Linux'` on Linux, `'Darwin'` on macOS, and `'Windows_NT'` on Windows.
 */
async function type(): Promise<Type> {
  return invokeTauriCommand<Type>({
    __tauriModule: 'Os',
    message: {
      cmd: 'type'
    }
  })
}


type Arch = 'x86' | 'x86_64' | 'arm' | 'aarch64' | 'mips' | 'mips64' | 'powerpc' | 'powerpc64' | 'riscv64' | 's390x' | 'sparc64'

/**
 * Returns the operating system CPU architecture for which the tauri app was compiled. Possible values are `'x86'`, `'x86_64'`, `'arm'`, `'aarch64'`, `'mips'`, `'mips64'`, `'powerpc'`, `'powerpc64'`, `'riscv64'`, `'s390x'`, `'sparc64'`
 */
async function arch(): Promise<Arch> {
  return invokeTauriCommand<Arch>({
    __tauriModule: 'Os',
    message: {
      cmd: 'arch'
    }
  })
}

/**
 * Returns the operating system's default directory for temporary files as a string.
 */
async function tempdir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'tempdir'
    }
  })
}

export { EOL, Platform, platform, version, OsType, type, Arch, arch, tempdir }
