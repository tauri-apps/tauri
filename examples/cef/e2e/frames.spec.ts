// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { APP_ORIGIN } from './app.js'
import { entries, expect, sample, test } from './fixtures.js'

test.describe('Frame lifecycle: on_frame_event', () => {
  test("a child frame's navigation is reported with its URL, which the portable callbacks never see", async ({
    main
  }) => {
    await main.locator('[data-frame="frame.html?v=2"]').click()

    const url = `${APP_ORIGIN}/frame.html?v=2`
    const started = entries(main, 'frame-log', {
      tag: 'navigationStarted',
      text: url
    })
    await expect(started).toHaveCount(1)
    await expect(started).toContainText('child frame')
    const committed = entries(main, 'frame-log', {
      tag: 'documentCommitted',
      text: url
    })
    await expect(committed).toHaveCount(1)
    await expect(committed).toContainText('child frame')

    // The child frame's console output reaches the same observer as the main
    // frame's, with the child document as its source.
    const hello = entries(main, 'console-log', {
      text: `child frame loaded: ${url}`
    })
    await expect(hello).toHaveCount(1)
    await expect(hello).toContainText('frame.html')
  })

  test('the whole native lifecycle is reported, down to the frame being destroyed', async ({
    main
  }) => {
    await main.locator('[data-frame="remove"]').click()

    await expect(
      entries(main, 'frame-log', { tag: 'detached', text: 'child frame' })
    ).not.toHaveCount(0)
    await expect(
      entries(main, 'frame-log', { tag: 'destroyed', text: 'child frame' })
    ).not.toHaveCount(0)
    // Gone from the DOM, and from the raw browser's frame count.
    await expect(main.locator('#child-frame')).toHaveCount(0)
    await expect
      .poll(async () => (await sample(main)).browser.frameCount)
      .toBe(1)
  })

  test('a failed navigation is reported as a failure, not as a document', async ({
    main
  }) => {
    await main.locator('[data-frame="https://example.invalid"]').click()
    await expect(
      entries(main, 'frame-log', {
        tag: 'navigationFailed',
        text: 'https://example.invalid'
      })
    ).not.toHaveCount(0)
  })
})
