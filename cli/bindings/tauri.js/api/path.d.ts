import { BaseDirectory } from './fs';
/**
 * @name appDir
 * @description Returns the path to the suggested directory for your app config files.
 * @return {Promise<string>}
 */
declare function appDir(): Promise<string>;
/**
 * @name audioDir
 * @description Returns the path to the user's audio directory.
 * @return {Promise<string>}
 */
declare function audioDir(): Promise<string>;
/**
 * @name cacheDir
 * @description Returns the path to the user's cache directory.
 * @return {Promise<string>}
 */
declare function cacheDir(): Promise<string>;
/**
 * @name configDir
 * @description Returns the path to the user's config directory.
 * @return {Promise<string>}
 */
declare function configDir(): Promise<string>;
/**
 * @name dataDir
 * @description Returns the path to the user's data directory.
 * @return {Promise<string>}
 */
declare function dataDir(): Promise<string>;
/**
 * @name desktopDir
 * @description Returns the path to the user's desktop directory.
 * @return {Promise<string>}
 */
declare function desktopDir(): Promise<string>;
/**
 * @name documentDir
 * @description Returns the path to the user's document directory.
 * @return {Promise<string>}
 */
declare function documentDir(): Promise<string>;
/**
 * @name downloadDir
 * @description Returns the path to the user's download directory.
 * @return {Promise<string>}
 */
declare function downloadDir(): Promise<string>;
/**
 * @name executableDir
 * @description Returns the path to the user's executable directory.
 * @return {Promise<string>}
 */
declare function executableDir(): Promise<string>;
/**
 * @name fontDir
 * @description Returns the path to the user's font directory.
 * @return {Promise<string>}
 */
declare function fontDir(): Promise<string>;
/**
 * @name homeDir
 * @description Returns the path to the user's home directory.
 * @return {Promise<string>}
 */
declare function homeDir(): Promise<string>;
/**
 * @name localDataDir
 * @description Returns the path to the user's local data directory.
 * @return {Promise<string>}
 */
declare function localDataDir(): Promise<string>;
/**
 * @name pictureDir
 * @description Returns the path to the user's picture directory.
 * @return {Promise<string>}
 */
declare function pictureDir(): Promise<string>;
/**
 * @name publicDir
 * @description Returns the path to the user's public directory.
 * @return {Promise<string>}
 */
declare function publicDir(): Promise<string>;
/**
 * @name resourceDir
 * @description Returns the path to the user's resource directory.
 * @return {Promise<string>}
 */
declare function resourceDir(): Promise<string>;
/**
 * @name runtimeDir
 * @descriptionReturns Returns the path to the user's runtime directory.
 * @return {Promise<string>}
 */
declare function runtimeDir(): Promise<string>;
/**
 * @name templateDir
 * @descriptionReturns Returns the path to the user's template directory.
 * @return {Promise<string>}
 */
declare function templateDir(): Promise<string>;
/**
 * @name videoDir
 * @descriptionReturns Returns the path to the user's video dir.
 * @return {Promise<string>}
 */
declare function videoDir(): Promise<string>;
/**
 * @name resolvePath
 * @descriptionReturns Resolves the path with the optional base directory.
 * @return {Promise<string>}
 */
declare function resolvePath(path: string, directory: BaseDirectory): Promise<string>;
export { appDir, audioDir, cacheDir, configDir, dataDir, desktopDir, documentDir, downloadDir, executableDir, fontDir, homeDir, localDataDir, pictureDir, publicDir, resourceDir, runtimeDir, templateDir, videoDir, resolvePath };
