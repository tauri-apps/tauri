// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect, sample, test } from './fixtures.js'

test.describe('Native state: with_cef_webview', () => {
  test("samples the calling webview on CEF's UI thread", async ({ main }) => {
    const first = await sample(main)
    expect(first.windowLabel).toBe('main')
    expect(first.browserId).toBeGreaterThan(0)
    expect(first.nativeWindowObserved).toBe(true)
    // Nothing to compare the native window with yet.
    expect(first.sameNativeWindowAsPreviousSample).toBeNull()
    expect(first.visible).toBe(true)
    expect(first.bounds).not.toBeNull()
    // Nothing observed, which is all `known: false` ever says.
    expect(first.dialogs).toEqual({
      known: false,
      kind: null,
      hasBrowserHandler: null
    })
    expect(first.popups).toEqual([])

    // Live state read off the raw `cef::Browser`, agreeing with the protocol
    // client's view of the page.
    expect(first.browser.mainFrameUrl).toBe(main.url())
    // The page and its child frame.
    expect(first.browser.frameCount).toBe(2)
    expect(first.browser.zoomLevel).toBe(0)
    expect(first.browser.devToolsOpen).toBe(false)

    // A document is admitted once every frame is attached and the load is done,
    // and `Webview::for_document` handed its own token selects this webview
    // back out of the family.
    await expect
      .poll(async () => {
        const snapshot = await sample(main)
        return [
          snapshot.documentAdmitted,
          snapshot.selectedByItsDocument,
          snapshot.browser.isLoading
        ]
      })
      .toEqual([true, true, false])

    // The label is not the identity: the token says it is still the same
    // native window as last time.
    expect((await sample(main)).sameNativeWindowAsPreviousSample).toBe(true)
  })

  test('observes a JavaScript dialog on a webview that is not the caller', async ({
    app,
    main
  }) => {
    await main.locator('#open-children').click()
    const child = await app.waitForPage(
      (url) => url.searchParams.get('style') === 'Chrome'
    )

    // Playwright dismisses dialogs nobody listens for; this one has to stay up
    // while the main window samples the webview it is blocking.
    const opened = child.waitForEvent('dialog')
    // Resolves only once the dialog is accepted below.
    const blocked: Promise<void> = child
      .evaluate(() => window.alert('observed from Rust'))
      .catch(() => {})
    const dialog = await opened
    expect(dialog.type()).toBe('alert')

    const snapshot = await sample(main, 'children-1-chrome')
    // The child's own label selected the browser; the snapshot names its window.
    expect(snapshot.windowLabel).toBe('children-1')
    expect(snapshot.dialogs.known).toBe(true)
    // Only the kind: the message is page content and is not retained.
    expect(snapshot.dialogs.kind).toBe('Alert')

    await dialog.accept()
    await blocked
  })
})
