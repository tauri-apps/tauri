// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { entries, expect, test } from './fixtures.js'

test.describe('Deep links: deep_link_schemes', () => {
  test('a deep link delivered to the running app arrives as RunEvent::Opened', async ({
    app,
    main
  }) => {
    // Through Launch Services on macOS; elsewhere by running the executable
    // again with the URL, which Chromium's process singleton relays to this
    // instance on a command line CEF has cleared and the runtime restores.
    await app.openDeepLink('tauri-cef-example://hello')
    await expect(
      entries(main, 'deeplink-log', { tag: 'opened' })
    ).toContainText('tauri-cef-example://hello')
  })
})
