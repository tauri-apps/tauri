// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// tauri.conf.json support for launch(): config discovery next to the app,
// TAURI_CONFIG environment merging (set by the Tauri CLI in dev mode with the
// fully merged config) and frontendDist-based asset resolution.

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { createRequire } from 'node:module'
import path from 'node:path'

const isObject = (value) =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

/**
 * Whether this is a compiled, distributable binary (Node Single Executable
 * Application) rather than a plain `node main.js` run. Such a binary must be
 * hermetic: the environment-variable config/dev overrides the Tauri CLI sets
 * in dev (`TAURI_CONFIG`, `TAURI_DEV`) are ignored so a shipped bundle can't
 * be repointed at attacker-controlled config or a dev URL through its env.
 */
export function isBundled() {
  try {
    return createRequire(import.meta.url)('node:sea').isSea()
  } catch {
    return false
  }
}

/**
 * The directory holding this app's bundled resources (cdylib, koffi native
 * module, packed assets, config) when running as a Single Executable
 * Application inside a Tauri bundle, or `null` in dev (`node main.js`).
 * Resolved from the executable path: `.app/Contents/Resources` on macOS, the
 * executable's directory elsewhere.
 */
export function bundledResourceDir() {
  if (!isBundled()) return null
  const exec = process.execPath
  const macos = exec.indexOf('/Contents/MacOS/')
  if (macos !== -1) return exec.slice(0, macos) + '/Contents/Resources'
  return path.dirname(exec)
}

/** JSON merge patch (like the CLI's config merging): objects merge
 * recursively, null removes the key, everything else replaces. */
export function mergeConfig(target, source) {
  for (const [key, value] of Object.entries(source)) {
    if (value === null) {
      delete target[key]
    } else if (isObject(value) && isObject(target[key])) {
      mergeConfig(target[key], value)
    } else {
      target[key] = isObject(value) ? mergeConfig({}, value) : value
    }
  }
  return target
}

/**
 * Resolves the app configuration:
 * 1. the explicit `config` option, or a `tauri.conf.json` found next to the
 *    app entry (then in the working directory);
 * 2. in dev, deep-merged with the `TAURI_CONFIG` environment variable (the
 *    Tauri CLI passes the fully merged config through it, including `--config`
 *    overrides and the built-in dev server URL). A compiled bundle ignores
 *    that env override — see {@link isBundled}.
 *
 * Returns `{ config, configDir }`; `configDir` anchors relative
 * `build.frontendDist` paths (null when the config was passed inline).
 */
export function resolveConfig(entryDir, explicit) {
  let config = null
  let configDir = null

  if (explicit) {
    config = structuredClone(explicit)
  } else {
    // resource dir first so a bundled app uses its packed config
    for (const dir of [bundledResourceDir(), entryDir, process.cwd()]) {
      if (!dir) continue
      const file = path.join(dir, 'tauri.conf.json')
      if (existsSync(file)) {
        config = JSON.parse(readFileSync(file, 'utf8'))
        configDir = dir
        break
      }
    }
  }

  if (!isBundled() && process.env.TAURI_CONFIG) {
    config = mergeConfig(config ?? {}, JSON.parse(process.env.TAURI_CONFIG))
  }
  if (!config) {
    throw new Error(
      'no configuration found — pass `config` to launch() or add a tauri.conf.json next to your app entry'
    )
  }

  return { config, configDir }
}

// Capability files carry these extensions; a `schemas/` subfolder holds JSON
// schema files, not capabilities, so it is skipped (mirrors tauri-build).
const CAPABILITY_EXTENSIONS = new Set(['.json', '.json5', '.toml'])

function walkCapabilityFiles(dir, files = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      if (entry.name !== 'schemas') walkCapabilityFiles(full, files)
    } else if (CAPABILITY_EXTENSIONS.has(path.extname(entry.name).toLowerCase())) {
      files.push(full)
    }
  }
  return files
}

/**
 * Reads capability files from a `capabilities/` directory next to the config,
 * mirroring the compile-time discovery a Rust app gets from tauri-build
 * (`capabilities/**` glob). Each file's raw content is a capability-file string
 * (JSON or TOML; a single capability or a list) handed to `add_capability`.
 *
 * The directory is looked up next to the resolved config (then the app entry,
 * then the bundled resource dir). Returns `[]` when there is none — the app
 * then falls back to the built-in `core:default` capability.
 */
export function resolveCapabilities(dirs) {
  for (const base of dirs) {
    if (!base) continue
    const capDir = path.join(base, 'capabilities')
    if (existsSync(capDir) && statSync(capDir).isDirectory()) {
      return walkCapabilityFiles(capDir)
        .sort()
        .map((file) => readFileSync(file, 'utf8'))
    }
  }
  return []
}

/**
 * Resolves where frontend assets come from, in precedence order: explicit
 * launch options, then the config's `build.frontendDist` (a `.assets` archive
 * — e.g. one packed by `tauri build`, resolved next to the config — or a
 * directory; URLs are handled by the Tauri runtime itself).
 *
 * The asset source is deliberately not overridable by an environment
 * variable: the frontend is trusted content (it can invoke commands), so its
 * origin must come from code or the bundled config, never the ambient env.
 *
 * Returns `{ dir }`, `{ archive }` or `{}`.
 */
export function resolveAssets({ assetsDir, assetsArchive }, config, configDir) {
  if (assetsDir) return { dir: assetsDir }
  if (assetsArchive) return { archive: assetsArchive }

  const dist = config?.build?.frontendDist
  if (typeof dist !== 'string' || /^https?:/.test(dist)) return {}
  const resolved = path.resolve(configDir ?? process.cwd(), dist)
  return dist.endsWith('.assets') ? { archive: resolved } : { dir: resolved }
}
