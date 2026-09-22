// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { entries, expect, test } from './fixtures.js'

test.describe('Console messages: on_console_message', () => {
  test('console output reaches Rust with no DevTools open, at its severity', async ({
    main
  }) => {
    // The same message, as the protocol client sees it (`Runtime.consoleAPICalled`)...
    const overTheProtocol = main.waitForEvent('console', (message) =>
      message.text().includes('console.log from the example page')
    )
    await main.locator('[data-console="log"]').click()
    // ...and as the app's observer reported it, tagged with the window it came
    // from and the script that wrote it.
    const entry = entries(main, 'console-log', {
      tag: 'info',
      text: 'console.log from the example page'
    })
    await expect(entry).toHaveCount(1)
    await expect(entry).toContainText('(main, ')
    await expect(entry).toContainText('main.js:')
    expect((await overTheProtocol).type()).toBe('log')

    await main.locator('[data-console="warn"]').click()
    await expect(
      entries(main, 'console-log', {
        tag: 'warning',
        text: 'console.warn from the example page'
      })
    ).toHaveCount(1)

    await main.locator('[data-console="error"]').click()
    await expect(
      entries(main, 'console-log', {
        tag: 'error',
        text: 'console.error from the example page'
      })
    ).toHaveCount(1)
  })

  test('an uncaught exception is console output too, from the renderer itself', async ({
    main
  }) => {
    await main.locator('[data-console="throw"]').click()
    await expect(
      entries(main, 'console-log', {
        tag: 'error',
        text: 'thrown from the example page'
      })
    ).toHaveCount(1)
  })
})
