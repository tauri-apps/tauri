// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function () {
  Object.defineProperty(window, '__TAURI_POST_MESSAGE__', {
    value: (message) => {
      const { cmd, callback, error, payload } = message
      const { data } = __RAW_process_ipc_message_fn__({ cmd, callback, error, ...payload })
      window.ipc.postMessage(data)
    }
  })
})()
