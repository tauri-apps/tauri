// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, eventually, describeApi } from '../helpers/index.js'

describeApi('webviewWindow', () => {
  it('getCurrentWebviewWindow reports the main label', async () => {
    expect(
      await tauri((api) => api.webviewWindow.getCurrentWebviewWindow().label)
    ).toBe('main')
  })

  it('creates, finds and closes a new webview window', async () => {
    // The label must match `main-*` to inherit the run-app capability.
    const created = await tauri(
      (api) =>
        new Promise<boolean>((resolve, reject) => {
          const w = new api.webviewWindow.WebviewWindow('main-e2e', {
            width: 400,
            height: 300
          })
          w.once('tauri://created', () => resolve(true)).catch(reject)
          w.once('tauri://error', (event) =>
            reject(
              new Error(`window creation failed: ${String(event.payload)}`)
            )
          ).catch(reject)
          setTimeout(() => reject(new Error('window creation timed out')), 8000)
        })
    )
    expect(created).toBe(true)

    const found = await tauri(async (api) => {
      const byLabel =
        await api.webviewWindow.WebviewWindow.getByLabel('main-e2e')
      const all = (await api.webviewWindow.getAllWebviewWindows()).map(
        (w) => w.label
      )
      return { exists: byLabel !== null, all }
    })
    expect(found.exists).toBe(true)
    expect(found.all).toContain('main-e2e')

    await tauri(async (api) => {
      const w = await api.webviewWindow.WebviewWindow.getByLabel('main-e2e')
      await w?.close()
      return null
    })

    await eventually(async () => {
      const labels = await tauri(async (api) =>
        (await api.webviewWindow.getAllWebviewWindows()).map((w) => w.label)
      )
      if (labels.includes('main-e2e')) {
        throw new Error('window is still present after close')
      }
    })
  })
})
