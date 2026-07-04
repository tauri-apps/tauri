// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describeApi } from '../helpers/index.js'

describeApi('image', () => {
  it('Image.new round-trips raw rgba pixels', async () => {
    // 2x1 image: red pixel, green pixel.
    const rgba = [255, 0, 0, 255, 0, 255, 0, 255]
    const result = await tauri(
      async (api, pixels, width, height) => {
        const image = await api.image.Image.new(pixels, width, height)
        const size = await image.size()
        const bytes = await image.rgba()
        return {
          width: size.width,
          height: size.height,
          byteLength: bytes.length
        }
      },
      rgba,
      2,
      1
    )
    expect(result.width).toBe(2)
    expect(result.height).toBe(1)
    expect(result.byteLength).toBe(rgba.length)
  })

  it('the default window icon is a usable image', async () => {
    const size = await tauri(async (api) => {
      const icon = await api.app.defaultWindowIcon()
      return icon ? await icon.size() : null
    })
    expect(size).not.toBeNull()
    expect(size!.width).toBeGreaterThan(0)
    expect(size!.height).toBeGreaterThan(0)
  })
})
