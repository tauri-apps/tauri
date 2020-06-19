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

export interface FsFileOption {
  path: string
  contents: string
}

export interface FileEntry {
  path: string
  // TODO why not camelCase ?
  is_dir: boolean
  name: string
}
