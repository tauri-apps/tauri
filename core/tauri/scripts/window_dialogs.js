window.alert = function (message) {
  window.__TAURI_INVOKE__('tauri', {
    __tauriModule: 'Dialog',
    message: {
      cmd: 'messageDialog',
      message: message.toString()
    }
  })
}

window.confirm = function (message) {
  return window.__TAURI_INVOKE__('tauri', {
    __tauriModule: 'Dialog',
    message: {
      cmd: 'confirmDialog',
      message: message.toString()
    }
  })
}
