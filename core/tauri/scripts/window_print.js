window.print = function () {
  return window.__TAURI_INVOKE__('tauri', {
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'print'
        }
      }
    }
  })
}
