#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Stages tauri-ffi release artifacts from CI build outputs, for one runtime at a
// time (--runtime, default wry). Used by .github/workflows/publish-ffi.yml;
// runnable locally against any directory laid out as
// <artifacts>/lib-<runtime>-<target>/<binaries>.
//
// Usage:
//   node bindings/scripts/dist.mjs c   --version 0.1.0 --runtime wry --artifacts <dir> --out <dir> [--require-all]
//   node bindings/scripts/dist.mjs npm --version 0.1.0 --runtime wry --artifacts <dir> --out <dir> [--require-all]
//
// `c` emits per-target archives (header + library + licenses), raw library
// assets (consumed by the Deno bindings' downloader) and a SHA256SUMS-<runtime>.
// `npm` stages the base package plus one platform package per target
// (optionalDependencies model, like esbuild/napi-rs). Each runtime ships its own
// library (libtauri_<runtime>); the shared C ABI (tauri_ffi.h) is runtime-agnostic.

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
import {
  TARGETS,
  RUNTIMES,
  DEFAULT_RUNTIME,
  libraryFiles,
  platformPackageName,
  artifactName
} from './targets.mjs'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')

const LICENSES = ['LICENSE.spdx', 'LICENSE-APACHE-2.0', 'LICENSE-MIT']

// ---------------------------------------------------------------------------

const [command, ...rest] = process.argv.slice(2)
const options = {}
for (let i = 0; i < rest.length; i++) {
  if (rest[i] === '--require-all') options.requireAll = true
  else if (rest[i].startsWith('--')) options[rest[i].slice(2)] = rest[++i]
}

if (!['c', 'npm'].includes(command) || !options.version || !options.artifacts || !options.out) {
  console.error(
    'usage: node bindings/scripts/dist.mjs <c|npm> --version <semver> [--runtime <name>] --artifacts <dir> --out <dir> [--require-all]'
  )
  process.exit(2)
}

const runtime = options.runtime ?? DEFAULT_RUNTIME
if (!RUNTIMES.includes(runtime)) {
  console.error(`error: unknown runtime '${runtime}' (known: ${RUNTIMES.join(', ')})`)
  process.exit(2)
}

const artifactsDir = path.resolve(options.artifacts)
const outDir = path.resolve(options.out)
const version = options.version

/** Targets whose binaries are present under <artifacts>/lib-<runtime>-<target>/. */
function availableTargets() {
  const found = []
  const missing = []
  for (const target of Object.keys(TARGETS)) {
    const dir = path.join(artifactsDir, artifactName(target, runtime))
    if (existsSync(path.join(dir, libraryFiles(target, runtime).lib))) found.push(target)
    else missing.push(target)
  }
  if (missing.length) {
    const message = `missing ${runtime} artifacts for: ${missing.join(', ')}`
    if (options.requireAll) {
      console.error(`error: ${message}`)
      process.exit(1)
    }
    console.warn(`warning: ${message} — continuing with ${found.length} target(s)`)
  }
  if (!found.length) {
    console.error(`error: no ${runtime} artifacts found`)
    process.exit(1)
  }
  return found
}

function binaries(target) {
  const { lib, extra } = libraryFiles(target, runtime)
  const dir = path.join(artifactsDir, artifactName(target, runtime))
  const files = [lib, ...extra].filter((f) => existsSync(path.join(dir, f)))
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
    const { lib } = libraryFiles(target, runtime)
    const stageName = `tauri-ffi-${version}-${runtime}-${target}`
    const stage = path.join(outDir, stageName)
    rmSync(stage, { recursive: true, force: true })
    mkdirSync(path.join(stage, 'include'), { recursive: true })
    mkdirSync(path.join(stage, 'lib'), { recursive: true })

    // The C ABI header is runtime-agnostic (same symbols for every runtime).
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
    const extension = lib.slice(lib.lastIndexOf('.'))
    const raw = `tauri_${runtime}-${target}${extension}`
    copyFileSync(
      path.join(artifactsDir, artifactName(target, runtime), lib),
      path.join(outDir, raw)
    )
    assets.push(raw)
  }

  // Per-runtime checksums so multiple runtimes can share one GitHub release
  // without their SHA256SUMS clobbering each other.
  const sums = assets.map((asset) => `${sha256(path.join(outDir, asset))}  ${asset}`).join('\n')
  const sumsFile = `SHA256SUMS-${runtime}`
  writeFileSync(path.join(outDir, sumsFile), sums + '\n')
  console.log(`staged ${assets.length + 1} C release assets (${runtime}) in ${outDir}`)
}

// ---------------------------------------------------------------------------

function distNpm(targets) {
  const npmDir = path.join(outDir, 'npm')

  for (const target of targets) {
    const { platform, arch } = TARGETS[target]
    const { lib, extra } = libraryFiles(target, runtime)
    // Staging folder names (`platform-…`/`base-…`) drive the publish order in
    // the workflow; the published package name comes from package.json.
    const dir = path.join(npmDir, `platform-${runtime}-${platform}-${arch}`)
    rmSync(dir, { recursive: true, force: true })
    mkdirSync(dir, { recursive: true })
    const files = [lib, ...extra].filter((f) =>
      existsSync(path.join(artifactsDir, artifactName(target, runtime), f))
    )
    for (const file of files) {
      copyFileSync(
        path.join(artifactsDir, artifactName(target, runtime), file),
        path.join(dir, file)
      )
    }
    for (const license of LICENSES) copyFileSync(path.join(repoRoot, license), path.join(dir, license))
    writeFileSync(
      path.join(dir, 'package.json'),
      JSON.stringify(
        {
          name: platformPackageName(target, runtime),
          version,
          description: `${platform}-${arch} ${runtime} library for @tauri-apps/node`,
          repository: { type: 'git', url: 'https://github.com/tauri-apps/tauri' },
          license: 'Apache-2.0 OR MIT',
          os: [platform],
          cpu: [arch],
          files,
          publishConfig: { access: 'public' }
        },
        null,
        2
      ) + '\n'
    )
  }

  // Base package: the default runtime keeps the canonical name (@tauri-apps/node);
  // additional runtimes get a suffixed base package (@tauri-apps/node-cef, …) so
  // installing one only pulls that runtime's (potentially large) library.
  const baseManifest = JSON.parse(
    readFileSync(path.join(repoRoot, 'bindings/node/package.json'), 'utf8')
  )
  const baseName =
    runtime === DEFAULT_RUNTIME ? baseManifest.name : `${baseManifest.name}-${runtime}`
  const baseDir = path.join(npmDir, `base-${runtime}`)
  rmSync(baseDir, { recursive: true, force: true })
  mkdirSync(baseDir, { recursive: true })
  cpSync(path.join(repoRoot, 'bindings/node/src'), path.join(baseDir, 'src'), { recursive: true })
  // Bake the runtime this package loads (dev/source default is wry).
  if (runtime !== DEFAULT_RUNTIME) {
    const ffiFile = path.join(baseDir, 'src', 'ffi.js')
    const source = readFileSync(ffiFile, 'utf8')
    const stamped = source.replace(
      /const DEFAULT_RUNTIME = '[^']*'/,
      `const DEFAULT_RUNTIME = '${runtime}'`
    )
    if (stamped === source) {
      console.error(`error: could not stamp DEFAULT_RUNTIME into ${ffiFile}`)
      process.exit(1)
    }
    writeFileSync(ffiFile, stamped)
  }
  copyFileSync(path.join(repoRoot, 'bindings/node/README.md'), path.join(baseDir, 'README.md'))
  for (const license of LICENSES) copyFileSync(path.join(repoRoot, license), path.join(baseDir, license))

  const manifest = baseManifest
  delete manifest.private
  manifest.name = baseName
  manifest.version = version
  manifest.files = ['src']
  manifest.publishConfig = { access: 'public' }
  manifest.repository = { type: 'git', url: 'https://github.com/tauri-apps/tauri' }
  manifest.license = 'Apache-2.0 OR MIT'
  manifest.optionalDependencies = Object.fromEntries(
    targets.map((target) => [platformPackageName(target, runtime), version])
  )
  // The workspace manifest points @tauri-apps/api at the local built dist
  // (packages/api/dist) for dev; publish needs a real semver range instead.
  if (manifest.dependencies?.['@tauri-apps/api']) {
    const apiPkg = JSON.parse(readFileSync(path.join(repoRoot, 'packages/api/package.json'), 'utf8'))
    manifest.dependencies['@tauri-apps/api'] = `^${apiPkg.version}`
  }
  writeFileSync(path.join(baseDir, 'package.json'), JSON.stringify(manifest, null, 2) + '\n')

  const staged = readdirSync(npmDir).filter((name) => name.endsWith(`-${runtime}`) || name.includes(`-${runtime}-`))
  console.log(`staged ${runtime} npm packages in ${npmDir}: ${staged.join(', ')}`)
  console.log('publish order: platform packages (platform-*) first, then the base package (base-*)')
}

// ---------------------------------------------------------------------------

const targets = availableTargets()
if (command === 'c') distC(targets)
else distNpm(targets)
