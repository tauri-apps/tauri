// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// `@tauri-apps/api` e2e suite against the examples/api app on an iOS
// simulator, through Appium's XCUITest driver. See wdio.mobile.ts.

import { mobileConfig } from './wdio.mobile.js'

export const config = mobileConfig('ios')
