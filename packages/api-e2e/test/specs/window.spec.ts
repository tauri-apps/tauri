// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, eventually, describeApi, itWm } from '../helpers/index.js'

describeApi('window', () => {
  it('getCurrentWindow reports the main label', async () => {
    expect(await tauri((api) => api.window.getCurrentWindow().label)).toBe(
      'main'
    )
  })

  it('getAllWindows includes the main window', async () => {
    const labels = await tauri(async (api) =>
      (await api.window.getAllWindows()).map((w) => w.label)
    )
    expect(labels).toContain('main')
  })

  it('title can be read and set', async () => {
    const original = await tauri((api) => api.window.getCurrentWindow().title())
    await tauri((api) => api.window.getCurrentWindow().setTitle('e2e title'))
    expect(await tauri((api) => api.window.getCurrentWindow().title())).toBe(
      'e2e title'
    )
    // restore
    await tauri(
      (api, title) => api.window.getCurrentWindow().setTitle(title),
      original
    )
  })

  it('reports a positive scale factor and sizes', async () => {
    const metrics = await tauri(async (api) => {
      const w = api.window.getCurrentWindow()
      const scale = await w.scaleFactor()
      const inner = await w.innerSize()
      const outer = await w.outerSize()
      return {
        scale,
        inner: { width: inner.width, height: inner.height },
        outer: { width: outer.width, height: outer.height }
      }
    })
    expect(metrics.scale).toBeGreaterThan(0)
    expect(metrics.inner.width).toBeGreaterThan(0)
    expect(metrics.inner.height).toBeGreaterThan(0)
    expect(metrics.outer.width).toBeGreaterThanOrEqual(metrics.inner.width)
  })

  it('reports a theme', async () => {
    const theme = await tauri((api) => api.window.getCurrentWindow().theme())
    expect(['light', 'dark']).toContain(theme)
  })

  it('monitor queries resolve to sane shapes', async () => {
    const monitors = await tauri(async (api) => {
      const current = await api.window.currentMonitor()
      const primary = await api.window.primaryMonitor()
      const available = await api.window.availableMonitors()
      return {
        currentHasSize: current
          ? current.size.width > 0 && current.size.height > 0
          : null,
        primaryHasSize: primary
          ? primary.size.width > 0 && primary.size.height > 0
          : null,
        availableCount: available.length
      }
    })
    // In a headed environment there is at least one monitor; in bare headless
    // there may be none, so only assert shape when present.
    if (monitors.currentHasSize !== null) {
      expect(monitors.currentHasSize).toBe(true)
    }
    expect(monitors.availableCount).toBeGreaterThanOrEqual(0)
  })

  it('setSize resizes the window', async () => {
    const scale = await tauri((api) =>
      api.window.getCurrentWindow().scaleFactor()
    )
    const original = await tauri(async (api) => {
      const size = await api.window.getCurrentWindow().innerSize()
      return { width: size.width, height: size.height }
    })

    await tauri(
      (api, width, height) =>
        api.window
          .getCurrentWindow()
          .setSize(new api.dpi.LogicalSize(width, height)),
      600,
      400
    )

    await eventually(async () => {
      const inner = await tauri(async (api) => {
        const size = await api.window.getCurrentWindow().innerSize()
        return { width: size.width, height: size.height }
      })
      const tolerance = Math.ceil(scale) * 4
      if (Math.abs(inner.width - 600 * scale) > tolerance) {
        throw new Error(`width ${inner.width} not near ${600 * scale}`)
      }
      if (Math.abs(inner.height - 400 * scale) > tolerance) {
        throw new Error(`height ${inner.height} not near ${400 * scale}`)
      }
    })

    // restore the original physical size
    await tauri(
      (api, width, height) =>
        api.window
          .getCurrentWindow()
          .setSize(new api.dpi.PhysicalSize(width, height)),
      original.width,
      original.height
    )
  })

  it('setResizable toggles resizability', async () => {
    await tauri((api) => api.window.getCurrentWindow().setResizable(false))
    expect(
      await tauri((api) => api.window.getCurrentWindow().isResizable())
    ).toBe(false)
    await tauri((api) => api.window.getCurrentWindow().setResizable(true))
    expect(
      await tauri((api) => api.window.getCurrentWindow().isResizable())
    ).toBe(true)
  })

  it('assorted setters resolve without throwing', async () => {
    await tauri(async (api) => {
      const w = api.window.getCurrentWindow()
      await w.setAlwaysOnTop(true)
      await w.setAlwaysOnTop(false)
      await w.setShadow(true)
      await w.setContentProtected(false)
      await w.setCursorVisible(true)
      await w.requestUserAttention(null)
      return null
    })
  })

  itWm('setPosition moves the window', async () => {
    const original = await tauri(async (api) => {
      const pos = await api.window.getCurrentWindow().outerPosition()
      return { x: pos.x, y: pos.y }
    })

    await tauri(
      (api, x, y) =>
        api.window
          .getCurrentWindow()
          .setPosition(new api.dpi.PhysicalPosition(x, y)),
      original.x + 40,
      original.y + 40
    )

    await eventually(async () => {
      const pos = await tauri(async (api) => {
        const p = await api.window.getCurrentWindow().outerPosition()
        return { x: p.x, y: p.y }
      })
      // Window managers can nudge the final position; allow a generous tolerance.
      if (
        Math.abs(pos.x - (original.x + 40)) > 30
        || Math.abs(pos.y - (original.y + 40)) > 30
      ) {
        throw new Error(
          `position ${pos.x},${pos.y} not near ${original.x + 40},${original.y + 40}`
        )
      }
    })

    await tauri(
      (api, x, y) =>
        api.window
          .getCurrentWindow()
          .setPosition(new api.dpi.PhysicalPosition(x, y)),
      original.x,
      original.y
    )
  })

  itWm('minimize and unminimize toggle the minimized state', async () => {
    await tauri((api) => api.window.getCurrentWindow().minimize())
    await eventually(async () => {
      if (
        !(await tauri((api) => api.window.getCurrentWindow().isMinimized()))
      ) {
        throw new Error('window is not minimized')
      }
    })
    await tauri((api) => api.window.getCurrentWindow().unminimize())
    await eventually(async () => {
      if (await tauri((api) => api.window.getCurrentWindow().isMinimized())) {
        throw new Error('window is still minimized')
      }
    })
  })

  itWm('maximize and unmaximize toggle the maximized state', async () => {
    await tauri((api) => api.window.getCurrentWindow().maximize())
    await eventually(async () => {
      if (
        !(await tauri((api) => api.window.getCurrentWindow().isMaximized()))
      ) {
        throw new Error('window is not maximized')
      }
    })
    await tauri((api) => api.window.getCurrentWindow().unmaximize())
    await eventually(async () => {
      if (await tauri((api) => api.window.getCurrentWindow().isMaximized())) {
        throw new Error('window is still maximized')
      }
    })
  })

  itWm('setFullscreen toggles fullscreen', async () => {
    await tauri((api) => api.window.getCurrentWindow().setFullscreen(true))
    await eventually(async () => {
      if (
        !(await tauri((api) => api.window.getCurrentWindow().isFullscreen()))
      ) {
        throw new Error('window is not fullscreen')
      }
    })
    await tauri((api) => api.window.getCurrentWindow().setFullscreen(false))
    await eventually(async () => {
      if (await tauri((api) => api.window.getCurrentWindow().isFullscreen())) {
        throw new Error('window is still fullscreen')
      }
    })
  })

  itWm('hide and show toggle visibility', async () => {
    await tauri((api) => api.window.getCurrentWindow().hide())
    await eventually(async () => {
      if (await tauri((api) => api.window.getCurrentWindow().isVisible())) {
        throw new Error('window is still visible')
      }
    })
    await tauri((api) => api.window.getCurrentWindow().show())
    await eventually(async () => {
      if (!(await tauri((api) => api.window.getCurrentWindow().isVisible()))) {
        throw new Error('window is not visible')
      }
    })
  })
})
