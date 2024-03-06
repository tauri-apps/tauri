#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const fs = require('fs')
const path = require('path')
const readline = require('readline')

const header = `Copyright 2019-2024 Tauri Programme within The Commons Conservancy
SPDX-License-Identifier: Apache-2.0
SPDX-License-Identifier: MIT`
const bundlerLicense =
  '// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>'
const denoLicense =
  '// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.'

const extensions = ['.rs', '.js', '.ts', '.yml', '.swift', '.kt']
const ignore = [
  'target',
  'templates',
  'node_modules',
  'gen',
  'dist',
  'bundle.global.js'
]

async function checkFile(file) {
  if (
    extensions.some((e) => file.endsWith(e)) &&
    !ignore.some((i) => file.includes(`/${i}/`) || path.basename(file) == i)
  ) {
    const fileStream = fs.createReadStream(file)
    const rl = readline.createInterface({
      input: fileStream,
      crlfDelay: Infinity
    })

    let contents = ``
    let i = 0
    for await (let line of rl) {
      // ignore empty lines, allow shebang and bundler license
      if (
        line.length === 0 ||
        line.startsWith('#!') ||
        line.startsWith('// swift-tools-version:') ||
        line === bundlerLicense ||
        line === denoLicense
      ) {
        continue
      }

      // strip comment marker
      if (line.startsWith('// ')) {
        line = line.substring(3)
      } else if (line.startsWith('# ')) {
        line = line.substring(2)
      }

      contents += line
      if (++i === 3) {
        break
      }
      contents += '\n'
    }
    if (contents !== header) {
      return true
    }
  }
  return false
}

async function check(src) {
  const missingHeader = []

  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const p = path.join(src, entry.name)

    if (entry.isSymbolicLink() || ignore.includes(entry.name)) {
      continue
    }

    if (entry.isDirectory()) {
      const missing = await check(p)
      missingHeader.push(...missing)
    } else {
      const isMissing = await checkFile(p)
      if (isMissing) {
        missingHeader.push(p)
      }
    }
  }

  return missingHeader
}

const [_bin, _script, ...files] = process.argv

if (files.length > 0) {
  async function run() {
    const missing = []
    for (const f of files) {
      const isMissing = await checkFile(f)
      if (isMissing) {
        missing.push(f)
      }
    }
    if (missing.length > 0) {
      console.log(missing.join('\n'))
      process.exit(1)
    }
  }

  run()
} else {
  check('.').then((missing) => {
    if (missing.length > 0) {
      console.log(missing.join('\n'))
      process.exit(1)
    }
  })
}
