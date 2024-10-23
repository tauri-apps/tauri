// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

window.__TAURI_ISOLATION_HOOK__ = (payload, options) => {
  console.log('hook', payload, options)
  return payload
}
