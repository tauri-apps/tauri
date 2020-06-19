import {
  HttpOptions,
  EventCallback,
  OpenDialogOptions,
  SaveDialogOptions,
  FsOptions,
  FsFileOption,
  FileEntry
} from './models'

declare global {
  interface Window {
    tauri: {
      // window
      setTitle: (title: string) => void
      open: (url: string) => void

      // process
      execute: (command: string, args?: string | string[]) => Promise<string>

      // http
      httpRequest: <T>(options: HttpOptions) => Promise<T>

      // event
      listen: (event: string, handler: EventCallback) => void
      emit: (event: string, payload?: string) => void

      // dialog
      openDialog: (options: OpenDialogOptions) => Promise<String | String[]>
      saveDialog: (options: SaveDialogOptions) => Promise<String>

      // cli
      cliMatches: () => any

      // fs
      readTextFile: (filePath: string, options: FsOptions) => Promise<string>
      readBinaryFile: (filePath: string, options: FsOptions) => Promise<string>
      writeFile: (file: FsFileOption, options: FsOptions) => Promise<void>
      readDir: (dir: string, options: FsOptions) => Promise<FileEntry[]>
      createDir: (dir: string, options: FsOptions) => Promise<void>
      removeDir: (dir: string, options: FsOptions) => Promise<void>
      copyFile: (source: string, destination: string, options: FsOptions) => Promise<void>
      removeFile: (file: string, options: FsOptions) => Promise<void>
      renameFile: (oldPath: string, newPath: string, options: FsOptions) => Promise<void>
    }
  }
}

export default window.tauri
