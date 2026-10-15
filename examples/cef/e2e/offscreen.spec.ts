// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { Page } from '@playwright/test'
import type { CefApp } from './app.js'
import { expect, sample, test, webviewLabels } from './fixtures.js'

// Windowless rendering is a process-wide CEF setting, off unless asked for.
test.use({ appEnv: { CEF_EXAMPLE_OFFSCREEN: 'on' } })

/** What the `offscreen_stats` command answers with. */
interface Stats {
  paints: number
  popupPaints: number
  frameSize: [number, number] | null
  popupRect: [number, number, number, number] | null
  pixel: [number, number, number, number] | null
}

const isOffscreenPage = (url: URL) => url.pathname === '/offscreen.html'

/** Opens an off-screen window from the main page, and attaches to its page. */
async function open(
  app: CefApp,
  main: Page,
  transparent = false
): Promise<{ label: string; page: Page }> {
  const label = await main.evaluate(
    (transparent) =>
      window.__TAURI__.core.invoke<string>('open_offscreen_window', {
        transparent
      }),
    transparent
  )
  const page = await app.waitForPage(isOffscreenPage)
  await page.waitForLoadState('load')
  return { label, page }
}

/**
 * What CEF painted into the example's render handler, with the view pixel at
 * the given CSS pixel when one is named.
 */
async function stats(
  main: Page,
  page: Page,
  label: string,
  at?: [number, number]
): Promise<Stats> {
  const scale = await page.evaluate(() => window.devicePixelRatio)
  return main.evaluate(
    ([label, at]) =>
      window.__TAURI__.core.invoke<Stats>('offscreen_stats', {
        label,
        x: at?.[0],
        y: at?.[1]
      }),
    [
      label,
      at && [Math.round(at[0] * scale), Math.round(at[1] * scale)]
    ] as const
  )
}

/** The page's viewport, in CSS pixels. */
function viewport(page: Page): Promise<[number, number]> {
  return page.evaluate(() => [window.innerWidth, window.innerHeight])
}

test.describe('Off-screen rendering: Offscreen', () => {
  test('the page is painted into the render handler at the device scale', async ({
    app,
    main
  }) => {
    const { label, page } = await open(app, main)
    const [width, height] = await viewport(page)
    const scale = await page.evaluate(() => window.devicePixelRatio)

    await expect
      .poll(async () => (await stats(main, page, label)).frameSize)
      .toEqual([Math.round(width * scale), Math.round(height * scale)])
    expect((await stats(main, page, label)).paints).toBeGreaterThan(0)

    // The page's marker, painted where the page put it.
    await expect
      .poll(async () => (await stats(main, page, label, [4, 4])).pixel)
      .toEqual([255, 0, 0, 255])
    // Where the page paints nothing, an opaque webview's browser paints white.
    expect(
      (await stats(main, page, label, [width - 4, height - 4])).pixel
    ).toEqual([255, 255, 255, 255])

    // There is no native browser view to inspect, so the snapshot reports the
    // requested layout instead.
    const snapshot = await sample(main, `${label}-page`)
    expect(snapshot.parentMatches).toBeNull()
    expect(snapshot.visible).toBe(true)
  })

  test('a transparent webview paints nothing where the page does not', async ({
    app,
    main
  }) => {
    const { label, page } = await open(app, main, true)
    const [width, height] = await viewport(page)

    await expect
      .poll(async () => (await stats(main, page, label, [4, 4])).pixel)
      .toEqual([255, 0, 0, 255])
    expect(
      (await stats(main, page, label, [width - 4, height - 4])).pixel
    ).toEqual([0, 0, 0, 0])
  })

  test('the view follows the window size, and hides and shows', async ({
    app,
    main
  }) => {
    const { label, page } = await open(app, main)
    const [width] = await viewport(page)

    await page.evaluate(() =>
      window.__TAURI__.window
        .getCurrentWindow()
        .setSize(new window.__TAURI__.window.LogicalSize(700, 500))
    )
    await expect.poll(async () => (await viewport(page))[0]).toBeLessThan(width)
    const [resizedWidth, resizedHeight] = await viewport(page)
    const scale = await page.evaluate(() => window.devicePixelRatio)
    await expect
      .poll(async () => (await stats(main, page, label)).frameSize)
      .toEqual([
        Math.round(resizedWidth * scale),
        Math.round(resizedHeight * scale)
      ])

    await page.evaluate(() =>
      window.__TAURI__.webview.getCurrentWebview().hide()
    )
    await expect
      .poll(async () => (await sample(main, `${label}-page`)).visible)
      .toBe(false)
    await page.evaluate(() =>
      window.__TAURI__.webview.getCurrentWebview().show()
    )
    await expect
      .poll(async () => (await sample(main, `${label}-page`)).visible)
      .toBe(true)
  })

  test('an open <select> is painted as a popup widget', async ({
    app,
    main
  }) => {
    const { label, page } = await open(app, main)
    await page.locator('#choice').click()
    await expect
      .poll(async () => (await stats(main, page, label)).popupRect)
      .not.toBeNull()
    // CEF sizes the popup widget before it paints it.
    await expect
      .poll(async () => (await stats(main, page, label)).popupPaints)
      .toBeGreaterThan(0)
    await page.keyboard.press('Escape')
    await expect
      .poll(async () => (await stats(main, page, label)).popupRect)
      .toBeNull()
  })

  test('window.open becomes a Tauri window', async ({ app, main }) => {
    const { page } = await open(app, main)
    await page.locator('[data-popup="popup.html"]').click()

    const popup = await app.waitForPage(
      (url) => url.pathname === '/popup.html' && url.hash === ''
    )
    await expect(popup.locator('#kind')).toContainText(
      'A Tauri window the new-window handler built'
    )
    expect(await webviewLabels(main)).toContain('popup-offscreen-1')
  })

  test('a CEF-owned popup of an off-screen browser is a windowed browser', async ({
    app,
    main
  }) => {
    const { label } = await open(app, main)

    // Opened with no Playwright connection around, see `evaluateDetached`.
    await app.evaluateDetached(
      `document.querySelector('[data-popup="popup.html#cef-owned"]').click()`,
      `window.__TAURI__.core
        .invoke('native_snapshot', { options: { label: null } })
        .then((snapshot) =>
          snapshot.popups.some(
            (popup) => popup.browser.mainFrameUrl && !popup.browser.isLoading
          )
        )`,
      isOffscreenPage
    )

    const snapshot = await sample(app.main, `${label}-page`)
    expect(snapshot.popups).toHaveLength(1)
    const [popup] = snapshot.popups
    expect(popup.openedByThisBrowser).toBe(true)
    // The opener renders off-screen; its popup does not, so CEF gave it a
    // native window of its own.
    expect(snapshot.browser.windowless).toBe(true)
    expect(popup.browser.windowless).toBe(false)

    await app.closeTarget((url) => url.hash === '#cef-owned')
    await expect
      .poll(async () => (await sample(app.main, `${label}-page`)).popups)
      .toEqual([])
  })
})
