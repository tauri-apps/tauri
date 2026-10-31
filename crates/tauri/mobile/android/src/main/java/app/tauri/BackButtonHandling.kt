// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri

interface BackButtonHandling {
  /**
   * Set to false to register no back callback, so the app can play Android's predictive back-to-home animation.
   */
  val handleBackButton: Boolean
}
