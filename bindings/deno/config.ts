// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// tauri.conf.json support for launch(): config discovery next to the app,
// TAURI_CONFIG environment merging (set by the Tauri CLI in dev mode with the
// fully merged config) and frontendDist-based asset resolution.

// deno-lint-ignore no-explicit-any
type Json = any

const isObject = (value: unknown): value is Record<string, Json> =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

/**
 * Whether this is a compiled, distributable binary (`deno compile`) rather
 * than a `deno run` of source. Such a binary must be hermetic: the
 * environment-variable config/dev overrides the Tauri CLI sets in dev
 * (`TAURI_CONFIG`, `TAURI_DEV`) are ignored so a shipped bundle can't be
 * repointed at attacker-controlled config or a dev URL through its env.
 */
export function isBundled(): boolean {
  return (Deno.build as { standalone?: boolean }).standalone === true
}

/**
 * The directory holding this app's bundled resources (cdylib, packed assets,
 * config) when running as a `deno compile` binary inside a Tauri bundle, or
 * `null` in dev (`deno run`). Resolved from `Deno.execPath()`:
 * `.app/Contents/Resources` on macOS, the executable's directory elsewhere.
 */
export function bundledResourceDir(): string | null {
  let exec: string
  try {
    exec = Deno.execPath()
  } catch {
    return null
  }
  const macos = exec.indexOf('/Contents/MacOS/')
  if (macos !== -1) return exec.slice(0, macos) + '/Contents/Resources'
  const slash = Math.max(exec.lastIndexOf('/'), exec.lastIndexOf('\\'))
  return slash === -1 ? null : exec.slice(0, slash)
}

/** JSON merge patch (like the CLI's config merging): objects merge
 * recursively, null removes the key, everything else replaces. */
export function mergeConfig(target: Record<string, Json>, source: Record<string, Json>): Record<string, Json> {
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

function readConfigIn(dir: string): Record<string, Json> | null {
  try {
    return JSON.parse(Deno.readTextFileSync(`${dir}/tauri.conf.json`))
  } catch {
    return null
  }
}

/**
 * Resolves the app configuration: the explicit `config` option, or a
 * `tauri.conf.json` found next to the app entry (then in the working
 * directory). In dev, it is deep-merged with the `TAURI_CONFIG` environment
 * variable (the Tauri CLI passes the fully merged config through it); a
 * compiled bundle ignores that env override — see {@linkcode isBundled}.
 */
export function resolveConfig(
  entryDir: string | null,
  explicit: unknown
): { config: Record<string, Json>; configDir: string | null } {
  let config: Record<string, Json> | null = null
  let configDir: string | null = null

  if (explicit) {
    config = structuredClone(explicit) as Record<string, Json>
  } else {
    // resource dir first so a bundled app uses its packed config
    for (const dir of [bundledResourceDir(), entryDir, Deno.cwd()]) {
      if (!dir) continue
      const found = readConfigIn(dir)
      if (found) {
        config = found
        configDir = dir
        break
      }
    }
  }

  const env = isBundled() ? undefined : Deno.env.get('TAURI_CONFIG')
  if (env) {
    config = mergeConfig(config ?? {}, JSON.parse(env))
  }
  if (!config) {
    throw new Error(
      'no configuration found — pass `config` to launch() or add a tauri.conf.json next to your app entry'
    )
  }

  return { config, configDir }
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
 */
export function resolveAssets(
  options: { assetsDir?: string; assetsArchive?: string },
  config: Record<string, Json>,
  configDir: string | null
): { dir?: string; archive?: string } {
  if (options.assetsDir) return { dir: options.assetsDir }
  if (options.assetsArchive) return { archive: options.assetsArchive }

  const dist = config?.build?.frontendDist
  if (typeof dist !== 'string' || /^https?:/.test(dist)) return {}
  const base = configDir ?? Deno.cwd()
  const resolved = dist.startsWith('/') ? dist : `${base}/${dist.replace(/^\.\//, '')}`
  return dist.endsWith('.assets') ? { archive: resolved } : { dir: resolved }
}
