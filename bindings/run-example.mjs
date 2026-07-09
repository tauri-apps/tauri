#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Runs a tauri-ffi example in any bound language. Detects the language from
// the entry file, builds the cdylib, and handles per-language prerequisites
// (koffi install, cffi check, C compilation).
//
// Usage:
//   node bindings/run-example.mjs <path-or-shorthand> [--no-build]
//   node bindings/run-example.mjs --list
//
// Examples:
//   node bindings/run-example.mjs node                    # bindings/node/examples/hello
//   node bindings/run-example.mjs python/hello
//   node bindings/run-example.mjs bindings/deno/examples/hello
//   node bindings/run-example.mjs c
//
// Environment: TAURI_FFI_LIB (override library path, passed through),
// PYTHON (python interpreter, default python3), CC (C compiler, default cc).

import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readdirSync, statSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const bindingsDir = path.join(repoRoot, 'bindings')

const argv = process.argv.slice(2)
const flags = new Set(argv.filter((a) => a.startsWith('--')))
const target = argv.find((a) => !a.startsWith('--'))

const LANGUAGES = ['node', 'deno', 'python', 'c']

function fail(message) {
  console.error(`error: ${message}`)
  process.exit(1)
}

function listExamples() {
  const rows = []
  for (const lang of LANGUAGES) {
    const dir = path.join(bindingsDir, lang, 'examples')
    if (!existsSync(dir)) continue
    for (const entry of readdirSync(dir)) {
      const full = path.join(dir, entry)
      if (statSync(full).isDirectory() || entry.endsWith('.c')) {
        rows.push([lang, path.relative(repoRoot, full)])
      }
    }
  }
  const width = Math.max(...rows.map(([lang]) => lang.length))
  for (const [lang, example] of rows) console.log(`${lang.padEnd(width)}  ${example}`)
}

/** Accepts a real path or a `<lang>[/<name>]` shorthand (name defaults to hello). */
function resolveTarget(input) {
  for (const base of [path.resolve(input), path.resolve(repoRoot, input)]) {
    if (existsSync(base)) return base
  }
  const [lang, ...rest] = input.split('/')
  if (LANGUAGES.includes(lang)) {
    const name = rest.join('/') || 'hello'
    for (const candidate of [
      path.join(bindingsDir, lang, 'examples', name),
      path.join(bindingsDir, lang, 'examples', `${name}.c`)
    ]) {
      if (existsSync(candidate)) return candidate
    }
  }
  fail(`example not found: ${input} (try --list)`)
}

/** Returns { language, entry } for a resolved file or directory. */
function detectEntry(resolved) {
  if (statSync(resolved).isDirectory()) {
    const candidates = ['main.js', 'main.mjs', 'main.ts', 'main.py']
    for (const name of candidates) {
      const entry = path.join(resolved, name)
      if (existsSync(entry)) return detectEntry(entry)
    }
    const cFile = readdirSync(resolved).find((f) => f.endsWith('.c'))
    if (cFile) return detectEntry(path.join(resolved, cFile))
    fail(`no entry (${candidates.join(', ')} or *.c) found in ${resolved}`)
  }
  const language = { '.js': 'node', '.mjs': 'node', '.ts': 'deno', '.py': 'python', '.c': 'c' }[
    path.extname(resolved)
  ]
  if (!language) fail(`cannot infer language from ${resolved}`)
  return { language, entry: resolved }
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', ...options })
  if (result.error?.code === 'ENOENT') fail(`\`${command}\` not found on PATH`)
  return result.status ?? 1
}

function libDir() {
  const name = { darwin: 'libtauri_ffi.dylib', linux: 'libtauri_ffi.so', win32: 'tauri_ffi.dll' }[
    process.platform
  ]
  for (const profile of ['debug', 'release']) {
    const dir = path.join(repoRoot, 'target', profile)
    if (existsSync(path.join(dir, name))) return dir
  }
  fail('tauri_ffi library not found after build')
}

// ---------------------------------------------------------------------------

if (flags.has('--list')) {
  listExamples()
  process.exit(0)
}
if (!target) {
  console.error('usage: node bindings/run-example.mjs <path-or-shorthand> [--no-build] | --list')
  process.exit(2)
}

const { language, entry } = detectEntry(resolveTarget(target))

if (!flags.has('--no-build')) {
  console.log('› cargo build -p tauri-ffi')
  if (run('cargo', ['build', '-p', 'tauri-ffi'], { cwd: repoRoot }) !== 0) {
    fail('cargo build failed')
  }
}

console.log(`› running ${path.relative(repoRoot, entry)} (${language})`)
let status
switch (language) {
  case 'node': {
    const pkgDir = path.join(bindingsDir, 'node')
    if (!existsSync(path.join(pkgDir, 'node_modules', 'koffi'))) {
      console.log('› pnpm install --filter @tauri-apps/node')
      if (run('pnpm', ['install', '--filter', '@tauri-apps/node'], { cwd: repoRoot }) !== 0) {
        fail('pnpm install failed (enable pnpm via `corepack enable`)')
      }
    }
    status = run(process.execPath, [entry])
    break
  }

  case 'deno': {
    status = run('deno', ['run', '-A', entry])
    break
  }

  case 'python': {
    const python = process.env.PYTHON || 'python3'
    const probe = spawnSync(python, ['-c', 'import cffi'], { stdio: 'pipe' })
    if (probe.status !== 0) {
      fail(
        `\`${python}\` cannot import cffi — install it (\`pip install cffi\`) or point PYTHON at an interpreter that has it`
      )
    }
    status = run(python, [entry])
    break
  }

  case 'c': {
    if (process.platform === 'win32') fail('C example compilation is not wired up for Windows yet')
    const cc = process.env.CC || 'cc'
    const outDir = path.join(repoRoot, 'target', 'ffi-examples')
    mkdirSync(outDir, { recursive: true })
    const binary = path.join(outDir, path.basename(entry, '.c'))
    const lib = libDir()
    console.log(`› ${cc} ${path.relative(repoRoot, entry)} -o ${path.relative(repoRoot, binary)}`)
    const compile = run(cc, [
      entry,
      '-I',
      path.join(bindingsDir, 'c'),
      '-L',
      lib,
      '-ltauri_ffi',
      `-Wl,-rpath,${lib}`,
      '-o',
      binary
    ])
    if (compile !== 0) fail('C compilation failed')
    status = run(binary, [])
    break
  }
}

process.exit(status)
