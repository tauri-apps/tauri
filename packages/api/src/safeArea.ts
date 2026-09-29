// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read the current document's safe area, as reported by its webview engine.
 *
 * For layouts that only need padding, prefer CSS `env(safe-area-inset-*)`:
 * the engine updates these values automatically when the safe area changes.
 * @module
 */

/** Distances from the viewport edges in CSS pixels, without device-pixel scaling. */
export interface SafeAreaInsets {
  top: number
  right: number
  bottom: number
  left: number
}

/**
 * Read a snapshot of the current document's CSS safe-area insets.
 *
 * This reads `env(safe-area-inset-*)`, not native window coordinates. A value of
 * zero can mean that the viewport is already inset or that the webview engine
 * does not expose safe areas. It does not establish that no system bar exists.
 *
 * For an immersive OpenHarmony or iOS page, include `viewport-fit=cover` in its
 * viewport meta tag. Call again after viewport changes if using the returned
 * values in JavaScript. CSS padding using `env(...)` updates automatically.
 *
 * Must be called in the document whose insets are needed. No IPC is performed.
 *
 * @example
 * ```typescript
 * import { getSafeAreaInsets } from '@tauri-apps/api/safeArea';
 * const { top, bottom } = getSafeAreaInsets();
 * ```
 */
export function getSafeAreaInsets(): SafeAreaInsets {
  const probe = document.createElement('div')
  probe.setAttribute('aria-hidden', 'true')
  probe.style.cssText = `
    all: initial !important;
    position: fixed !important;
    visibility: hidden !important;
    pointer-events: none !important;
    width: 0 !important;
    height: 0 !important;
    padding-top: env(safe-area-inset-top, 0px) !important;
    padding-right: env(safe-area-inset-right, 0px) !important;
    padding-bottom: env(safe-area-inset-bottom, 0px) !important;
    padding-left: env(safe-area-inset-left, 0px) !important;
  `
  document.documentElement.appendChild(probe)
  try {
    const style = getComputedStyle(probe)
    const pixels = (value: string): number => {
      const result = Number.parseFloat(value)
      return Number.isFinite(result) && result > 0 ? result : 0
    }
    return {
      top: pixels(style.paddingTop),
      right: pixels(style.paddingRight),
      bottom: pixels(style.paddingBottom),
      left: pixels(style.paddingLeft)
    }
  } finally {
    probe.remove()
  }
}
