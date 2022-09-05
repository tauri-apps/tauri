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

/**
 * The operating system-specific end-of-line marker.
 * - `\n` on POSIX
 * - `\r\n` on Windows
 * */
const EOL = isWindows() ? '\r\n' : '\n'

type Platform =
  | 'linux'
  | 'macos'
  | 'ios'
  | 'freebsd'
  | 'dragonfly'
  | 'netbsd'
  | 'openbsd'
  | 'solaris'
  | 'android'
  | 'windows'

/**
 * Returns a string identifying the operating system platform.
 * The value is set at compile time. Possible values are `'linux'`, `'darwin'`, `'ios'`, `'freebsd'`, `'dragonfly'`, `'netbsd'`, `'openbsd'`, `'solaris'`, `'android'`, `'win32'`
 * @example
 * ```typescript
 * import { platform } from '@tauri-apps/api/os';
 * const platformName = await platform();
 * ```
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
 */
async function version(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'version'
    }
  })
}

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
 * Returns the operating system CPU architecture for which the tauri app was compiled.
 * Possible values are `'x86'`, `'x86_64'`, `'arm'`, `'aarch64'`, `'mips'`, `'mips64'`, `'powerpc'`, `'powerpc64'`, `'riscv64'`, `'s390x'`, `'sparc64'`.
 * @example
 * ```typescript
 * import { arch } from '@tauri-apps/api/os';
 * const archName = await arch();
 * ```
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
 */
async function tempdir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'tempdir'
    }
  })
}

/**
 * Returns the host name of the operating system as a string.
 * @example
 * ```typescript
 * import { hostname } from '@tauri-apps/api/os';
 * const hostname = await hostname();
 * ```
 */
async function hostname(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Os',
    message: {
      cmd: 'hostname'
    }
  })
}

export { EOL, platform, version, arch, tempdir, hostname }
export type { Platform, Arch }
