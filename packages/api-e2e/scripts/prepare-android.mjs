// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Adds what the Android specs need to the generated Android project of
// examples/api (`src-tauri/gen/android`, which `tauri android init` creates
// from the CLI template): an activity to host the webviews the multiwebview
// specs add to a window. Each webview added to a window that already hosts one
// lives in an activity of its own, started in the window's task.
//
// Idempotent; run it after `tauri android init` and before building the APK.

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const projectDir = path.resolve(
  dirname,
  '..',
  '..',
  '..',
  'examples',
  'api',
  'src-tauri',
  'gen',
  'android'
)
const mainDir = path.join(projectDir, 'app', 'src', 'main')

/** `identifier` in examples/api's tauri.conf.json, the Kotlin package. */
const appPackage = 'com.tauri.api'

/** Must match `EMBEDDED_ACTIVITY` in test/helpers. */
const activity = 'EmbeddedWebviewActivity'

const manifestPath = path.join(mainDir, 'AndroidManifest.xml')
if (!fs.existsSync(manifestPath)) {
  throw new Error(
    `${manifestPath} is missing — run \`tauri android init\` in examples/api first.`
  )
}

const kotlinPath = path.join(
  mainDir,
  'java',
  ...appPackage.split('.'),
  `${activity}.kt`
)
fs.writeFileSync(
  kotlinPath,
  `package ${appPackage}

// Hosts the webviews the e2e suite adds to a window (packages/api-e2e/scripts/prepare-android.mjs).
class ${activity} : TauriActivity()
`
)

const manifest = fs.readFileSync(manifestPath, 'utf8')
if (!manifest.includes(`android:name=".${activity}"`)) {
  fs.writeFileSync(
    manifestPath,
    manifest.replace(
      '</application>',
      `    <!-- api-e2e: hosts the webviews added to a window -->
        <activity
            android:name=".${activity}"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|locale|smallestScreenSize|screenLayout|uiMode"
            android:exported="false" />
    </application>`
    )
  )
}

console.log(`added ${activity} to ${projectDir}`)
