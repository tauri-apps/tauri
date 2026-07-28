// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

describeApi('webview', () => {
  it('getCurrentWebview reports the main label', async () => {
    expect(await tauri((api) => api.webview.getCurrentWebview().label)).toBe(
      'main'
    )
  })

  it('getAllWebviews includes the main webview', async () => {
    const labels = await tauri(async (api) =>
      (await api.webview.getAllWebviews()).map((w) => w.label)
    )
    expect(labels).toContain('main')
  })

  it('exposes a physical position and size', async () => {
    const rect = await tauri(async (api) => {
      const wv = api.webview.getCurrentWebview()
      const pos = await wv.position()
      const size = await wv.size()
      return { x: pos.x, y: pos.y, width: size.width, height: size.height }
    })
    expect(rect.width).toBeGreaterThan(0)
    expect(rect.height).toBeGreaterThan(0)
  })

  it('setZoom adjusts and restores the zoom level', async () => {
    // Requires `core:webview:allow-set-webview-zoom` in the run-app capability.
    await tauri(async (api) => {
      const wv = api.webview.getCurrentWebview()
      await wv.setZoom(1.5)
      await wv.setZoom(1)
      return null
    })
  })
})
