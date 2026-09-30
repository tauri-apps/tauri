// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// `@tauri-apps/api` e2e suite against the examples/api app on an Android
// device or emulator, through Appium's UiAutomator2 driver. See wdio.mobile.ts.

import { mobileConfig } from './wdio.mobile.js'

export const config = mobileConfig('android')
