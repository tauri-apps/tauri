// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { entries, expect, test } from './fixtures.js'

test.describe('Permissions: on_permission_request', () => {
  test("notifications are granted by the handler, before Chrome's prompt, once per origin", async ({
    main
  }) => {
    await main.locator('[data-permission="notifications"]').click()

    const decided = entries(main, 'permission-log', {
      tag: 'allow',
      text: 'Notifications'
    })
    await expect(decided).toHaveCount(1)
    await expect(decided).toContainText("granted without Chrome's prompt")
    await expect(
      entries(main, 'permission-log', {
        tag: 'page',
        text: 'notifications: granted'
      })
    ).toHaveCount(1)
    expect(await main.evaluate(() => Notification.permission)).toBe('granted')

    // The decision is stored in the profile: a second request is answered from
    // the stored setting and never reaches the handler again.
    await main.locator('[data-permission="notifications"]').click()
    await expect(
      entries(main, 'permission-log', {
        tag: 'page',
        text: 'notifications: granted'
      })
    ).toHaveCount(2)
    await expect(decided).toHaveCount(1)
  })

  test('geolocation reaches the handler too', async ({ main }) => {
    await main.locator('[data-permission="geolocation"]').click()
    // Whether a position is then available is up to the machine; what is
    // certain is that the handler was consulted, and what it answered.
    await expect(
      entries(main, 'permission-log', { tag: 'allow', text: 'Geolocation' })
    ).toHaveCount(1)
  })
})
