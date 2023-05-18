// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// The path module provides utilities for working with file and directory
/// paths.
///
/// This package is also accessible with `window.__TAURI__.path` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.path`](https://tauri.app/v1/api/config/#allowlistconfig.path)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "path": {
///         "all": true, // enable all Path APIs
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
library tauri_path;

import 'package:universal_io/io.dart';

import 'fs.dart';
import 'helpers/tauri.dart';

/// Returns the path to the suggested directory for your app config files.
///
/// @deprecated since 1.2.0: Will be removed in 2.0.0. Use {@link appConfigDir}
/// or {@link appDataDir} instead.
/// @since 1.0.0
Future<String> appDir() async => appConfigDir();

/// Returns the path to the suggested directory for your app's config files.
/// Resolves to `$configDir/$bundleIdentifier`, where `bundleIdentifier` is the
/// value
/// [`tauri.bundle.identifier`](https://tauri.app/v1/api/config/#bundleconfig.identifier)
/// is configured in `tauri.conf.json`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show appConfigDir;
///
/// appConfigDir().then(
///   (final String appConfigDirPath) => print(appConfigDirPath),
/// );
/// ```
///
/// @since 1.2.0
Future<String> appConfigDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.AppConfig,
        ),
      ),
    );

/// Returns the path to the suggested directory for your app's data files.
/// Resolves to `$dataDir/$bundleIdentifier`, where `bundleIdentifier` is the
/// value
/// [`tauri.bundle.identifier`](https://tauri.app/v1/api/config/#bundleconfig.identifier)
/// is configured in `tauri.conf.json`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show appDataDir;
///
/// appDataDir().then(
///   (final String appDataDirPath) => print(appDataDirPath),
/// );
/// ```
///
/// @since 1.2.0
Future<String> appDataDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.AppData,
        ),
      ),
    );

/// Returns the path to the suggested directory for your app's local data files.
/// Resolves to `$localDataDir/$bundleIdentifier`, where `bundleIdentifier` is
/// the value
/// [`tauri.bundle.identifier`](https://tauri.app/v1/api/config/#bundleconfig.identifier)
/// is configured in `tauri.conf.json`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show appLocalDataDir;
///
/// appLocalDataDir().then(
///   (final String appLocalDataDirPath) => print(appLocalDataDirPath),
/// );
/// ```
///
/// @since 1.2.0
Future<String> appLocalDataDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.AppLocalData,
        ),
      ),
    );

/// Returns the path to the suggested directory for your app's cache files.
/// Resolves to `$cacheDir/$bundleIdentifier`, where `bundleIdentifier` is the
/// value
/// [`tauri.bundle.identifier`](https://tauri.app/v1/api/config/#bundleconfig.identifier)
/// is configured in `tauri.conf.json`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show appCacheDir;
///
/// appCacheDir().then(
///   (final String appCacheDirPath) => print(appCacheDirPath),
/// );
/// ```
///
/// @since 1.2.0
Future<String> appCacheDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.AppCache,
        ),
      ),
    );

/// Returns the path to the user's audio directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///   [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///   `XDG_MUSIC_DIR`.
///   - **macOS:** Resolves to `$HOME/Music`.
///   - **Windows:** Resolves to `{FOLDERID_Music}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show audioDir;
///
/// audioDir().then(
///   (final String audioDirPath) => print(audioDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> audioDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Audio,
        ),
      ),
    );

/// Returns the path to the user's cache directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_CACHE_HOME` or `$HOME/.cache`.
///   - **macOS:** Resolves to `$HOME/Library/Caches`.
///   - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show cacheDir;
///
/// cacheDir().then(
///   (final String cacheDirPath) => print(cacheDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> cacheDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Cache,
        ),
      ),
    );

/// Returns the path to the user's config directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`.
///   - **macOS:** Resolves to `$HOME/Library/Application Support`.
///   - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show configDir;
///
/// configDir().then(
///   (final String configDirPath) => print(configDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> configDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Config,
        ),
      ),
    );

/// Returns the path to the user's data directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
///   - **macOS:** Resolves to `$HOME/Library/Application Support`.
///   - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show dataDir;
///
/// dataDir().then(
///   (final String dataDirPath) => print(dataDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> dataDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Data,
        ),
      ),
    );

/// Returns the path to the user's desktop directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_DESKTOP_DIR`.
///   - **macOS:** Resolves to `$HOME/Desktop`.
///   - **Windows:** Resolves to `{FOLDERID_Desktop}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show desktopDir;
///
/// desktopDir().then(
///   (final String desktopPath) => print(desktopPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> desktopDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Desktop,
        ),
      ),
    );

/// Returns the path to the user's document directory.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show documentDir;
///
/// documentDir().then(
///   (final String documentDirPath) => print(documentDirPath),
/// );
/// ```
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_DOCUMENTS_DIR`.
///   - **macOS:** Resolves to `$HOME/Documents`.
///   - **Windows:** Resolves to `{FOLDERID_Documents}`.
///
/// @since 1.0.0
Future<String> documentDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Document,
        ),
      ),
    );

/// Returns the path to the user's download directory.
///
/// #### Platform-specific
///
///   - **Linux**: Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_DOWNLOAD_DIR`.
///   - **macOS**: Resolves to `$HOME/Downloads`.
///   - **Windows**: Resolves to `{FOLDERID_Downloads}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show downloadDir;
///
/// downloadDir().then(
///   (final String downloadDirPath) => print(downloadDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> downloadDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Download,
        ),
      ),
    );

/// Returns the path to the user's executable directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_BIN_HOME/../bin` or `$XDG_DATA_HOME/../bin`
///     or `$HOME/.local/bin`.
///   - **macOS:** Not supported.
///   - **Windows:** Not supported.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show executableDir;
///
/// executableDir().then(
///   (final String executableDirPath) => print(executableDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> executableDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Executable,
        ),
      ),
    );

/// Returns the path to the user's font directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_DATA_HOME/fonts` or
///     `$HOME/.local/share/fonts`.
///   - **macOS:** Resolves to `$HOME/Library/Fonts`.
///   - **Windows:** Not supported.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show fontDir;
///
/// fontDir().then(
///   (final String fontDirPath) => print(fontDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> fontDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Font,
        ),
      ),
    );

/// Returns the path to the user's home directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$HOME`.
///   - **macOS:** Resolves to `$HOME`.
///   - **Windows:** Resolves to `{FOLDERID_Profile}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show homeDir;
///
/// fontDir().then(
///   (final String fontDirPath) => print(fontDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> homeDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Home,
        ),
      ),
    );

/// Returns the path to the user's local data directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
///   - **macOS:** Resolves to `$HOME/Library/Application Support`.
///   - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show localDataDir;
///
/// localDataDir().then(
///   (final String localDataDirPath) => print(localDataDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> localDataDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.LocalData,
        ),
      ),
    );

/// Returns the path to the user's picture directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_PICTURES_DIR`.
///   - **macOS:** Resolves to `$HOME/Pictures`.
///   - **Windows:** Resolves to `{FOLDERID_Pictures}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show pictureDir;
///
/// pictureDir().then(
///   (final String pictureDirPath) => print(pictureDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> pictureDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Picture,
        ),
      ),
    );

/// Returns the path to the user's public directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_PUBLICSHARE_DIR`.
///   - **macOS:** Resolves to `$HOME/Public`.
///   - **Windows:** Resolves to `{FOLDERID_Public}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show publicDir;
///
/// publicDir().then(
///   (final String publicDirPath) => print(publicDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> publicDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Public,
        ),
      ),
    );

/// Returns the path to the application's resource directory.
/// To resolve a resource path, see the [[resolveResource |
/// `resolveResource API`]].
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show resourceDir;
///
/// resourceDir().then(
///   (final String resourceDirPath) => print(resourceDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> resourceDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Resource,
        ),
      ),
    );

/// Resolve the path to a resource file.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show resolveResource;
///
/// resolveResource('script.sh').then(
///   (final String resourcePath) => print(resourcePath),
/// );
/// ```
///
///   * [resourcePath]: The path to the resource.
///
/// Must follow the same syntax as defined in `tauri.conf.json > tauri > bundle
/// > resources`, i.e. keeping subfolders and parent dir components (`../`).
///
/// @returns The full path to the resource.
///
/// @since 1.0.0
Future<String> resolveResource(final String resourcePath) async =>
    invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: resourcePath,
          directory: BaseDirectory.Resource,
        ),
      ),
    );

/// Returns the path to the user's runtime directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$XDG_RUNTIME_DIR`.
///   - **macOS:** Not supported.
///   - **Windows:** Not supported.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show runtimeDir;
///
/// runtimeDir().then(
///   (final String runtimeDirPath) => print(runtimeDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> runtimeDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Runtime,
        ),
      ),
    );

/// Returns the path to the user's template directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_TEMPLATES_DIR`.
///   - **macOS:** Not supported.
///   - **Windows:** Resolves to `{FOLDERID_Templates}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show templateDir;
///
/// templateDir().then(
///   (final String templateDirPath) => print(templateDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> templateDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Template,
        ),
      ),
    );

/// Returns the path to the user's video directory.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to
///     [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)'
///     `XDG_VIDEOS_DIR`.
///   - **macOS:** Resolves to `$HOME/Movies`.
///   - **Windows:** Resolves to `{FOLDERID_Videos}`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show videoDir;
///
/// videoDir().then(
///   (final String videoDirPath) => print(videoDirPath),
/// );
/// ```
///
/// @since 1.0.0
Future<String> videoDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.Video,
        ),
      ),
    );

/// Returns the path to the suggested log directory.
///
/// @deprecated since 1.2.0: Will be removed in 2.0.0. Use {@link appLogDir}
/// instead.
/// @since 1.0.0
Future<String> logDir() async => appLogDir();

/// Returns the path to the suggested directory for your app's log files.
///
/// #### Platform-specific
///
///   - **Linux:** Resolves to `$configDir/$bundleIdentifier/logs`.
///   - **macOS:** Resolves to `${homeDir}/Library/Logs/{bundleIdentifier}`
///   - **Windows:** Resolves to `$configDir/$bundleIdentifier/logs`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show appLogDir;
///
/// appLogDir().then(
///   (final String appLogDirPath) => print(appLogDirPath),
/// );
/// ```
///
/// @since 1.2.0
Future<String> appLogDir() async => invokeTauriCommand<String>(
      const TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(
          cmd: 'resolvePath',
          path: '',
          directory: BaseDirectory.AppLog,
        ),
      ),
    );

/// Provides the platform-specific path segment separator:
/// - `\` on Windows
/// - `/` on POSIX
///
/// @since 1.0.0
final String sep = Platform.isWindows ? r'\' : '/';

/// Provides the platform-specific path segment delimiter:
/// - `;` on Windows
/// - `:` on POSIX
///
/// @since 1.0.0
final String delimiter = Platform.isWindows ? ';' : ':';

/// Resolves a sequence of `paths` or `path` segments into an absolute path.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show resolve, appDataDir;
///
/// appDataDir()
///   .then(
///     (final String appDataDirPath) => resolve(
///       <String>[appDataDirPath, '..', 'users', 'tauri', 'avatar.png'],
///     )
///   ).then((final String path) => print(path));
/// );
/// ```
///
/// @since 1.0.0
Future<String> resolve(final List<String> paths) async =>
    invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'resolve', paths: paths),
      ),
    );

/// Normalizes the given `path`, resolving `'..'` and `'.'` segments and resolve
/// symbolic links.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show normalize, appDataDir;
///
/// appDataDir()
///   .then((final String appDataDirPath) => normalize(appDataDirPath))
///   .then((final String path) => print(path));
/// );
/// ```
///
/// @since 1.0.0
Future<String> normalize(final String path) async => invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'normalize', path: path),
      ),
    );

/// Joins all given `path` segments together using the platform-specific
/// separator as a delimiter, then normalizes the resulting path.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show join, appDataDir;
///
/// appDataDir()
///   .then(
///     (final String appDataDirPath) => join(
///       <String>[appDataDirPath, 'users', 'tauri', 'avatar.png'],
///     )
///   ).then((final String path) => print(path));
/// );
/// ```
///
/// @since 1.0.0
Future<String> join(final List<String> paths) async =>
    invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'join', paths: paths),
      ),
    );

/// Returns the directory name of a `path`. Trailing directory separators are
/// ignored.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show dirname, appDataDir;
///
/// appDataDir()
///   .then((final String appDataDirPath) => dirname(appDataDirPath))
///   .then((final String dir) => print(dir));
/// );
/// ```
///
/// @since 1.0.0
Future<String> dirname(final String path) async => invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'dirname', path: path),
      ),
    );

/// Returns the extension of the `path`.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show extname, resolveResource;
///
/// resolveResource('app.conf')
///   .then((final String resourcePath) => extname(resourcePath))
///   .then((final String ext) => print(ext));
/// ```
///
/// @since 1.0.0
Future<String> extname(final String path) async => invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'extname', path: path),
      ),
    );

/// Returns the last portion of a `path`. Trailing directory separators are
/// ignored.
///
/// ```dart
/// import, resolveResource 'package:tauri_apps/api/path.dart' show basename;
///
/// resolveResource('app.conf')
///   .then((final String resourcePath) => basename(resourcePath))
///   .then((final String base) => print(base));
/// ```
///
///   * [ext]: An optional file extension to be removed from the returned path.
///
/// @since 1.0.0
Future<String> basename(final String path, [final String? ext]) async =>
    invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'basename', path: path, ext: ext),
      ),
    );

/// Returns whether the path is absolute or not.
///
/// ```dart
/// import 'package:tauri_apps/api/path.dart' show isAbsolute;
/// isAbsolute('/home/tauri').then((final bool absolute) => print(absolute));
/// ```
///
/// @since 1.0.0
Future<bool> isAbsolute(final String path) async => invokeTauriCommand<bool>(
      TauriCommand(
        tauriModule: TauriModule.Path,
        message: TauriCommandMessage(cmd: 'isAbsolute', path: path),
      ),
    );
