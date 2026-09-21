// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { entries, expect, sample, test, webviewLabels } from './fixtures.js'

test.describe('Chrome accelerators: allow_chrome_commands', () => {
  test('the window keeps only the families it was built with', async ({
    app,
    main
  }) => {
    await main.locator('#command-groups input[value="document"]').check()
    await main.locator('#command-groups input[value="history"]').check()
    await main.locator('#open-accelerators').click()
    await expect(
      entries(main, 'popup-log', { tag: 'window', text: 'accelerators-1' })
    ).toContainText('keeping [document, history]')

    // The window reads the families back from the initialization script it
    // was built with.
    const window = await app.waitForPage(
      (url) => url.pathname === '/accelerators.html'
    )
    await expect(window.locator('#allowed-summary')).toHaveText(
      'allow_chrome_commands([document, history])'
    )
    const rows = window.locator('#commands tr')
    await expect(rows.filter({ hasText: 'Ctrl+P' })).toContainText(
      'kept — prints this page'
    )
    await expect(rows.filter({ hasText: 'Alt+Left' })).toContainText('kept')
    await expect(rows.filter({ hasText: 'Ctrl+N' })).toContainText(
      'swallowed by the runtime'
    )
    await expect(rows.filter({ hasText: 'Ctrl+H' })).toContainText(
      'swallowed by the runtime'
    )

    // Its console output is reported to the main window under its label.
    await expect(
      entries(main, 'console-log', { text: 'accelerator window ready' })
    ).toContainText('(accelerators-1')
    expect(await webviewLabels(main)).toEqual(['accelerators-1', 'main'])
  })

  test('with nothing ticked every family is swallowed, in Alloy style too', async ({
    app,
    main
  }) => {
    await main.locator('#runtime-style').selectOption('alloy')
    await main.locator('#open-accelerators').click()
    await expect(
      entries(main, 'popup-log', { tag: 'window', text: 'accelerators-1' })
    ).toContainText('keeping [nothing]')

    const window = await app.waitForPage(
      (url) => url.pathname === '/accelerators.html'
    )
    await expect(window.locator('#allowed-summary')).toHaveText(
      'no allowed groups: everything below is swallowed'
    )
    await expect(
      window.locator('#commands tr').filter({ hasText: 'kept' })
    ).toHaveCount(0)

    // An Alloy style browser is a CEF browser like any other to the runtime.
    const snapshot = await sample(main, 'accelerators-1')
    expect(snapshot.windowLabel).toBe('accelerators-1')
    expect(snapshot.browser.mainFrameUrl).toBe(window.url())
  })
})
