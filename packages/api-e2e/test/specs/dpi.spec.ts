// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

describeApi('dpi', () => {
  it('LogicalSize.toPhysical scales up by the factor', async () => {
    const physical = await tauri((api) => {
      const size = new api.dpi.LogicalSize(100, 50).toPhysical(2)
      return { width: size.width, height: size.height }
    })
    expect(physical).toEqual({ width: 200, height: 100 })
  })

  it('PhysicalSize.toLogical scales down by the factor', async () => {
    const logical = await tauri((api) => {
      const size = new api.dpi.PhysicalSize(200, 100).toLogical(2)
      return { width: size.width, height: size.height }
    })
    expect(logical).toEqual({ width: 100, height: 50 })
  })

  it('LogicalPosition.toPhysical and PhysicalPosition.toLogical round-trip', async () => {
    const result = await tauri((api) => {
      const physical = new api.dpi.LogicalPosition(10, 20).toPhysical(2)
      const logical = new api.dpi.PhysicalPosition(10, 20).toLogical(2)
      return {
        physical: { x: physical.x, y: physical.y },
        logical: { x: logical.x, y: logical.y }
      }
    })
    expect(result.physical).toEqual({ x: 20, y: 40 })
    expect(result.logical).toEqual({ x: 5, y: 10 })
  })

  it('Size and Position wrappers delegate conversions', async () => {
    const result = await tauri((api) => {
      const size = new api.dpi.Size(
        new api.dpi.LogicalSize(100, 50)
      ).toPhysical(2)
      const position = new api.dpi.Position(
        new api.dpi.PhysicalPosition(20, 40)
      ).toLogical(2)
      return {
        size: { width: size.width, height: size.height },
        position: { x: position.x, y: position.y }
      }
    })
    expect(result.size).toEqual({ width: 200, height: 100 })
    expect(result.position).toEqual({ x: 10, y: 20 })
  })

  it('window sizes convert consistently against the real scale factor', async () => {
    const consistent = await tauri(async (api) => {
      const w = api.window.getCurrentWindow()
      const scale = await w.scaleFactor()
      const inner = await w.innerSize()
      const logical = inner.toLogical(scale)
      return Math.abs(logical.width * scale - inner.width) < 1
    })
    expect(consistent).toBe(true)
  })
})
