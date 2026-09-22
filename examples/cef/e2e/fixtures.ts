// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
  test as base,
  expect,
  type Locator,
  type Page,
  type TestInfo
} from '@playwright/test'
import { CefApp } from './app.js'

interface Fixtures {
  /** A fresh instance of the example, launched for this test alone. */
  app: CefApp
  /** The main window's page. */
  main: Page
}

interface Options {
  /**
   * Environment for the app under test, on top of what `CefApp.launch` sets:
   * the `CEF_EXAMPLE_*` knobs the README lists. Set it for a group of tests with
   * `test.use({ appEnv: { CEF_EXAMPLE_AUTOPLAY: 'off' } })`.
   */
  appEnv: Record<string, string>
}

export const test = base.extend<Fixtures & Options>({
  appEnv: [{}, { option: true }],

  app: async ({ appEnv }, use, testInfo) => {
    const app = await CefApp.launch({ env: appEnv })
    await use(app)
    if (testInfo.status !== testInfo.expectedStatus) {
      await attachEvidence(app, testInfo)
    }
    await app.close()
  },

  main: async ({ app }, use) => {
    await use(app.main)
  }
})

export { expect }

/**
 * Playwright's own screenshot-on-failure only knows the browsers it launched, so
 * a failed test gets a screenshot of every page over the protocol, and what the
 * app wrote to its standard streams.
 */
async function attachEvidence(app: CefApp, testInfo: TestInfo): Promise<void> {
  await testInfo.attach('app-output.txt', {
    body: app.output.join(''),
    contentType: 'text/plain'
  })
  // One per URL: a browser several connections attached to is listed once per
  // connection. A wedged browser answers nothing, hence the short timeouts.
  const pages = new Map(app.pages().map((page) => [page.url(), page]))
  for (const [index, [url, page]] of [...pages].entries()) {
    const slug = url.replace(/[^a-z0-9]+/gi, '-')
    try {
      await testInfo.attach(`page-${index}-${slug}.png`, {
        body: await page.screenshot({ timeout: 5_000 }),
        contentType: 'image/png'
      })
    } catch {
      // a page that is going away, or not answering, cannot be captured
    }
  }
}

/**
 * The entries of one of the main page's log panels, narrowed to the ones whose
 * tag (the label every entry starts with: the console level, the frame event
 * name, the DevTools message id, ...) and text match.
 */
export function entries(
  main: Page,
  panel: string,
  filter: { tag?: string; text?: string | RegExp } = {}
): Locator {
  let entries = main.locator(`#${panel} .entry`)
  if (filter.tag !== undefined) {
    entries = entries.filter({
      has: main.locator('.tag', {
        hasText: new RegExp(`^${escapeRegExp(filter.tag)}$`)
      })
    })
  }
  if (filter.text !== undefined) {
    entries = entries.filter({ hasText: filter.text })
  }
  return entries
}

/** The labels of every webview the app has, as `getAllWebviews` lists them. */
export function webviewLabels(main: Page): Promise<string[]> {
  return main.evaluate(async () =>
    (await window.__TAURI__.webview.getAllWebviews())
      .map((webview) => webview.label)
      .sort()
  )
}

/** The JSON the `native_snapshot` command answers with, as `SnapshotInfo` serializes. */
export interface Snapshot {
  browserId: number
  windowLabel: string | null
  nativeWindowObserved: boolean
  sameNativeWindowAsPreviousSample: boolean | null
  documentAdmitted: boolean
  selectedByItsDocument: boolean | null
  parentMatches: boolean | null
  visible: boolean | null
  bounds: unknown
  dialogs: DialogInfo
  browser: BrowserInfo
  popups: PopupInfo[]
}

export interface DialogInfo {
  known: boolean
  kind: string | null
  hasBrowserHandler: boolean | null
}

export interface BrowserInfo {
  mainFrameUrl: string | null
  frameCount: number
  isLoading: boolean
  zoomLevel: number
  devToolsOpen: boolean
}

export interface PopupInfo {
  browserId: number
  openedByThisBrowser: boolean
  visible: boolean | null
  dialogs: DialogInfo
  selectedByItsDocument: boolean | null
  browser: BrowserInfo
}

/**
 * Samples a webview's native state through the "Native state" panel: the
 * webview labeled `label`, or the main one — the caller of the command — when
 * no label is given.
 */
export async function sample(main: Page, label = ''): Promise<Snapshot> {
  const target = main.locator('#snapshot-target')
  if (label) {
    // The list of labels is refreshed when the select is focused, so a window
    // opened a moment ago is on offer.
    await target.focus()
    await target
      .locator(`option[value="${label}"]`)
      .waitFor({ state: 'attached' })
    await target.selectOption(label)
  } else {
    await target.selectOption({ index: 0 })
  }
  const output = main.locator('#snapshot-output')
  // Cleared first: a sample identical to the previous one would otherwise be
  // indistinguishable from no answer.
  await output.evaluate((element) => {
    element.textContent = ''
  })
  await main.locator('#snapshot').click()
  await expect(output).not.toBeEmpty()
  const text = (await output.textContent()) ?? ''
  try {
    return JSON.parse(text) as Snapshot
  } catch {
    throw new Error(`native_snapshot failed: ${text}`)
  }
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
