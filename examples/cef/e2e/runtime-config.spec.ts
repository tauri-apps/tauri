// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect, test } from './fixtures.js'

test.describe('Runtime configuration: tauri_runtime_cef::Cef', () => {
  test('the page runs on the Chromium the protocol client is attached to', async ({
    app,
    main
  }) => {
    // `Browser.getVersion`, asked by Playwright over the wire when it connected...
    const [version] = app.browser.version().match(/\d+\.\d+\.\d+\.\d+/) ?? []
    expect(version).toBeDefined()
    // ...and `RuntimeHandle::webview_version`, asked from Rust: the same build.
    await expect(main.locator('#chrome-version')).toHaveText(
      `Chromium ${version}`
    )
    await expect(main.locator('#cef-api-version')).toHaveText(/^CEF API \d+$/)
  })

  test('settings are readable back from the renderer, not just the Rust struct', async ({
    main
  }) => {
    // `accept_language_list`, as the page reports it and as the protocol
    // client reads it out of the renderer directly. CEF hands the list to
    // `navigator.languages` as written, quality values included.
    await expect(main.locator('#languages')).toHaveText(/^en-US, en.*pt-BR/)
    const languages = await main.evaluate(() => navigator.languages)
    expect(languages[0]).toBe('en-US')
    expect(languages.some((language) => language.startsWith('pt-BR'))).toBe(
      true
    )
    // `command_line_arg("autoplay-policy", "no-user-gesture-required")`:
    // nothing on the page has been clicked, and the context started running
    // anyway. Chromium has no API for the policy, so the behaviour is the readback.
    await expect(main.locator('#autoplay')).toHaveText('running')
    // `command_line_args([("js-flags", "--expose-gc")])`, set on the browser
    // process and forwarded to the renderer by Chromium itself.
    await expect(main.locator('#expose-gc')).toContainText('a function')
    expect(
      await main.evaluate(() => typeof (window as { gc?: unknown }).gc)
    ).toBe('function')
  })

  test('the configuration table shows what this run was launched with', async ({
    app,
    main
  }) => {
    const row = (name: string) =>
      main.locator('#configuration tr').filter({ hasText: name })
    // The port this very connection came in through.
    await expect(row('remote_debugging')).toContainText(
      `Port { port: ${app.port} }`
    )
    // The profile directory the launcher gave this run.
    await expect(row('root_cache_path')).toContainText(app.cacheDir)
    await expect(row('secret_storage')).toContainText('Mock')
    await expect(row('sandbox')).toContainText('Auto')
  })

  test.describe('with CEF_EXAMPLE_AUTOPLAY=off', () => {
    test.use({ appEnv: { CEF_EXAMPLE_AUTOPLAY: 'off' } })

    test('an AudioContext created with no user gesture starts suspended', async ({
      main
    }) => {
      // Chromium's own default, now that the switch is not set.
      await expect(main.locator('#autoplay')).toHaveText('suspended')
      await expect(
        main.locator('#configuration tr').filter({ hasText: 'autoplay-policy' })
      ).toContainText('unset')
    })
  })
})
