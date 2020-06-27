export enum BaseDirectory {
  Audio = 1,
  Cache,
  Config,
  Data,
  LocalData,
  Desktop,
  Document,
  Download,
  Executable,
  Font,
  Home,
  Picture,
  Public,
  Runtime,
  Template,
  Video,
  Resource,
  App,
}

export interface FsOptions {
  dir?: BaseDirectory
}

export interface FsTextFileOption {
  path: string
  contents: string
}

export interface FsBinaryFileOption {
  path: string
  contents: ArrayBuffer
}

export interface FileEntry {
  path: string
  // name of the directory/file
  // can be null if the path terminates with `..`
  name?: string
  // children of this entry if it's a directory; null otherwise
  children?: FileEntry[]
}
