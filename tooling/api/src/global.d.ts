/** @ignore */
interface Window {
  __TAURI__: {
    __INTERNALS__: {
      invoke: typeof invoke
      transformCallback: typeof transformCallback
      convertFileSrc: typeof convertFileSrc
      ipc: (message: any) => void
      metadata: {
        windows: WindowDef[]
        currentWindow: WindowDef
      }
      path: {
        sep: string
        delimiter: string
      }
    }
  }
}
