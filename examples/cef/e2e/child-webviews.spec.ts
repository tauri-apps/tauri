// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { entries, expect, sample, test, webviewLabels } from './fixtures.js'

test.describe('Child webviews: WebviewBuilderCefExt', () => {
  test('one window hosts a Chrome style and an Alloy style browser side by side', async ({
    app,
    main
  }) => {
    await main.locator('#open-children').click()
    await expect(
      entries(main, 'popup-log', { tag: 'window', text: 'children-1' })
    ).toContainText('two child webviews')

    // Two browsers, two page targets, one Tauri window.
    const chrome = await app.waitForPage(
      (url) => url.searchParams.get('style') === 'Chrome'
    )
    const alloy = await app.waitForPage(
      (url) => url.searchParams.get('style') === 'Alloy'
    )
    await expect(chrome.locator('#style-name')).toHaveText('Chrome style')
    await expect(alloy.locator('#style-name')).toHaveText('Alloy style')
    expect(await webviewLabels(main)).toEqual([
      'children-1-alloy',
      'children-1-chrome',
      'main'
    ])

    // Each child's console output reaches the main window, under the child's
    // own label rather than a window's.
    await chrome.locator('#say').click()
    await expect(
      entries(main, 'console-log', {
        text: 'hello from the Chrome style child webview'
      })
    ).toContainText('(children-1-chrome')
    await alloy.locator('#say').click()
    await expect(
      entries(main, 'console-log', {
        text: 'hello from the Alloy style child webview'
      })
    ).toContainText('(children-1-alloy')
  })

  test('either child can be sampled by its own label from the main window', async ({
    main
  }) => {
    await main.locator('#open-children').click()

    const chrome = await sample(main, 'children-1-chrome')
    const alloy = await sample(main, 'children-1-alloy')
    // Two browsers in one Tauri window: the snapshot names the window.
    expect(chrome.windowLabel).toBe('children-1')
    expect(alloy.windowLabel).toBe('children-1')
    expect(chrome.browserId).not.toBe(alloy.browserId)
    expect(chrome.nativeWindowObserved).toBe(true)
    expect(alloy.nativeWindowObserved).toBe(true)
    expect(chrome.browser.mainFrameUrl).toContain('child.html?style=Chrome')
    expect(alloy.browser.mainFrameUrl).toContain('child.html?style=Alloy')

    // A second sample of the same label is the same native window.
    expect(
      (await sample(main, 'children-1-alloy')).sameNativeWindowAsPreviousSample
    ).toBe(true)
  })
})
