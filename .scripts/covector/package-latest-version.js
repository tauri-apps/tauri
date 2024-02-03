#!/usr/bin/env node

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/*
This script is solely intended to be run as part of the `covector publish` step to
check the latest version of a crate, considering the current minor version.
*/

const https = require('https')

const kind = process.argv[2]
const packageName = process.argv[3]
const packageVersion = process.argv[4]
const target = packageVersion.substring(0, packageVersion.lastIndexOf('.'))

let url = null
switch (kind) {
  case 'cargo':
    url = `https://crates.io/api/v1/crates/${packageName}`
    break
  case 'npm':
    url = `https://registry.npmjs.org/${packageName}`
    break
  default:
    throw new Error('unexpected kind ' + kind)
}

const options = {
  headers: {
    'Content-Type': 'application/json',
    Accept: 'application/json',
    'User-Agent': 'tauri (https://github.com/tauri-apps/tauri)'
  }
}

https.get(url, options, (response) => {
  let chunks = []
  response.on('data', function (chunk) {
    chunks.push(chunk)
  })

  response.on('end', function () {
    const data = JSON.parse(chunks.join(''))
    if (kind === 'cargo') {
      const versions = data.versions?.filter((v) => v.num.startsWith(target)) ?? []
      console.log(versions.length ? versions[0].num : '0.0.0')
    } else if (kind === 'npm') {
      const versions = Object.keys(data.versions).filter((v) =>
        v.startsWith(target)
      )
      console.log(versions[versions.length - 1] || '0.0.0')
    }
  })
})
