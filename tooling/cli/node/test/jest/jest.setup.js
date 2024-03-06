// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// 30 minute timeout: 20 minutes wasn't always enough for compilation in GitHub Actions.
jest.setTimeout(1800000)

setTimeout(() => {
  // do nothing
}, 1)
