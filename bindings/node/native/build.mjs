#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Builds `tauri_node.node`, the Node-API addon that runs the tauri-ffi event
// loop off koffi's private stack (see tauri_node.c for why it has to exist).
//
// It is one dependency-free C file against the Node-API headers, so it builds
// with a plain compiler invocation rather than node-gyp — keeping the bindings'
// "no gyp on the user's machine" property. The addon links nothing: `napi_*`
// resolves from the host node binary when the addon is loaded, which is why the
// linker is told to allow undefined symbols.
//
// Usage:
//   node bindings/node/native/build.mjs [--out <file>] [--target <rust triple>]
//
// `--target` takes the same triples as the Rust build matrix, so CI can build
// the addon next to the library it ships with. Only the arch is cross-built
// (Apple and MSVC toolchains do that from either host); Linux arm64 builds on a
// native runner, as the library does.
//
// Headers come from the `node-api-headers` package. On Windows the import
// library it ships is required, because PE cannot leave symbols undefined.

import { execFileSync } from 'node:child_process'
import { createRequire } from 'node:module'
import { mkdirSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const require = createRequire(import.meta.url)

const args = process.argv.slice(2)
const flag = (name, fallback) => {
  const i = args.indexOf(name)
  return i === -1 ? fallback : args[i + 1]
}

const out = flag('--out', path.join(here, 'tauri_node.node'))
const target = flag('--target', null)
mkdirSync(path.dirname(out), { recursive: true })

const arch = target
  ? target.startsWith('aarch64')
    ? 'arm64'
    : 'x64'
  : process.arch === 'arm64'
    ? 'arm64'
    : 'x64'
const platform = target
  ? target.includes('windows')
    ? 'win32'
    : target.includes('apple')
      ? 'darwin'
      : 'linux'
  : process.platform

const include = path.join(
  path.dirname(require.resolve('node-api-headers/package.json')),
  'include'
)
const source = path.join(here, 'tauri_node.c')

if (platform === 'win32') {
  // PE resolves imports at link time, so the napi_* symbols need an import
  // library — generated here from the .def node-api-headers ships.
  const defs = path.join(path.dirname(include), 'def', 'node_api.def')
  const lib = path.join(path.dirname(out), 'node_api.lib')
  execFileSync(
    'lib',
    [
      `/def:${defs}`,
      `/out:${lib}`,
      `/machine:${arch === 'arm64' ? 'arm64' : 'x64'}`
    ],
    {
      stdio: 'inherit'
    }
  )
  execFileSync(
    'cl',
    [
      '/nologo',
      '/O2',
      '/LD',
      `/I${include}`,
      source,
      `/Fe:${out}`,
      '/link',
      lib
    ],
    {
      stdio: 'inherit',
      cwd: path.dirname(out)
    }
  )
} else {
  const cc = process.env.CC || 'cc'
  const flags = ['-O2', '-shared', '-fPIC', `-I${include}`, source, '-o', out]
  if (platform === 'darwin') {
    // Mach-O has no undefined-symbol default: napi_* arrives from the host node
    // binary at dlopen time, so the linker has to be told to leave it alone.
    flags.push('-undefined', 'dynamic_lookup')
    flags.push('-arch', arch === 'arm64' ? 'arm64' : 'x86_64')
  } else {
    flags.push('-ldl')
  }
  execFileSync(cc, flags, { stdio: 'inherit' })
}

console.log(`built ${out}`)
