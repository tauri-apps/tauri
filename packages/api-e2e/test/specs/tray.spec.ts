// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

// Tray commands are allowed by default. A tray host has to be available anyway,
// since the app registers its own tray icon at startup (see the README for
// headless Linux sessions); `E2E_SKIP=tray` still skips this spec if needed.
describeApi('tray', () => {
  it('creates a tray icon, looks it up and removes it', async () => {
    const created = await tauri(async (api) => {
      const icon = await api.app.defaultWindowIcon()
      const tray = await api.tray.TrayIcon.new({
        id: 'e2e-tray',
        icon: icon ?? undefined,
        tooltip: 'e2e tray'
      })
      return tray.id
    })
    expect(created).toBe('e2e-tray')

    const foundById = await tauri(async (api) => {
      const tray = await api.tray.TrayIcon.getById('e2e-tray')
      return tray !== null
    })
    expect(foundById).toBe(true)

    await tauri(async (api) => {
      const tray = await api.tray.TrayIcon.getById('e2e-tray')
      await tray?.setVisible(true)
      return null
    })

    const removed = await tauri(async (api) => {
      await api.tray.TrayIcon.removeById('e2e-tray')
      const tray = await api.tray.TrayIcon.getById('e2e-tray')
      return tray === null
    })
    expect(removed).toBe(true)
  })
})
