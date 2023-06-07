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

const covectorConfig = JSON.parse(
  fs.readFileSync('.changes/config.json', 'utf8')
)
const tags = Object.keys(covectorConfig.changeTags)

const missingTagsFiles = {}
const unknownTagsFiles = {}

const changeFiles = fs
  .readdirSync('.changes')
  .filter((f) => f.endsWith('.md') && f.toLowerCase() !== 'readme.md')
for (const file of changeFiles) {
  const content = fs.readFileSync(path.join('.changes', file), 'utf8')
  const [frontMatter] = /^---[\s\S.]*---\n/i.exec(content)
  const packages = frontMatter
    .split('\n')
    .filter((l) => !(l === '---' || !l))
    .map((l) => l.replace(/('|")/g, '').split(':'))

  for (const [package, _, tag] of packages) {
    if (!tag) {
      if (ignorePackages.includes(package)) continue

      if (!missingTagsFiles[file]) missingTagsFiles[file] = []
      missingTagsFiles[file].push(package)
    } else if (!tags.includes(tag)) {
      if (!unknownTagsFiles[file]) unknownTagsFiles[file] = []
      unknownTagsFiles[file].push({ package, tag })
    }
  }
}
const missingTagsEntries = Object.entries(missingTagsFiles)
const unknownTagsEntries = Object.entries(unknownTagsFiles)
if (missingTagsEntries.length > 0 || unknownTagsEntries.length > 0) {
  for (const [file, packages] of missingTagsEntries) {
    for (const package of packages) {
      console.error(
        `Package \`${package}\` is missing a change tag in ${path.join(
          '.changes',
          file
        )} `
      )
    }
  }

  for (const [file, packages] of unknownTagsEntries) {
    for (const { package, tag } of packages) {
      console.error(
        `Package \`${package}\` has an uknown change tag ${tag} in ${path.join(
          '.changes',
          file
        )} `
      )
    }
  }

  process.exit(1)
}
