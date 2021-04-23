// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

export interface TauriBuildConfig {
  appName: string
  windowTitle: string
  /**
   * the path to the app's dist dir
   * this path must contain your index.html file
   */
  distDir: string
  /**
   * the app's dev server URL, or the path to the directory containing an index.html to open
   */
  devPath: string
  /**
   * a shell command to run before `tauri dev` kicks in
   */
  beforeDevCommand?: string
  /**
   * a shell command to run before `tauri build` kicks in
   */
  beforeBuildCommand?: string
  withGlobalTauri?: boolean
}
