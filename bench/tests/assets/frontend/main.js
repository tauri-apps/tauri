// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The placeholders are filled in by the build script, which generates the rest of the frontend.
const IMAGES = __IMAGES__
const CHUNKS = __CHUNKS__
const DATA_SIZE = 1024 * 1024

const { invoke } = window.__TAURI__.core

async function main() {
  const metrics = {}

  // every script before this one was loaded and executed
  for (let chunk = 0; chunk < CHUNKS; chunk++) {
    if (!window.__bench[`chunk${chunk}`]) {
      throw new Error(`chunk ${chunk} did not load`)
    }
  }

  // the images and fonts a page requests while it renders
  await Promise.all(
    Array.from({ length: IMAGES }, async (_, image) => {
      const response = await fetch(`assets/image-${image}.bin`)
      if (!response.ok) {
        throw new Error(`image ${image} failed with ${response.status}`)
      }
      await response.arrayBuffer()
    })
  )

  // the asset protocol by itself: looped, so the time does not depend on the webview boot
  const iterations = 20
  const start = performance.now()
  for (let i = 0; i < iterations; i++) {
    const response = await fetch(`assets/data.bin?${i}`, { cache: 'no-store' })
    const data = await response.arrayBuffer()
    if (data.byteLength !== DATA_SIZE) {
      throw new Error(`unexpected asset size ${data.byteLength}`)
    }
  }
  metrics.asset_fetch_1mb_ms = (performance.now() - start) / iterations

  await invoke('assets_loaded', { metrics })
}

main().catch((error) => invoke('fail', { message: String(error) }))
