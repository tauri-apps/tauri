#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { readdirSync, readFileSync, writeFileSync } = require('node:fs')
const { join } = require('node:path')

const SOURCE_NAME = '@tauri-apps/cli'
const TARGET_NAME = '@tauri-apps/cli-cef'

const cliDir = process.cwd()
const npmDir = join(cliDir, 'npm')
const cefCliVersionPath = join(cliDir, '.cef-cli-version')
const tauriCliCargoTomlPath = join(cliDir, '../../crates/tauri-cli/Cargo.toml')

const cefCliVersion = readFileSync(cefCliVersionPath, 'utf8').trim()

if (!cefCliVersion) {
  throw new Error(`expected a version in ${cefCliVersionPath}`)
}

function rewritePackageName(packageJsonPath, setVersion = false) {
  const pkg = JSON.parse(readFileSync(packageJsonPath, 'utf8'))
  if (setVersion) {
    pkg.version = cefCliVersion
  }
  if (typeof pkg.name === 'string' && pkg.name.startsWith(SOURCE_NAME)) {
    pkg.name = pkg.name.replace(SOURCE_NAME, TARGET_NAME)
  }
  writeFileSync(packageJsonPath, `${JSON.stringify(pkg, null, 2)}\n`)
  console.log(`updated package metadata in ${packageJsonPath}`)
}

rewritePackageName(join(cliDir, 'package.json'), true)

for (const entry of readdirSync(npmDir, { withFileTypes: true })) {
  if (!entry.isDirectory()) {
    continue
  }
  rewritePackageName(join(npmDir, entry.name, 'package.json'))
}

const indexJsPath = join(cliDir, 'index.js')
const indexContents = readFileSync(indexJsPath, 'utf8')
const rewrittenIndexContents = indexContents.replace(
  /@tauri-apps\/cli(?=[-/'"`])/g,
  TARGET_NAME
)

if (rewrittenIndexContents !== indexContents) {
  writeFileSync(indexJsPath, rewrittenIndexContents)
  console.log(`rewrote native binding imports in ${indexJsPath}`)
}

const tauriCliCargoToml = readFileSync(tauriCliCargoTomlPath, 'utf8')
const tauriCliCargoTomlWithVersion = tauriCliCargoToml.replace(
  /^version = ".*"$/m,
  `version = "${cefCliVersion}"`
)

if (tauriCliCargoTomlWithVersion !== tauriCliCargoToml) {
  writeFileSync(tauriCliCargoTomlPath, tauriCliCargoTomlWithVersion)
  console.log(`updated tauri-cli version in ${tauriCliCargoTomlPath}`)
}
