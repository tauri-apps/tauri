// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function (message) {
  return JSON.stringify(message, (_k, val) => {
    if (val instanceof Map) {
      let o = {};
      val.forEach((v, k) => o[k] = v);
      return o;
    } else if (val instanceof Object && '__TAURI_CHANNEL_MARKER__' in val && typeof val.id === 'number') {
      return `__CHANNEL__:${val.id}`
    } else {
      return val;
    }
  })
})
