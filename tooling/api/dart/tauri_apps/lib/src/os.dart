// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names

/// Provides operating system-related utility methods and properties.
///
/// This package is also accessible with `window.__TAURI__.os` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.os`](https://tauri.app/v1/api/config/#allowlistconfig.os)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "os": {
///         "all": true, // enable all Os APIs
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_os;

import 'package:universal_io/io.dart' as io;

import './helpers/tauri.dart';

enum Platform {
  linux,
  darwin,
  ios,
  freebsd,
  dragonfly,
  netbsd,
  openbsd,
  solaris,
  android,
  win32,
}

enum OsType {
  Linux,
  Darwin,
  Windows_NT,
}

enum Arch {
  x86,
  x86_64,
  arm,
  aarch64,
  mips,
  mips64,
  powerpc,
  powerpc64,
  riscv64,
  s390x,
  sparc64,
}

/// The operating system-specific end-of-line marker.
/// - `\n` on POSIX
/// - `\r\n` on Windows
///
/// @since 1.0.0
// ignore: non_constant_identifier_names
String get EOL => io.Platform.isWindows ? '\r\n' : '\n';

/// Returns a String identifying the operating system platform.
/// The value is set at compile time. Possible values are:
///   * `'linux'`,
///   * `'darwin'`,
///   * `'ios'`,
///   * `'freebsd'`,
///   * `'dragonfly'`,
///   * `'netbsd'`,
///   * `'openbsd'`,
///   * `'solaris'`,
///   * `'android'`,
///   * `'win32'`.
///
/// ```dart
/// import 'package:tauri_apps/api/os.dart' show platform;
///
/// (() async {
///   final Platform platformName = await platform();
/// })();
/// ```
///
/// @since 1.0.0
///
Future<Platform> platform() async => invokeTauriCommand<Platform>(
      const TauriCommand(
        tauriModule: TauriModule.Os,
        message: TauriCommandMessage(
          cmd: 'platform',
        ),
      ),
    );

/// Returns a String identifying the kernel version.
///
/// ```dart
/// import 'package:tauri_apps/api/os.dart' show version;
///
/// (() async {
///   final String osVersion = await version();
/// })();
/// ```
///
/// @since 1.0.0
Future<String> version() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Os,
        message: TauriCommandMessage(
          cmd: 'version',
        ),
      ),
    );

/// Returns `'Linux'` on Linux, `'Darwin'` on macOS, and `'Windows_NT'` on
/// Windows.
///
/// ```dart
/// import 'package:tauri_apps/api/os.dart' show type;
///
/// (() async {
///   final OsType osType = await type();
/// })();
/// ```
///
/// @since 1.0.0
Future<OsType> type() async => invokeTauriCommand<OsType>(
      const TauriCommand(
        tauriModule: TauriModule.Os,
        message: TauriCommandMessage(
          cmd: 'osType',
        ),
      ),
    );

/// Returns the operating system CPU architecture for which the tauri app was
/// compiled.
///
/// Possible values are:
///   * `'x86'`,
///   * `'x86_64'`,
///   * `'arm'`,
///   * `'aarch64'`,
///   * `'mips'`,
///   * `'mips64'`,
///   * `'powerpc'`,
///   * `'powerpc64'`,
///   * `'riscv64'`,
///   * `'s390x'`,
///   * `'sparc64'`.
///
/// ```dart
/// import 'package:tauri_apps/api/os.dart' show arch;
///
/// (() async {
///   final Arch archName = await arch();
/// })();
/// ```
///
/// @since 1.0.0
Future<Arch> arch() async => invokeTauriCommand<Arch>(
      const TauriCommand(
        tauriModule: TauriModule.Os,
        message: TauriCommandMessage(
          cmd: 'arch',
        ),
      ),
    );

/// Returns the operating system's default directory for temporary files as a
/// String.
///
/// ```dart
/// import 'package:tauri_apps/api/os.dart' show tempdir;
///
/// (() async {
///   final String tempdirPath = await tempdir();
/// })();
/// ```
///
/// @since 1.0.0
Future<String> tempdir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Os,
        message: TauriCommandMessage(
          cmd: 'tempdir',
        ),
      ),
    );
