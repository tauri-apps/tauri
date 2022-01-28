import { BaseDirectory } from './fs';
/**
 * The path module provides utilities for working with file and directory paths.
 *
 * This package is also accessible with `window.__TAURI__.path` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "path": {
 *         "all": true, // enable all Path APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */
/**
 * Returns the path to the suggested directory for your app config files.
 * Resolves to `${configDir}/${bundleIdentifier}`, where `bundleIdentifier` is the value configured on `tauri.conf.json > tauri > bundle > identifier`.
 *
 * @returns
 */
declare function appDir(): Promise<string>;
/**
 * Returns the path to the user's audio directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_MUSIC_DIR`.
 * - **macOS:** Resolves to `$HOME/Music`.
 * - **Windows:** Resolves to `{FOLDERID_Music}`.
 *
 * @returns
 */
declare function audioDir(): Promise<string>;
/**
 * Returns the path to the user's cache directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_CACHE_HOME` or `$HOME/.cache`.
 * - **macOS:** Resolves to `$HOME/Library/Caches`.
 * - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
 *
 * @returns
 */
declare function cacheDir(): Promise<string>;
/**
 * Returns the path to the user's config directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`.
 * - **macOS:** Resolves to `$HOME/Library/Application Support`.
 * - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
 *
 * @returns
 */
declare function configDir(): Promise<string>;
/**
 * Returns the path to the user's data directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
 * - **macOS:** Resolves to `$HOME/Library/Application Support`.
 * - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
 *
 * @returns
 */
declare function dataDir(): Promise<string>;
/**
 * Returns the path to the user's desktop directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DESKTOP_DIR`.
 * - **macOS:** Resolves to `$HOME/Library/Desktop`.
 * - **Windows:** Resolves to `{FOLDERID_Desktop}`.

 * @returns
 */
declare function desktopDir(): Promise<string>;
/**
 * Returns the path to the user's document directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOCUMENTS_DIR`.
 * - **macOS:** Resolves to `$HOME/Documents`.
 * - **Windows:** Resolves to `{FOLDERID_Documents}`.
 *
 * @returns
 */
declare function documentDir(): Promise<string>;
/**
 * Returns the path to the user's download directory.
 *
 * #### Platform-specific
 *
 * - **Linux**: Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOWNLOAD_DIR`.
 * - **macOS**: Resolves to `$HOME/Downloads`.
 * - **Windows**: Resolves to `{FOLDERID_Downloads}`.
 *
 * @returns
 */
declare function downloadDir(): Promise<string>;
/**
 * Returns the path to the user's executable directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_BIN_HOME/../bin` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`.
 * - **macOS:** Not supported.
 * - **Windows:** Not supported.
 *
 * @returns
 */
declare function executableDir(): Promise<string>;
/**
 * Returns the path to the user's font directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_DATA_HOME/fonts` or `$HOME/.local/share/fonts`.
 * - **macOS:** Resolves to `$HOME/Library/Fonts`.
 * - **Windows:** Not supported.
 *
 * @returns
 */
declare function fontDir(): Promise<string>;
/**
 * Returns the path to the user's home directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$HOME`.
 * - **macOS:** Resolves to `$HOME`.
 * - **Windows:** Resolves to `{FOLDERID_Profile}`.
 *
 * @returns
 */
declare function homeDir(): Promise<string>;
/**
 * Returns the path to the user's local data directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
 * - **macOS:** Resolves to `$HOME/Library/Application Support`.
 * - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
 *
 * @returns
 */
declare function localDataDir(): Promise<string>;
/**
 * Returns the path to the user's picture directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PICTURES_DIR`.
 * - **macOS:** Resolves to `$HOME/Pictures`.
 * - **Windows:** Resolves to `{FOLDERID_Pictures}`.
 *
 * @returns
 */
declare function pictureDir(): Promise<string>;
/**
 * Returns the path to the user's public directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PUBLICSHARE_DIR`.
 * - **macOS:** Resolves to `$HOME/Public`.
 * - **Windows:** Resolves to `{FOLDERID_Public}`.
 *
 * @returns
 */
declare function publicDir(): Promise<string>;
/**
 * Returns the path to the user's resource directory.
 *
 * @returns
 */
declare function resourceDir(): Promise<string>;
/**
 * Returns the path to the user's runtime directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_RUNTIME_DIR`.
 * - **macOS:** Not supported.
 * - **Windows:** Not supported.
 *
 * @returns
 */
declare function runtimeDir(): Promise<string>;
/**
 * Returns the path to the user's template directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_TEMPLATES_DIR`.
 * - **macOS:** Not supported.
 * - **Windows:** Resolves to `{FOLDERID_Templates}`.
 *
 * @returns
 */
declare function templateDir(): Promise<string>;
/**
 * Returns the path to the user's video directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_VIDEOS_DIR`.
 * - **macOS:** Resolves to `$HOME/Movies`.
 * - **Windows:** Resolves to `{FOLDERID_Videos}`.
 *
 * @returns
 */
declare function videoDir(): Promise<string>;
/**
 * Returns the path to the current working directory.
 *
 * @returns
 */
declare function currentDir(): Promise<string>;
/**
 * Returns the path to the suggested log directory.
 *
 * ### Platform-specific
 *
 * - **Linux:** Resolves to `${configDir}/${bundleIdentifier}`.
 * - **macOS:** Resolves to `${homeDir}//Library/Logs/{bundleIdentifier}`
 * - **Windows:** Resolves to `${configDir}/${bundleIdentifier}`.
 *
 * @returns
 */
declare function logDir(): Promise<string>;
/**
 * Provides the platform-specific path segment separator:
 * - `\` on Windows
 * - `/` on POSIX
 */
declare const sep: string;
/**
 * Provides the platform-specific path segment delimiter:
 * - `;` on Windows
 * - `:` on POSIX
 */
declare const delimiter: string;
/**
 * Resolves a sequence of `paths` or `path` segments into an absolute path.
 *
 * @param paths A sequence of paths or path segments.
 */
declare function resolve(...paths: string[]): Promise<string>;
/**
 * Normalizes the given `path`, resolving `'..'` and `'.'` segments and resolve symolic links.
 */
declare function normalize(path: string): Promise<string>;
/**
 *  Joins all given `path` segments together using the platform-specific separator as a delimiter, then normalizes the resulting path.
 *
 * @param paths A sequence of path segments.
 */
declare function join(...paths: string[]): Promise<string>;
/**
 * Returns the directory name of a `path`. Trailing directory separators are ignored.
 */
declare function dirname(path: string): Promise<string>;
/**
 * Returns the extension of the `path`.
 */
declare function extname(path: string): Promise<string>;
/**
 *  Returns the last portion of a `path`. Trailing directory separators are ignored.
 *
 * @param ext An optional file extension to be removed from the returned path.
 */
declare function basename(path: string, ext?: string): Promise<string>;
declare function isAbsolute(path: string): Promise<boolean>;
export { appDir, audioDir, cacheDir, configDir, dataDir, desktopDir, documentDir, downloadDir, executableDir, fontDir, homeDir, localDataDir, pictureDir, publicDir, resourceDir, runtimeDir, templateDir, videoDir, currentDir, logDir, BaseDirectory, sep, delimiter, resolve, normalize, join, dirname, extname, basename, isAbsolute };
