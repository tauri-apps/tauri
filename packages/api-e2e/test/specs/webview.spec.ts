// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  eventually,
  describeApi,
  itDesktop,
  itOn,
  androidActivities,
  pressAndroidBack,
  EMBEDDED_ACTIVITY
} from '../helpers/index.js'

/**
 * Adds a webview to the main window (multiwebview), hosted on Android by
 * `EMBEDDED_ACTIVITY`, and resolves once it is created.
 */
async function addWebview(label: string): Promise<void> {
  const created = await tauri(
    (api, label, activityName) =>
      new Promise<boolean>((resolve, reject) => {
        const webview = new api.webview.Webview(
          api.window.getCurrentWindow(),
          label,
          {
            url: 'index.html',
            x: 0,
            y: 0,
            width: 100,
            height: 100,
            activityName
          }
        )
        webview.once('tauri://created', () => resolve(true)).catch(reject)
        webview
          .once('tauri://error', (event) =>
            reject(
              new Error(`webview creation failed: ${String(event.payload)}`)
            )
          )
          .catch(reject)
        setTimeout(() => reject(new Error('webview creation timed out')), 15000)
      }),
    label,
    EMBEDDED_ACTIVITY
  )
  expect(created).toBe(true)
}

async function webviewLabels(): Promise<string[]> {
  return tauri(async (api) =>
    (await api.webview.getAllWebviews()).map((w) => w.label)
  )
}

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

  // `webview_position`/`webview_size` are desktop-only commands: on mobile the
  // webview always fills the window.
  itDesktop('exposes a physical position and size', async () => {
    const rect = await tauri(async (api) => {
      const wv = api.webview.getCurrentWebview()
      const pos = await wv.position()
      const size = await wv.size()
      return { x: pos.x, y: pos.y, width: size.width, height: size.height }
    })
    expect(rect.width).toBeGreaterThan(0)
    expect(rect.height).toBeGreaterThan(0)
  })

  // A webview added to a window that already hosts one lives in an activity of
  // its own, started in the window's task, so it belongs to the same window;
  // activity embedding rules can show the two side by side on large screens.
  itOn(
    'android',
    'adds a webview in an activity of the window task and closes it',
    async () => {
      const initial = await androidActivities()
      expect(initial).toHaveLength(1)
      const main = initial[0]

      await addWebview('main-e2e-embedded')
      expect(await webviewLabels()).toContain('main-e2e-embedded')

      const activities = await eventually(async () => {
        const activities = await androidActivities()
        if (activities.length !== 2) {
          throw new Error(
            `expected the window's and the webview's activities, got ${JSON.stringify(activities)}`
          )
        }
        return activities
      })
      expect(activities).toContainEqual({
        activity: `.${EMBEDDED_ACTIVITY}`,
        task: main.task
      })

      await tauri(async (api) => {
        const webview =
          await api.webview.Webview.getByLabel('main-e2e-embedded')
        await webview?.close()
        return null
      })

      expect(await webviewLabels()).not.toContain('main-e2e-embedded')
      // closing the webview finishes its activity, and leaves the window alone
      await eventually(async () => {
        const activities = await androidActivities()
        if (activities.length !== 1 || activities[0].task !== main.task) {
          throw new Error(
            `expected only the window's activity, got ${JSON.stringify(activities)}`
          )
        }
      })
      expect(await webviewLabels()).toContain('main')
    }
  )

  // On a phone the webview's activity covers the window's, and the back button
  // closes it: the webview is destroyed with it.
  itOn(
    'android',
    'destroys the webview when its activity is closed',
    async () => {
      await addWebview('main-e2e-back')
      await tauri(async (api) => {
        const webview = await api.webview.Webview.getByLabel('main-e2e-back')
        // @ts-expect-error test-only global
        window.__e2eWebviewDestroyed = false
        await webview?.once('tauri://destroyed', () => {
          // @ts-expect-error test-only global
          window.__e2eWebviewDestroyed = true
        })
        return null
      })
      await eventually(async () => {
        const activities = await androidActivities()
        if (!activities.some((a) => a.activity === `.${EMBEDDED_ACTIVITY}`)) {
          throw new Error('the webview activity is not running yet')
        }
      })

      await pressAndroidBack()

      await eventually(async () => {
        const labels = await webviewLabels()
        if (labels.includes('main-e2e-back')) {
          throw new Error('webview is still present after its activity closed')
        }
      })
      expect(
        // @ts-expect-error test-only global
        await tauri(() => window.__e2eWebviewDestroyed as boolean)
      ).toBe(true)
      await eventually(async () => {
        const activities = await androidActivities()
        if (activities.some((a) => a.activity === `.${EMBEDDED_ACTIVITY}`)) {
          throw new Error('the webview activity is still running')
        }
      })

      // the label is free again
      await addWebview('main-e2e-back')
      await tauri(async (api) => {
        await (await api.webview.Webview.getByLabel('main-e2e-back'))?.close()
        return null
      })
    }
  )

  itDesktop('setZoom adjusts and restores the zoom level', async () => {
    // Requires `core:webview:allow-set-webview-zoom` in the run-app capability.
    await tauri(async (api) => {
      const wv = api.webview.getCurrentWebview()
      await wv.setZoom(1.5)
      await wv.setZoom(1)
      return null
    })
  })
})
