#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
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

function checkChangeFiles(changeFiles) {
  for (const file of changeFiles) {
    const content = fs.readFileSync(file, 'utf8')
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
          `Package \`${package}\` is missing a change tag in ${file} `
        )
      }
    }

    for (const [file, packages] of unknownTagsEntries) {
      for (const { package, tag } of packages) {
        console.error(
          `Package \`${package}\` has an uknown change tag ${tag} in ${file} `
        )
      }
    }

    process.exit(1)
  }
}

const [_bin, _script, ...files] = process.argv

if (files.length > 0) {
  checkChangeFiles(
    files.filter((f) => f.toLowerCase() !== '.changes/readme.md')
  )
} else {
  const changeFiles = fs
    .readdirSync('.changes')
    .filter((f) => f.endsWith('.md') && f.toLowerCase() !== 'readme.md')
    .map((p) => path.join('.changes', p))
  checkChangeFiles(changeFiles)
}
