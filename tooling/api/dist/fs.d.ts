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
    App = 18,
    Current = 19,
    Log = 20
}
interface FsOptions {
    dir?: BaseDirectory;
}
interface FsDirOptions {
    dir?: BaseDirectory;
    recursive?: boolean;
}
interface FsTextFileOption {
    path: string;
    contents: string;
}
interface FsBinaryFileOption {
    path: string;
    contents: ArrayBuffer;
}
interface FileEntry {
    path: string;
    /**
     * Name of the directory/file
     * can be null if the path terminates with `..`
     */
    name?: string;
    /** Children of this entry if it's a directory; null otherwise */
    children?: FileEntry[];
}
/**
 * Reads a file as UTF-8 encoded string.
 *
 * @param filePath Path to the file.
 * @param options Configuration object.
 * @returns A promise resolving to the file content as a UTF-8 encoded string.
 */
declare function readTextFile(filePath: string, options?: FsOptions): Promise<string>;
/**
 * Reads a file as byte array.
 *
 * @param filePath Path to the file.
 * @param options Configuration object.
 * @returns A promise resolving to the file bytes array.
 */
declare function readBinaryFile(filePath: string, options?: FsOptions): Promise<number[]>;
/**
 * Writes a text file.
 *
 * @param file File configuration object.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function writeFile(file: FsTextFileOption, options?: FsOptions): Promise<void>;
/**
 * Writes a binary file.
 *
 * @param file Write configuration object.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function writeBinaryFile(file: FsBinaryFileOption, options?: FsOptions): Promise<void>;
/**
 * List directory files.
 *
 * @param dir Path to the directory to read.
 * @param options Configuration object.
 * @returns A promise resolving to the directory entries.
 */
declare function readDir(dir: string, options?: FsDirOptions): Promise<FileEntry[]>;
/**
 * Creates a directory.
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, the promise will be rejected.
 *
 * @param dir Path to the directory to create.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function createDir(dir: string, options?: FsDirOptions): Promise<void>;
/**
 * Removes a directory.
 * If the directory is not empty and the `recursive` option isn't set to true, the promise will be rejected.
 *
 * @param dir Path to the directory to remove.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function removeDir(dir: string, options?: FsDirOptions): Promise<void>;
/**
 * Copys a file to a destination.
 *
 * @param source A path of the file to copy.
 * @param destination A path for the destination file.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function copyFile(source: string, destination: string, options?: FsOptions): Promise<void>;
/**
 * Removes a file.
 *
 * @param file Path to the file to remove.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function removeFile(file: string, options?: FsOptions): Promise<void>;
/**
 * Renames a file.
 *
 * @param oldPath A path of the file to rename.
 * @param newPath A path of the new file name.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
 */
declare function renameFile(oldPath: string, newPath: string, options?: FsOptions): Promise<void>;
export type { FsOptions, FsDirOptions, FsTextFileOption, FsBinaryFileOption, FileEntry };
export { BaseDirectory as Dir, readTextFile, readBinaryFile, writeFile, writeBinaryFile, readDir, createDir, removeDir, copyFile, removeFile, renameFile };
