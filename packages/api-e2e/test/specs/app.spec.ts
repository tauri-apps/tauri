// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, eventually, describeApi } from '../helpers/index.js'

describeApi('app', () => {
  it('getName returns the configured product name', async () => {
    expect(await tauri((api) => api.app.getName())).toBe('Tauri API')
  })

  it('getVersion returns the configured version', async () => {
    expect(await tauri((api) => api.app.getVersion())).toBe('1.0.0')
  })

  it('getTauriVersion returns a semver string', async () => {
    expect(await tauri((api) => api.app.getTauriVersion())).toMatch(
      /^\d+\.\d+\.\d+/
    )
  })

  it('getIdentifier returns the bundle identifier', async () => {
    expect(await tauri((api) => api.app.getIdentifier())).toBe('com.tauri.api')
  })

  it('defaultWindowIcon returns an image with a positive size', async () => {
    const size = await tauri(async (api) => {
      const icon = await api.app.defaultWindowIcon()
      return icon ? await icon.size() : null
    })
    expect(size).not.toBeNull()
    expect(size!.width).toBeGreaterThan(0)
    expect(size!.height).toBeGreaterThan(0)
  })

  it('setTheme applies a theme and can be reset to the system default', async () => {
    await tauri((api) => api.app.setTheme('dark'))
    // On Linux the app-level theme is not observable through `window.theme()`:
    // tao's event-loop `set_theme` only flips the GTK `prefer-dark` setting and
    // never updates the window's stored theme, so `theme()` keeps reporting the
    // portal/system value (and blocks on a 5s dbus timeout when no portal runs,
    // as in headless CI). Elsewhere the change is reflected on the window.
    if (process.platform !== 'linux') {
      await eventually(async () => {
        const theme = await tauri((api) =>
          api.window.getCurrentWindow().theme()
        )
        if (theme !== 'dark') {
          throw new Error(`expected theme "dark", got "${theme}"`)
        }
      })
    }
    // restore
    await tauri((api) => api.app.setTheme(null))
  })
})
