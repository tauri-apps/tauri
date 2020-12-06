export declare enum BaseDirectory {
    Audio = 1,
    Cache = 2,
    Config = 3,
    Data = 4,
    LocalData = 5,
    Desktop = 6,
    Document = 7,
    Download = 8,
    Executable = 9,
    Font = 10,
    Home = 11,
    Picture = 12,
    Public = 13,
    Runtime = 14,
    Template = 15,
    Video = 16,
    Resource = 17,
    App = 18
}
export interface FsOptions {
    dir?: BaseDirectory;
}
export interface FsTextFileOption {
    path: string;
    contents: string;
}
export interface FsBinaryFileOption {
    path: string;
    contents: ArrayBuffer;
}
export interface FileEntry {
    path: string;
    name?: string;
    children?: FileEntry[];
}
/**
 * @name readTextFile
 * @description Reads a file as text
 * @param {string} filePath path to the file
 * @param {FsOptions} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<string>}
 */
declare function readTextFile(filePath: string, options?: FsOptions): Promise<string>;
/**
 * @name readBinaryFile
 * @description Reads a file as binary
 * @param {string} filePath path to the file
 * @param {FsOptions} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<number[]>}
 */
declare function readBinaryFile(filePath: string, options?: FsOptions): Promise<number[]>;
/**
 * writes a text file
 *
 * @param file
 * @param file.path path of the file
 * @param file.contents contents of the file
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
declare function writeFile(file: FsTextFileOption, options?: FsOptions): Promise<void>;
/**
 * writes a binary file
 *
 * @param file
 * @param file.path path of the file
 * @param file.contents contents of the file
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
declare function writeBinaryFile(file: FsBinaryFileOption, options?: FsOptions): Promise<void>;
/**
 * list directory files
 *
 * @param dir path to the directory to read
 * @param [options] configuration object
 * @param [options.recursive] whether to list dirs recursively or not
 * @param [options.dir] base directory
 * @return
 */
declare function readDir(dir: string, options?: FsOptions): Promise<FileEntry[]>;
/**
 * Creates a directory
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, it will be rejected
 *
 * @param dir path to the directory to create
 * @param [options] configuration object
 * @param [options.recursive] whether to create the directory's parent components or not
 * @param [options.dir] base directory
 * @return
 */
declare function createDir(dir: string, options?: FsOptions): Promise<void>;
/**
 * Removes a directory
 * If the directory is not empty and the `recursive` option isn't set to true, it will be rejected
 *
 * @param dir path to the directory to remove
 * @param [options] configuration object
 * @param [options.recursive] whether to remove all of the directory's content or not
 * @param [options.dir] base directory
 * @return
 */
declare function removeDir(dir: string, options?: FsOptions): Promise<void>;
/**
 * Copy file
 *
 * @param source
 * @param destination
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
declare function copyFile(source: string, destination: string, options?: FsOptions): Promise<void>;
/**
 * Removes a file
 *
 * @param file path to the file to remove
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
declare function removeFile(file: string, options?: FsOptions): Promise<void>;
/**
 * Renames a file
 *
 * @param oldPath
 * @param newPath
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
declare function renameFile(oldPath: string, newPath: string, options?: FsOptions): Promise<void>;
export { BaseDirectory as Dir, readTextFile, readBinaryFile, writeFile, writeBinaryFile, readDir, createDir, removeDir, copyFile, removeFile, renameFile };
