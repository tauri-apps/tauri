// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function (message) {
  return JSON.stringify(message, (_k, val) => {
    if (val instanceof Map) {
      let o = {};
      val.forEach((v, k) => o[k] = v);
      return o;
    } else {
      return val;
    }
  })
})
