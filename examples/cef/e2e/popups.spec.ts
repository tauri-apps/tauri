// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { APP_ORIGIN } from './app.js'
import { entries, expect, sample, test, webviewLabels } from './fixtures.js'

const CEF_OWNED_URL = `${APP_ORIGIN}/popup.html#cef-owned`

/**
 * Opens the CEF-owned popup. Not a click on its button: see `evaluateDetached`
 * for why the popup has to be opened, and loaded, with no Playwright connection
 * around. The app's own native snapshot says when it has loaded.
 */
const openCefOwnedPopup = `document.querySelector('[data-popup="popup.html#cef-owned"]').click()`
const cefOwnedPopupLoaded = `window.__TAURI__.core
  .invoke('native_snapshot', { options: { label: null } })
  .then((snapshot) =>
    snapshot.popups.some(
      (popup) => popup.browser.mainFrameUrl && !popup.browser.isLoading
    )
  )`

test.describe('Popups: AsCefWindowOpener', () => {
  // Answering `NewWindowResponse::Create` from `on_new_window` wedges the
  // browser process on macOS as of this writing: the handler builds the Tauri
  // window from inside CEF's `on_before_popup`, on the UI thread, while the
  // opener's renderer sits in its synchronous new-window request. It reproduces
  // with no DevTools client attached at all (the window is opened from a
  // `setTimeout` after the only client disconnected), so it is not the suite's
  // doing. The CEF-owned path below is unaffected.
  test.fixme('window.open becomes a Tauri window, with the opener CEF reported', async ({
    app,
    main
  }) => {
    await main.locator('[data-popup="popup.html"]').click()

    // A window of the app's own — so a page on this connection like any other.
    const popup = await app.waitForPage(
      (url) => url.pathname === '/popup.html' && url.hash === ''
    )
    await expect(popup.locator('#kind')).toContainText(
      'A Tauri window the new-window handler built'
    )
    // Built by the app, not by CEF, so it has no opener relationship.
    await expect(popup.locator('#opener')).toHaveText('null')

    // The new-window callback was told the opener's URL by CEF directly.
    const entry = entries(main, 'popup-log', {
      tag: 'new window',
      text: `${APP_ORIGIN}/popup.html`
    })
    await expect(entry).toHaveCount(1)
    await expect(entry).toContainText(`opener: ${APP_ORIGIN}/`)
    await expect(entry).toContainText('answered with: popup-1')
    expect(await webviewLabels(main)).toEqual(['main', 'popup-1'])

    // Closing the page over the protocol closes the Tauri window.
    await popup.close()
    await expect.poll(() => webviewLabels(main)).toEqual(['main'])
  })

  test('#cef-owned leaves the popup to CEF, and the opener sees it under popups()', async ({
    app
  }) => {
    await app.evaluateDetached(openCefOwnedPopup, cefOwnedPopupLoaded)
    const main = app.main

    // The new-window callback was told the opener's URL by CEF directly, and
    // answered `Allow`: no Tauri window came of it.
    const entry = entries(main, 'popup-log', {
      tag: 'new window',
      text: '#cef-owned'
    })
    await expect(entry).toContainText(`opener: ${APP_ORIGIN}/`)
    await expect(entry).toContainText(
      'answered with: (CEF-owned, no Tauri label)'
    )
    expect(await webviewLabels(main)).toEqual(['main'])

    // The opener's snapshot observes it instead: a browser of its own, opened
    // by this one, which selects itself back out of the family by its own
    // document token.
    const snapshot = await sample(main)
    expect(snapshot.popups).toHaveLength(1)
    const [observed] = snapshot.popups
    expect(observed.openedByThisBrowser).toBe(true)
    expect(observed.browserId).not.toBe(snapshot.browserId)
    expect(observed.browser.mainFrameUrl).toBe(CEF_OWNED_URL)
    await expect
      .poll(async () => (await sample(main)).popups[0]?.selectedByItsDocument)
      .toBe(true)

    // Closed through the DevTools server — Playwright has no usable page for
    // it, see below — the opener stops seeing it.
    await app.closeTarget((url) => url.href === CEF_OWNED_URL)
    await expect.poll(async () => (await sample(main)).popups).toEqual([])
  })

  // The browser CEF opens gets none of the runtime's request handling, so the
  // app's own origin is refused where a Tauri window loads it: instead of
  // popup.html the popup shows Chromium's "tauri.localhost refused to connect"
  // page, and CEF logs "Chrome style is not supported for this browser".
  test.fixme('the CEF-owned popup loads the app page it asked for', async ({
    app
  }) => {
    await app.evaluateDetached(openCefOwnedPopup, cefOwnedPopupLoaded)
    const popup = await app.waitForPage((url) => url.href === CEF_OWNED_URL)
    await expect(popup.locator('#kind')).toContainText('Opened by CEF itself')
    // Opened by CEF from the page's own `window.open`, so the opener is linked.
    await expect(popup.locator('#opener')).toHaveText('set')
  })
})
