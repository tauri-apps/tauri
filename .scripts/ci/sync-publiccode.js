#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/*
This script is solely intended to be run as part of the `covector version` step to
keep `softwareVersion` and `releaseDate` in `../../publiccode.yml` in sync with the
`tauri` crate version.
*/

const { readFileSync, writeFileSync } = require('fs')
const { resolve } = require('path')

const cargoTomlPath = resolve(__dirname, '../../crates/tauri/Cargo.toml')
const publiccodePath = resolve(__dirname, '../../publiccode.yml')

const version = readFileSync(cargoTomlPath, 'utf-8').match(
  /^version\s*=\s*"([^"]+)"/m
)?.[1]
if (!version) {
  throw new Error(`could not find the tauri version in ${cargoTomlPath}`)
}
const releaseDate = new Date().toISOString().slice(0, 10)

let publiccode = readFileSync(publiccodePath, 'utf-8')
for (const [key, value] of [
  ['softwareVersion', version],
  ['releaseDate', releaseDate]
]) {
  const re = new RegExp(`^${key}:.*$`, 'm')
  if (!re.test(publiccode)) {
    throw new Error(`could not find ${key} in ${publiccodePath}`)
  }
  publiccode = publiccode.replace(re, `${key}: "${value}"`)
}

writeFileSync(publiccodePath, publiccode)
console.log(`wrote ${version} (${releaseDate}) into publiccode.yml`)
