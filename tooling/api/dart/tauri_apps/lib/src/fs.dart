// ignore_for_file: constant_identifier_names

/// Access the file system.
///
/// This package is also accessible with `window.__TAURI__.fs` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be added to
/// [`tauri.allowlist.fs`](https://tauri.app/v1/api/config/#allowlistconfig.fs)
/// in `tauri.conf.json`:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "fs": {
///         "all": true, // enable all FS APIs
///         "readFile": true,
///         "writeFile": true,
///         "readDir": true,
///         "copyFile": true,
///         "createDir": true,
///         "removeDir": true,
///         "removeFile": true,
///         "renameFile": true,
///         "exists": true
///       }
///     }
///   }
/// }
/// ```
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
///
/// ## Security
///
/// This module prevents path traversal, not allowing absolute paths or parent
/// dir components (i.e. "/usr/path/to/file" or "../path/to/file" paths are not
/// allowed).
/// Paths accessed with this API must be relative to one of the [BaseDirectory]
/// so if you need access to arbitrary filesystem paths, you must write such
/// logic on the core layer instead.
///
/// The API has a scope configuration that forces you to restrict the paths that
/// can be accessed using glob patterns.
///
/// The scope configuration is an array of glob patterns describing folder paths
/// that are allowed.
///
/// For instance, this scope configuration only allows accessing files on the
/// *databases* folder of the {@link path.appDataDir | $APPDATA directory}:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "fs": {
///         "scope": ["$APPDATA/databases/*"]
///       }
///     }
///   }
/// }
/// ```
///
/// Notice the use of the `$APPDATA` variable. The value is injected at runtime,
/// resolving to the {@link path.appDataDir | app data directory}.
///
/// The available variables are:
///   * {@link path.appConfigDir | `$APPCONFIG`},
///     {@link path.appDataDir | `$APPDATA`},
///     {@link path.appLocalDataDir | `$APPLOCALDATA`},
///   * {@link path.appCacheDir | `$APPCACHE`},
///     {@link path.appLogDir | `$APPLOG`},
///   * {@link path.audioDir | `$AUDIO`},
///     {@link path.cacheDir | `$CACHE`},
///     {@link path.configDir | `$CONFIG`},
///     {@link path.dataDir | `$DATA`},
///   * {@link path.localDataDir | `$LOCALDATA`},
///     {@link path.desktopDir | `$DESKTOP`},
///     {@link path.documentDir | `$DOCUMENT`},
///   * {@link path.downloadDir | `$DOWNLOAD`},
///     {@link path.executableDir | `$EXE`},
///     {@link path.fontDir | `$FONT`},
///     {@link path.homeDir | `$HOME`},
///   * {@link path.pictureDir | `$PICTURE`},
///     {@link path.publicDir | `$PUBLIC`},
///     {@link path.runtimeDir | `$RUNTIME`},
///   * {@link path.templateDir | `$TEMPLATE`},
///     {@link path.videoDir | `$VIDEO`},
///     {@link path.resourceDir | `$RESOURCE`},
///     {@link path.appDir | `$APP`},
///   * {@link path.logDir | `$LOG`},
///     {@link os.tempdir | `$TEMP`}.
///
/// Trying to execute any API with a URL not configured on the scope results in
/// a future rejection due to denied access.
///
/// Note that this scope applies to **all** APIs on this module.
library tauri_fs;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';
import 'helpers/tauri.dart'
    show TauriCommand, TauriCommandMessage, TauriModule, invokeTauriCommand;

class BinaryFileContents<T> {
  factory BinaryFileContents(final dynamic data) {
    if (data is Iterable<int> || data is ByteBuffer) {
      return BinaryFileContents<T>._(data as T);
    }

    throw UnsupportedError(
      'data must be of type Iterbale<int> || ByteBuffer, '
      'but ${data.runtimeType} was provided.',
    );
  }

  const BinaryFileContents._(this.data);

  final T data;
}

/// @since 1.0.0
enum BaseDirectory {
  App,
  AppCache,
  AppConfig,
  AppData,
  AppLocalData,
  AppLog,
  Audio(1),
  Cache,
  Config,
  Data,
  Desktop,
  Document,
  Download,
  Executable,
  Font,
  Home,
  LocalData,
  Log,
  Picture,
  Public,
  Resource,
  Runtime,
  Temp,
  Template,
  Video;

  const BaseDirectory([this.id]);

  final int? id;
}

/// @since 1.0.0
class FsOptions {
  const FsOptions({this.dir});

  final BaseDirectory? dir;

  /// note that adding fields here needs a change in the [writeBinaryFile] check
}

/// @since 1.0.0
class FsDirOptions {
  const FsDirOptions({this.dir, this.recursive = false});

  final BaseDirectory? dir;
  final bool recursive;
}

/// Options object used to write a UTF-8 string to a file.
///
/// @since 1.0.0
class FsTextFileOption {
  const FsTextFileOption({required this.path, required this.contents});

  /// Path to the file to write.
  final String path;

  /// The UTF-8 string to write to the file.
  final String contents;
}

/// Options object used to write a binary data to a file.
///
/// @since 1.0.0
class FsBinaryFileOption {
  const FsBinaryFileOption({required this.path, required this.contents});

  /// Path to the file to write.
  final String path;

  /// The byte array contents.
  final BinaryFileContents contents;
}

/// @since 1.0.0
abstract class FileEntry {
  const FileEntry({
    required this.path,
    this.name,
    this.children,
  });

  /// Path to the file to write.
  final String path;

  /// Name of the directory/file can be null if the path terminates with `..`
  final String? name;

  /// Children of this entry if it's a directory; null otherwise
  final List<FileEntry>? children;
}

/// Reads a file as an UTF-8 encoded string.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show readTextFile, BaseDirectory;
/// // Read the text file in the `$APPCONFIG/app.conf` path
/// (() async {
///   final String contents = await readTextFile(
///     filePath: 'app.conf',
///     options: FsOptions(
///       dir: BaseDirectory.AppConfig,
///     ),
///   );
/// })();
/// ```
///
/// @since 1.0.0
Future<String> readTextFile({
  required final String filePath,
  final FsOptions options = const FsOptions(),
}) async =>
    invokeTauriCommand<String>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'readFile',
          path: filePath,
          options: options,
        ),
      ),
    );

/// Reads a file as byte array.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show readBinaryFile, BaseDirectory;
/// // Read the image file in the `$RESOURCEDIR/avatar.png` path
/// (() async {
///   final Uint8List contents = await readBinaryFile(
///     filePath: 'avatar.png',
///     options: FsOptions(
///       dir: BaseDirectory.Resource,
///     ),
///   );
/// })();
/// ```
///
/// @since 1.0.0
Future<Uint8List> readBinaryFile({
  required final String filePath,
  final FsOptions options = const FsOptions(),
}) async {
  final List<int> arr = await invokeTauriCommand<List<int>>(
    TauriCommand(
      tauriModule: TauriModule.Fs,
      message: TauriCommandMessage(
        cmd: 'readFile',
        path: filePath,
        options: options,
      ),
    ),
  );
  return Uint8List.fromList(arr);
}

/// Writes a UTF-8 text file.
///
/// Either [file] or [path] should be provided. If both are provided, [file]
/// takes precedence.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show writeTextFile, BaseDirectory;
/// // Write a text file to the `$APPCONFIG/app.conf` path
/// (() async {
///   await writeTextFile(
///     path: 'app.conf',
///     contents: 'file contents',
///     options: FsOptions(
///       dir: BaseDirectory.AppConfig,
///     ),
///   );
///
/// // OR
///
///   await writeTextFile(
///     file: FsTextFileOption(
///       path: 'app.conf',
///       contents: 'file contents',
///     ),
///     options: FsOptions(
///       dir: BaseDirectory.AppConfig,
///     ),
///   );
/// })();
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> writeTextFile({
  final FsTextFileOption? file,
  final dynamic path,
  final dynamic contents,
  final FsOptions? options,
}) async {
  FsTextFileOption localFile = const FsTextFileOption(path: '', contents: '');
  dynamic fileOptions;

  if (file != null) {
    localFile = file;
  } else {
    if (path is FsTextFileOption) {
      localFile = path;
    }

    if (path is String) {
      if (contents is String?) {
        localFile = FsTextFileOption(path: path, contents: contents ?? '');
      } else {
        localFile = FsTextFileOption(path: path, contents: '');
        fileOptions = contents;
      }
    }
  }

  fileOptions ??= options;

  return invokeTauriCommand<void>(
    TauriCommand(
      tauriModule: TauriModule.Fs,
      message: TauriCommandMessage(
        cmd: 'writeFile',
        path: localFile.path,
        contents: List<int>.from(
          const Utf8Encoder().convert(localFile.contents),
        ),
        options: fileOptions,
      ),
    ),
  );
}

/// Writes a byte array content to a file.
///
/// Either [file] or [path] should be provided. If both are provided, [file]
/// takes precedence.
///
/// * [path]: Must be of type [String] | [FsBinaryFileOption],
/// * [contents]: Must be of type [BinaryFileContents] | [FsOptions] | null,
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show writeBinaryFile, BaseDirectory;
/// // Write a binary file to the `$APPDATA/avatar.png` path
/// (() async {
///   await writeBinaryFile(
///     path: 'avatar.png',
///     contents: Uint8List.fromList(<int>[]),
///     options: FsOptions(
///       dir: BaseDirectory.AppData,
///     ),
///   );
///
/// // OR
///
///   await writeBinaryFile(
///     file: FsBinaryFileOption(
///       path: 'avatar.png',
///       contents: Uint8List.fromList(<int>[]),
///     ),
///     options: FsOptions(
///       dir: BaseDirectory.AppData,
///     ),
///   );
/// })();
/// ```
///
/// * [options] Configuration object.
/// * [file] The object containing the file path and contents.
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> writeBinaryFile({
  final FsBinaryFileOption? file,
  final dynamic path,
  final dynamic contents,
  final FsOptions? options,
}) async {
  FsBinaryFileOption localFile = FsBinaryFileOption(
    path: '',
    contents: BinaryFileContents<Iterable<int>>(<int>[]),
  );
  dynamic fileOptions;

  if (file != null) {
    localFile = file;
  } else {
    if (path is FsBinaryFileOption) {
      localFile = path;
    } else if (path is String) {
      localFile = FsBinaryFileOption(path: path, contents: contents ?? <int>[]);
    }
  }

  if (contents is FsOptions && contents.dir != null) {
    fileOptions = contents;
  }

  fileOptions ??= options;

  return invokeTauriCommand<void>(
    TauriCommand(
      tauriModule: TauriModule.Fs,
      message: TauriCommandMessage(
        cmd: 'writeFile',
        path: localFile.path,
        contents: List<int>.from(
          localFile.contents.data is Uint8List
              ? localFile.contents.data
              : localFile.contents.data is Iterable<int>
                  ? Uint8List.fromList(
                      (localFile.contents.data as Iterable<int>).toList(),
                    )
                  : Uint8List.view(localFile.contents.data as ByteBuffer),
        ),
        options: fileOptions,
      ),
    ),
  );
}

/// List directory files.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show readDir, BaseDirectory;
/// // Reads the `$APPDATA/users` directory recursively
/// (()) async {
///   final List<FileEntry> entries = await readDir(
///     dir: 'users',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppData,
///       recursive: true,
///     ),
///   );
/// }();
///
/// void processEntries(final List<FileEntry> entries) {
///   for (final FileEntry entry in entries) {
///     print('Entry: ${entry.path}');
///     if (entry.children != null && entry.children.isNotEmpty) {
///       processEntries(entry.children);
///     }
///   }
/// }
/// ```
///
/// @since 1.0.0
Future<List<FileEntry>> readDir({
  required final String dir,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<List<FileEntry>>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'readDir',
          path: dir,
          options: options,
        ),
      ),
    );

/// Creates a directory.
/// If one of the path's parent components doesn't exist
/// and the `recursive` option isn't set to true, the future will be rejected.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show createDir, BaseDirectory;
/// // Create the `$APPDATA/users` directory
/// (() async {
///   await createDir(
///     dir: 'users',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppData,
///       recursive: true,
///     ),
///   );
/// })();
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> createDir({
  required final String dir,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'createDir',
          path: dir,
          options: options,
        ),
      ),
    );

/// Removes a directory.
///
/// If the directory is not empty and the `recursive` option isn't set to true,
/// the future will be rejected.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show removeDir, BaseDirectory;
/// // Remove the directory `$APPDATA/users`
/// (() async {
///   await removeDir(
///     dir: 'users',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppData,
///     ),
///   );
/// })();
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> removeDir({
  required final String dir,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'removeDir',
          path: dir,
          options: options,
        ),
      ),
    );

/// Copies a file to a destination.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show copyFile, BaseDirectory;
/// // Copy the `$APPCONFIG/app.conf` file to `$APPCONFIG/app.conf.bk`
/// (() async {
///   await copyFile(
///     source: 'app.conf',
///     destination: 'app.conf.bk',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppConfig,
///     ),
///   );
/// })();
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> copyFile({
  required final String source,
  required final String destination,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'copyFile',
          source: source,
          destination: destination,
          options: options,
        ),
      ),
    );

/// Removes a file.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show removeFile, BaseDirectory;
/// // Remove the `$APPConfig/app.conf` file
/// (() async {
///   await removeFile(
///     file: 'app.conf',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppConfig,
///     ),
///   );
/// })();
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> removeFile({
  required final String file,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'removeFile',
          path: file,
          options: options,
        ),
      ),
    );

/// Renames a file.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show renameFile, BaseDirectory;
/// // Rename the `$APPDATA/avatar.png` file
/// (() async {
///   await renameFile(
///     oldPath: 'avatar.png',
///     newPath: 'deleted.png',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppData,
///     ),
///   );
/// })();
/// ```
///
/// @returns A future indicating the success or failure of the operation.
///
/// @since 1.0.0
Future<void> renameFile({
  required final String oldPath,
  required final String newPath,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<void>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'renameFile',
          oldPath: oldPath,
          newPath: newPath,
          options: options,
        ),
      ),
    );

/// Check if a path exists.
///
/// ```dart
/// import 'package:tauri_apps/api/fs.dart' show exists, BaseDirectory;
/// // Check if the `$APPDATA/avatar.png` file exists
/// (() async {
///   await exists(
///     path: 'avatar.png',
///     options: FsDirOptions(
///       dir: BaseDirectory.AppData,
///     ),
///   );
/// })();
/// ```
///
/// @since 1.1.0
Future<bool> exists({
  required final String path,
  final FsDirOptions options = const FsDirOptions(),
}) async =>
    invokeTauriCommand<bool>(
      TauriCommand(
        tauriModule: TauriModule.Fs,
        message: TauriCommandMessage(
          cmd: 'exists',
          path: path,
          options: options,
        ),
      ),
    );
