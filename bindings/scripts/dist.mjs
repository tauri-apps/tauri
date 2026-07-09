#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Stages tauri-ffi release artifacts from CI build outputs. Used by
// .github/workflows/publish-ffi.yml; runnable locally against any directory
// laid out as <artifacts>/lib-<target>/<binaries>.
//
// Usage:
//   node bindings/scripts/dist.mjs c   --version 0.1.0 --artifacts <dir> --out <dir> [--require-all]
//   node bindings/scripts/dist.mjs npm --version 0.1.0 --artifacts <dir> --out <dir> [--require-all]
//
// `c` emits per-target archives (header + library + licenses), raw library
// assets (consumed by the Deno bindings' downloader) and SHA256SUMS.
// `npm` stages the @tauri-apps/node base package plus one platform package
// per target (optionalDependencies model, like esbuild/napi-rs).

import { execFileSync } from 'node:child_process'
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync
} from 'node:fs'
import { createHash } from 'node:crypto'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')

export const TARGETS = {
  'x86_64-apple-darwin': { platform: 'darwin', arch: 'x64', lib: 'libtauri_ffi.dylib' },
  'aarch64-apple-darwin': { platform: 'darwin', arch: 'arm64', lib: 'libtauri_ffi.dylib' },
  'x86_64-pc-windows-msvc': {
    platform: 'win32',
    arch: 'x64',
    lib: 'tauri_ffi.dll',
    extra: ['tauri_ffi.dll.lib']
  },
  'aarch64-pc-windows-msvc': {
    platform: 'win32',
    arch: 'arm64',
    lib: 'tauri_ffi.dll',
    extra: ['tauri_ffi.dll.lib']
  },
  'x86_64-unknown-linux-gnu': { platform: 'linux', arch: 'x64', lib: 'libtauri_ffi.so' },
  'aarch64-unknown-linux-gnu': { platform: 'linux', arch: 'arm64', lib: 'libtauri_ffi.so' }
}

const LICENSES = ['LICENSE.spdx', 'LICENSE_APACHE-2.0', 'LICENSE_MIT']

// ---------------------------------------------------------------------------

const [command, ...rest] = process.argv.slice(2)
const options = {}
for (let i = 0; i < rest.length; i++) {
  if (rest[i] === '--require-all') options.requireAll = true
  else if (rest[i].startsWith('--')) options[rest[i].slice(2)] = rest[++i]
}

if (!['c', 'npm'].includes(command) || !options.version || !options.artifacts || !options.out) {
  console.error(
    'usage: node bindings/scripts/dist.mjs <c|npm> --version <semver> --artifacts <dir> --out <dir> [--require-all]'
  )
  process.exit(2)
}

const artifactsDir = path.resolve(options.artifacts)
const outDir = path.resolve(options.out)
const version = options.version

/** Targets whose binaries are present under <artifacts>/lib-<target>/. */
function availableTargets() {
  const found = []
  const missing = []
  for (const [target, info] of Object.entries(TARGETS)) {
    const dir = path.join(artifactsDir, `lib-${target}`)
    if (existsSync(path.join(dir, info.lib))) found.push(target)
    else missing.push(target)
  }
  if (missing.length) {
    const message = `missing artifacts for: ${missing.join(', ')}`
    if (options.requireAll) {
      console.error(`error: ${message}`)
      process.exit(1)
    }
    console.warn(`warning: ${message} — continuing with ${found.length} target(s)`)
  }
  if (!found.length) {
    console.error('error: no artifacts found')
    process.exit(1)
  }
  return found
}

function binaries(target) {
  const info = TARGETS[target]
  const dir = path.join(artifactsDir, `lib-${target}`)
  const files = [info.lib, ...(info.extra ?? [])].filter((f) => existsSync(path.join(dir, f)))
  return files.map((f) => path.join(dir, f))
}

function sha256(file) {
  return createHash('sha256').update(readFileSync(file)).digest('hex')
}

// ---------------------------------------------------------------------------

function distC(targets) {
  mkdirSync(outDir, { recursive: true })
  const assets = []

  for (const target of targets) {
    const info = TARGETS[target]
    const stageName = `tauri-ffi-${version}-${target}`
    const stage = path.join(outDir, stageName)
    rmSync(stage, { recursive: true, force: true })
    mkdirSync(path.join(stage, 'include'), { recursive: true })
    mkdirSync(path.join(stage, 'lib'), { recursive: true })

    copyFileSync(
      path.join(repoRoot, 'bindings/c/tauri_ffi.h'),
      path.join(stage, 'include/tauri_ffi.h')
    )
    for (const binary of binaries(target)) {
      copyFileSync(binary, path.join(stage, 'lib', path.basename(binary)))
    }
    copyFileSync(path.join(repoRoot, 'bindings/c/README.md'), path.join(stage, 'README.md'))
    for (const license of LICENSES) {
      copyFileSync(path.join(repoRoot, license), path.join(stage, license))
    }

    const archive = `${stageName}.tar.gz`
    execFileSync('tar', ['-czf', path.join(outDir, archive), '-C', outDir, stageName])
    rmSync(stage, { recursive: true, force: true })
    assets.push(archive)

    // Raw library asset — consumed by the Deno bindings' first-run download.
    const extension = info.lib.slice(info.lib.lastIndexOf('.'))
    const raw = `tauri_ffi-${target}${extension}`
    copyFileSync(path.join(artifactsDir, `lib-${target}`, info.lib), path.join(outDir, raw))
    assets.push(raw)
  }

  const sums = assets.map((asset) => `${sha256(path.join(outDir, asset))}  ${asset}`).join('\n')
  writeFileSync(path.join(outDir, 'SHA256SUMS'), sums + '\n')
  console.log(`staged ${assets.length + 1} C release assets in ${outDir}`)
}

// ---------------------------------------------------------------------------

function platformPackageName(target) {
  const { platform, arch } = TARGETS[target]
  return `@tauri-apps/node-${platform}-${arch}`
}

function distNpm(targets) {
  const npmDir = path.join(outDir, 'npm')
  rmSync(npmDir, { recursive: true, force: true })

  for (const target of targets) {
    const info = TARGETS[target]
    const dir = path.join(npmDir, `node-${info.platform}-${info.arch}`)
    mkdirSync(dir, { recursive: true })
    copyFileSync(
      path.join(artifactsDir, `lib-${target}`, info.lib),
      path.join(dir, info.lib)
    )
    for (const license of LICENSES) copyFileSync(path.join(repoRoot, license), path.join(dir, license))
    writeFileSync(
      path.join(dir, 'package.json'),
      JSON.stringify(
        {
          name: platformPackageName(target),
          version,
          description: `${info.platform}-${info.arch} tauri-ffi library for @tauri-apps/node`,
          repository: { type: 'git', url: 'https://github.com/tauri-apps/tauri' },
          license: 'Apache-2.0 OR MIT',
          os: [info.platform],
          cpu: [info.arch],
          files: [info.lib],
          publishConfig: { access: 'public' }
        },
        null,
        2
      ) + '\n'
    )
  }

  // Base package: sources + rewritten manifest with optionalDependencies.
  const baseDir = path.join(npmDir, 'node')
  mkdirSync(baseDir, { recursive: true })
  cpSync(path.join(repoRoot, 'bindings/node/src'), path.join(baseDir, 'src'), { recursive: true })
  copyFileSync(path.join(repoRoot, 'bindings/node/README.md'), path.join(baseDir, 'README.md'))
  for (const license of LICENSES) copyFileSync(path.join(repoRoot, license), path.join(baseDir, license))

  const manifest = JSON.parse(readFileSync(path.join(repoRoot, 'bindings/node/package.json'), 'utf8'))
  delete manifest.private
  manifest.version = version
  manifest.files = ['src']
  manifest.publishConfig = { access: 'public' }
  manifest.repository = { type: 'git', url: 'https://github.com/tauri-apps/tauri' }
  manifest.license = 'Apache-2.0 OR MIT'
  manifest.optionalDependencies = Object.fromEntries(
    targets.map((target) => [platformPackageName(target), version])
  )
  writeFileSync(path.join(baseDir, 'package.json'), JSON.stringify(manifest, null, 2) + '\n')

  const staged = readdirSync(npmDir)
  console.log(`staged npm packages in ${npmDir}: ${staged.join(', ')}`)
  console.log('publish order: platform packages first, then the base package')
}

// ---------------------------------------------------------------------------

const targets = availableTargets()
if (command === 'c') distC(targets)
else distNpm(targets)
