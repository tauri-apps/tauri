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
  androidActivities
} from '../helpers/index.js'

describeApi('webviewWindow', () => {
  it('getCurrentWebviewWindow reports the main label', async () => {
    expect(
      await tauri((api) => api.webviewWindow.getCurrentWebviewWindow().label)
    ).toBe('main')
  })

  // On iOS a new window is a new scene that takes over the foreground (and the
  // automation session's webview context); Android has its own test below.
  itDesktop('creates, finds and closes a new webview window', async () => {
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

  // Each window is an activity in a task of its own (a separate window in
  // Recents, split screen and desktop windowing), by default a new instance of
  // the launcher activity. The new task comes to the foreground, but the
  // session stays attached to the main window's page, which keeps running.
  itOn(
    'android',
    'opens a window in a task of its own and removes the task on close',
    async () => {
      const initial = await androidActivities()
      expect(initial).toHaveLength(1)
      const main = initial[0]
      expect(main.activity).toBe('.MainActivity')

      const created = await tauri(
        (api) =>
          new Promise<boolean>((resolve, reject) => {
            const w = new api.webviewWindow.WebviewWindow('main-e2e-task')
            w.once('tauri://created', () => resolve(true)).catch(reject)
            w.once('tauri://error', (event) =>
              reject(
                new Error(`window creation failed: ${String(event.payload)}`)
              )
            ).catch(reject)
            setTimeout(
              () => reject(new Error('window creation timed out')),
              15000
            )
          })
      )
      expect(created).toBe(true)

      const window = await tauri(async (api) => {
        const w =
          await api.webviewWindow.WebviewWindow.getByLabel('main-e2e-task')
        return {
          labels: (await api.webviewWindow.getAllWebviewWindows()).map(
            (w) => w.label
          ),
          activity: await w?.activityName()
        }
      })
      expect(window.labels).toContain('main-e2e-task')
      expect(window.activity).toMatch(/MainActivity/)

      const opened = await eventually(async () => {
        const activities = await androidActivities()
        if (activities.length !== 2) {
          throw new Error(
            `expected the main and the new window's activities, got ${JSON.stringify(activities)}`
          )
        }
        return activities
      })
      expect(opened.every((a) => a.activity === '.MainActivity')).toBe(true)
      expect(new Set(opened.map((a) => a.task)).size).toBe(2)
      expect(opened.map((a) => a.task)).toContain(main.task)

      await tauri(async (api) => {
        const w =
          await api.webviewWindow.WebviewWindow.getByLabel('main-e2e-task')
        await w?.close()
        return null
      })

      await eventually(async () => {
        const labels = await tauri(async (api) =>
          (await api.webviewWindow.getAllWebviewWindows()).map((w) => w.label)
        )
        if (labels.includes('main-e2e-task')) {
          throw new Error('window is still present after close')
        }
      })
      // the window's task is gone; the main window's is untouched
      await eventually(async () => {
        const activities = await androidActivities()
        if (activities.length !== 1 || activities[0].task !== main.task) {
          throw new Error(
            `expected only the main window's activity, got ${JSON.stringify(activities)}`
          )
        }
      })
    }
  )
})
