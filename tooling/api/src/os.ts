// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides operating system-related utility methods and properties.
 *
 * This package is also accessible with `window.__TAURI__.os` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.os`](https://tauri.app/v1/api/config/#allowlistconfig.os) in `tauri.conf.json`:
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

type Platform =
  | 'linux'
  | 'darwin'
  | 'ios'
  | 'freebsd'
  | 'dragonfly'
  | 'netbsd'
  | 'openbsd'
  | 'solaris'
  | 'android'
  | 'win32'

type OsType = 'Linux' | 'Darwin' | 'Windows_NT'

type Arch =
  | 'x86'
  | 'x86_64'
  | 'arm'
  | 'aarch64'
  | 'mips'
  | 'mips64'
  | 'powerpc'
  | 'powerpc64'
  | 'riscv64'
  | 's390x'
  | 'sparc64'

/**
 * The operating system-specific end-of-line marker.
 * - `\n` on POSIX
 * - `\r\n` on Windows
 *
 * @since 1.0.0
 * */
const EOL = isWindows() ? '\r\n' : '\n'

/**
 * Returns a string identifying the operating system platform.
 * The value is set at compile time. Possible values are `'linux'`, `'darwin'`, `'ios'`, `'freebsd'`, `'dragonfly'`, `'netbsd'`, `'openbsd'`, `'solaris'`, `'android'`, `'win32'`
 * @example
 * ```typescript
 * import { platform } from '@tauri-apps/api/os';
 * const platformName = await platform();
 * ```
 *
 * @since 1.0.0
 *
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
 * @example
 * ```typescript
 * import { version } from '@tauri-apps/api/os';
 * const osVersion = await version();
 * ```
 *
 * @since 1.0.0
 */
async function version(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'version'
    }
  })
}

/**
 * Returns `'Linux'` on Linux, `'Darwin'` on macOS, and `'Windows_NT'` on Windows.
 * @example
 * ```typescript
 * import { type } from '@tauri-apps/api/os';
 * const osType = await type();
 * ```
 *
 * @since 1.0.0
 */
async function type(): Promise<OsType> {
  return invokeTauriCommand<OsType>({
    __tauriModule: 'Os',
    message: {
      cmd: 'osType'
    }
  })
}

/**
 * Returns the operating system CPU architecture for which the tauri app was compiled.
 * Possible values are `'x86'`, `'x86_64'`, `'arm'`, `'aarch64'`, `'mips'`, `'mips64'`, `'powerpc'`, `'powerpc64'`, `'riscv64'`, `'s390x'`, `'sparc64'`.
 * @example
 * ```typescript
 * import { arch } from '@tauri-apps/api/os';
 * const archName = await arch();
 * ```
 *
 * @since 1.0.0
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
 * @example
 * ```typescript
 * import { tempdir } from '@tauri-apps/api/os';
 * const tempdirPath = await tempdir();
 * ```
 *
 * @since 1.0.0
 */
async function tempdir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'tempdir'
    }
  })
}

export { EOL, platform, version, type, arch, tempdir }
export type { Platform, OsType, Arch }
