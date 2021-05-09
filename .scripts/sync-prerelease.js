#!/usr/bin/env node
// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/*
This script is solely intended to be run as part of the `covector version` step to
keep the `tauri-release` crate version without the `beta` or `beta-rc` suffix.
*/

const { readFileSync, writeFileSync } = require("fs")

const runtimeManifestPath = '../../core/tauri-runtime/Cargo.toml'
const dependencyManifestPaths = ['../../core/tauri/Cargo.toml']
const changelogPath = '../../core/tauri-runtime/CHANGELOG.md'

const bump = process.argv[2]

let runtimeManifest = readFileSync(runtimeManifestPath, "utf-8")
runtimeManifest = runtimeManifest.replace(/version = "(\d+\.\d+\.\d+)-[^0-9\.]+\.0"/, 'version = "$1"')
writeFileSync(runtimeManifestPath, runtimeManifest)

let changelog = readFileSync(changelogPath, "utf-8")
changelog = changelog.replace(/(\d+\.\d+\.\d+)-[^0-9\.]+\.0/, '$1')
writeFileSync(changelogPath, changelog)

for (const dependencyManifestPath of dependencyManifestPaths) {
  let dependencyManifest = readFileSync(dependencyManifestPath, "utf-8")
  dependencyManifest = dependencyManifest.replace(/tauri-runtime = { version = "(\d+\.\d+\.\d+)-[^0-9\.]+\.0"/, 'tauri-runtime = { version = "$1"')
  writeFileSync(dependencyManifestPath, dependencyManifest)
}
