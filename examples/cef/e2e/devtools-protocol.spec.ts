// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { entries, expect, test } from './fixtures.js'
import type { Page } from '@playwright/test'

/** Fills the panel from one of its presets and sends it, returning the request id. */
async function send(main: Page, method: string): Promise<string> {
  await main.locator('#cdp-presets button', { hasText: method }).click()
  await expect(main.locator('#cdp-method')).toHaveValue(method)
  await main.locator('#cdp-send').click()
  const sent = entries(main, 'cdp-log', { tag: 'sent', text: method })
  await expect(sent).toHaveText(
    new RegExp(`#\\d+ ${method.replace('.', '\\.')}`)
  )
  return ((await sent.textContent()) ?? '').match(/#(\d+)/)![1]
}

test.describe('DevTools protocol: send_dev_tools_message / on_dev_tools_protocol', () => {
  test('a request sent from Rust is answered under the id the runtime allocated', async ({
    main
  }) => {
    const id = await send(main, 'Browser.getVersion')
    // The result comes back on the observer, correlated by that id.
    const result = entries(main, 'cdp-log', { tag: `#${id}` })
    await expect(result).toHaveCount(1)
    await expect(result).toContainText('"product":"Chrome/')

    // The same question, asked from outside the process over the same
    // protocol: same answer.
    const session = await main.context().newCDPSession(main)
    const { product } = await session.send('Browser.getVersion')
    await expect(result).toContainText(product)
    await session.detach()
  })

  test('protocol events reach the observer once their domain is enabled', async ({
    main
  }) => {
    const id = await send(main, 'Network.enable')
    await expect(entries(main, 'cdp-log', { tag: `#${id}` })).toHaveText(/\{\}/)

    // Anything the page fetches from now on is a `Network.requestWillBeSent`.
    await main.evaluate(() =>
      fetch('frame.html?from-the-page').then((response) => response.text())
    )
    await expect(
      entries(main, 'cdp-log', {
        tag: 'event',
        text: 'Network.requestWillBeSent'
      }).filter({ hasText: 'frame.html?from-the-page' })
    ).not.toHaveCount(0)
  })
})
