// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

if (!('path' in window.__TAURI__)) {
  window.__TAURI__.path = {}
}

window.__TAURI__.path.__sep = __TEMPLATE_sep__
window.__TAURI__.path.__delimiter = __TEMPLATE_delimiter__
