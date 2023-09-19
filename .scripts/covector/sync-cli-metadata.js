#!/usr/bin/env node

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/*
This script is solely intended to be run as part of the `covector version` step to
keep the `../tooling/cli/metadata.json` up to date with other version bumps. Long term
we should look to find a more "rusty way" to import / "pin" a version value in our tauri-cli
rust binaries.
*/

const { readFileSync, writeFileSync } = require('fs')
const { resolve } = require('path')

const packageNickname = process.argv[2]
const filePath =
  packageNickname === '@tauri-apps/cli'
    ? `../../../tooling/cli/metadata-v2.json`
    : `../../tooling/cli/metadata-v2.json`
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
      readFileSync(resolve(filePath, '../../../.changes/pre.json'), 'utf-8')
    )
    return `${v.join('.')}-${pre.tag}.0`
  }
  return v.join('.')
}

// read file into js object
const metadata = JSON.parse(readFileSync(filePath, 'utf-8'))

// set field version
let version
if (packageNickname === '@tauri-apps/cli') {
  version = inc(metadata['cli.js'].version)
  metadata['cli.js'].version = version
} else {
  version = inc(metadata[packageNickname])
  metadata[packageNickname] = version
}

writeFileSync(filePath, JSON.stringify(metadata, null, 2) + '\n')
console.log(`wrote ${version} for ${packageNickname} into metadata-v2.json`)
console.dir(metadata)
