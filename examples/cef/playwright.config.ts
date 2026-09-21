// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  // Every test launches the app and attaches to it over the Chrome DevTools
  // Protocol (see e2e/app.ts): Playwright launches no browser of its own here,
  // and the tests drive real windows on one desktop, so they run one at a time.
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  // Launching CEF is most of a test's time; the assertions themselves wait on
  // the app's own log panels, which fill within a second or so.
  timeout: 90_000,
  expect: { timeout: 15_000 },
  reporter: process.env.CI
    ? [['list'], ['html', { open: 'never' }]]
    : [['list']],
  // Builds the app (`pnpm tauri build --debug`) unless CEF_E2E_SKIP_BUILD or
  // CEF_E2E_APP_PATH is set.
  globalSetup: './e2e/global-setup.ts'
})
