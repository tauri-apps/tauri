#!/usr/bin/env node

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const fs = require('fs')
const path = require('path')
const ignorePackages = [
  'tauri-macros',
  'tauri-codegen',
  'tauri-runtime',
  'tauri-runtime-wry',
  'tauri-driver'
]

const missingTagsFiles = {}

const changeFiles = fs
  .readdirSync('.changes')
  .filter((f) => f.endsWith('.md') && f.toLowerCase() !== 'readme.md')
for (const file of changeFiles) {
  const content = fs.readFileSync(path.join('.changes', file), 'utf8')
  const [frontMatter] = /^---[\s\S.]*---\n/i.exec(content)
  const packages = frontMatter
    .split('\n')
    .filter((l) => !(l === '---' || !l))
    .map((l) => l.split(':'))

  for (const [p, _, tag] of packages) {
    const package = p.replace(/('|")/g, '')
    if (ignorePackages.includes(package)) continue
    if (!tag) {
      if (!missingTagsFiles[file]) missingTagsFiles[file] = []
      missingTagsFiles[file].push(package)
    }
  }
}

if (Object.keys(missingTagsFiles).length !== 0) {
  for (const [file, packages] of Object.entries(missingTagsFiles)) {
    for (const package of packages) {
      console.error(
        `Package \`${package}\` is missing a change tag in ${path.join(
          '.changes',
          file
        )} `
      )
    }
  }
  process.exit(1)
}
