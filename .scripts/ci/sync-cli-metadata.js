#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/*
This script is solely intended to be run as part of the `covector version` step to
keep the `../../crates/tauri-cli/metadata-v2.json` up to date with other version bumps. Long term
we should look to find a more "rusty way" to import / "pin" a version value in our tauri-cli
rust binaries.
*/

const { readFileSync, writeFileSync } = require('fs')
const { resolve } = require('path')

const packageNickname = process.argv[2]
const filePath = resolve(__dirname, '../../crates/tauri-cli/metadata-v2.json')
const bump = process.argv[3]
let index = null

switch (bump) {
  case 'major':
  case 'premajor':
    index = 0
    break
  case 'minor':
    index = 1
    break
  case 'patch':
    index = 2
    break
  case 'prerelease':
  case 'prepatch':
    index = 3
    break
  default:
    throw new Error('unexpected bump ' + bump)
}

const inc = (version) => {
  const v = version.split('.')
  for (let i = 0; i < v.length; i++) {
    if (i === index) {
      v[i] = String(Number(v[i]) + 1)
    } else if (i > index) {
      v[i] = 0
    }
  }
  if (bump === 'premajor') {
    const pre = JSON.parse(
      readFileSync(resolve(__dirname, '../../.changes/pre.json'), 'utf-8')
    )
    return `${v.join('.')}-${pre.tag}.0`
  }
  return v.join('.')
}

// read file into js object
const metadata = JSON.parse(readFileSync(filePath, 'utf-8'))

// Extract CEF version from tauri-runtime-cef/Cargo.toml if it exists
// This runs whenever tauri-runtime-cef version is bumped
if (packageNickname === 'tauri-runtime-cef') {
  try {
    const cargoLockPath = resolve(__dirname, '../../Cargo.lock')
    const cargoLock = readFileSync(cargoLockPath, 'utf-8')
    // Find the [package] section for "cef"
    // e.g.:
    // [[package]]
    // name = "cef"
    // version = "141.6.0+141.0.11"
    const pkgRegex =
      /\[\[package\]\][^\[]+?name\s*=\s*"cef"[^\[]+?version\s*=\s*"([^"]+)"/m
    const match = cargoLock.match(pkgRegex)
    if (match && match[1]) {
      const cefVersion = match[1]
      metadata.cef = cefVersion
      console.log(`Extracted CEF version ${cefVersion} from Cargo.lock`)
    } else {
      throw new Error('Could not find cef package and version in Cargo.lock')
    }
  } catch (error) {
    throw new Error(
      `Failed to extract CEF version from Cargo.lock: ${error.message}`
    )
  }
}

// set field version
let version
if (packageNickname === '@tauri-apps/cli') {
  version = inc(metadata['cli.js'].version)
  metadata['cli.js'].version = version
} else if (
  packageNickname
  !== 'tauri-runtime-cef' /* for cef we only update the cef version */
) {
  version = inc(metadata[packageNickname])
  metadata[packageNickname] = version
}

writeFileSync(filePath, JSON.stringify(metadata, null, 2) + '\n')
console.log(`wrote ${version} for ${packageNickname} into metadata-v2.json`)
console.dir(metadata)
