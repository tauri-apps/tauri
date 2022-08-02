#!/usr/bin/env node
 // Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/*
This script is solely intended to be run as part of the `covector version` step to
keep the `../tooling/cli/metadata.json` up to date with other version bumps. Long term
we should look to find a more "rusty way" to import / "pin" a version value in our cli.rs
rust binaries.
*/

const {
  readFileSync,
  writeFileSync
} = require('fs')

const packageNickname = process.argv[2]
const filePath = packageNickname === 'cli.js' ? `../../../tooling/cli/metadata.json` : `../../tooling/cli/metadata.json`
const bump = process.argv[3]
let index = null

switch (bump) {
  case 'major':
    index = 0;
    break;
  case 'minor':
    index = 1;
    break;
  case 'patch':
    index = 2;
    break;
  default:
    throw new Error('unexpected bump ' + bump)
}

const inc = (version) => {
  const v = version.split('.')
  const n = v[index]
  v[index] = String(Number(n) + 1)
  return v.join('.')
}

// read file into js object
const metadata = JSON.parse(readFileSync(filePath, 'utf-8'))

// set field version
let version
if (packageNickname === 'cli.js') {
  version = inc(metadata[packageNickname].version)
  metadata[packageNickname].version = version
} else {
  version = inc(metadata[packageNickname])
  metadata[packageNickname] = version
}

writeFileSync(filePath, JSON.stringify(metadata, null, 2) + '\n')
console.log(`wrote ${version} for ${packageNickname} into metadata.json`)
console.dir(metadata)
