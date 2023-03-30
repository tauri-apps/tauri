// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
