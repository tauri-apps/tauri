// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { appExecutable, builtApp, exampleDir } from './app.js'

/** Builds the app the tests launch, unless told to reuse an existing build. */
export default function globalSetup(): void {
  if (!process.env.CEF_E2E_SKIP_BUILD && !process.env.CEF_E2E_APP_PATH) {
    // The example's `tauri` script is `node ../../packages/cli/tauri.js`, so the
    // native CLI has to have been built (`pnpm build:cli` at the repo root). The
    // CLI detects the CEF runtime from the example's manifest: on macOS it ships
    // the framework and the helper apps in the `.app`, which is the only place
    // the app can run from; elsewhere `--no-bundle` leaves the bare executable
    // with the CEF distribution laid out next to it by the `cef` crate.
    const args = [
      'tauri',
      'build',
      '--debug',
      ...(process.platform === 'darwin'
        ? ['--bundles', 'app']
        : ['--no-bundle'])
    ]
    const build = spawnSync('pnpm', args, {
      cwd: exampleDir,
      stdio: 'inherit',
      shell: true
    })
    if (build.status !== 0) {
      throw new Error(
        `\`pnpm ${args.join(' ')}\` failed with status ${build.status}`
      )
    }
  }

  const executable = appExecutable()
  if (!fs.existsSync(executable)) {
    throw new Error(
      `app not found at ${executable} — build it (unset CEF_E2E_SKIP_BUILD) or point CEF_E2E_APP_PATH at an existing build.`
    )
  }

  // A stale CLI build, one predating the bundler's runtime detection, produces a
  // bundle without the framework, and the executable aborts loading it. Catch
  // that here rather than as an opaque connection timeout.
  if (
    process.platform === 'darwin'
    && !fs.existsSync(
      path.join(
        builtApp(),
        'Contents',
        'Frameworks',
        'Chromium Embedded Framework.framework'
      )
    )
  ) {
    throw new Error(
      `${builtApp()} does not bundle the CEF framework — it was built by a CLI that predates the CEF runtime: run \`pnpm build:cli\` and rebuild.`
    )
  }
}
