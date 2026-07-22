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

function exists(path: string): boolean {
  try {
    Deno.statSync(path)
    return true
  } catch {
    return false
  }
}

/** `productName` from the embedded config — the directory name Linux packages
 * derive their resource path from (`/usr/lib/<productName>`). */
function bundledProductName(): string | null {
  const text = readEmbeddedText('config.json')
  if (!text) return null
  try {
    return JSON.parse(text).productName ?? null
  } catch {
    return null
  }
}

// Empty marker file the CLI stages into the resource dir, so we can pick it out
// among the candidate locations below — and not mistake a user's own
// `resources/` directory for it. Kept in sync with the Tauri CLI's
// `RESOURCE_MARKER`.
const RESOURCE_MARKER = '.tauri-resources'

/**
 * The directory holding this app's bundled resources (cdylib, libcef and its
 * pak/locale files for a cef build) when running as a `deno compile` binary
 * inside a Tauri bundle, or `null` in dev (`deno run`).
 *
 * Resolved from `Deno.execPath()`, mirroring what `tauri_utils::platform`'s
 * `resource_dir` does for a Rust app — each packaging format puts resources
 * somewhere different relative to the binary. We return the first candidate
 * holding the `.tauri-resources` marker; a flat layout with no marker falls
 * back to the executable's own directory, which {@link libraryPath} verifies
 * before use:
 * - macOS `.app`: `Contents/MacOS/<bin>` → `Contents/Resources`
 * - `tauri build` output, run from `dist/<profile>` without packaging: the
 *   sibling `resources/` the CLI stages into
 * - Windows installers: next to the executable
 * - Linux deb/rpm: binary in `/usr/bin`, resources in `/usr/lib/<productName>`
 *   (a cef build moves the binary to `/usr/share/<productName>/<bin>`)
 * - Linux AppImage: the same, under `$APPDIR`
 */
export function bundledResourceDir(): string | null {
  if (!isBundled()) return null
  let exec: string
  try {
    exec = Deno.execPath()
  } catch {
    return null
  }
  const candidates: string[] = []
  const macos = exec.indexOf('/Contents/MacOS/')
  if (macos !== -1)
    candidates.push(exec.slice(0, macos) + '/Contents/Resources')
  const slash = Math.max(exec.lastIndexOf('/'), exec.lastIndexOf('\\'))
  if (slash === -1) return null
  const dir = exec.slice(0, slash)
  candidates.push(`${dir}/resources`) // unpackaged `tauri build` output
  candidates.push(dir) // installers that stage flat next to the exe (Windows)

  if (Deno.build.os === 'linux') {
    const name = bundledProductName()
    if (name) {
      // `../lib/<name>` covers deb/rpm (/usr/bin → /usr/lib/<name>) and an
      // AppImage whose AppDir mirrors that layout; `../../lib/<name>` covers a
      // cef build, whose bundler moves the binary to /usr/share/<name>/<bin> so
      // libcef resolves via $ORIGIN (Deno.execPath() follows the /usr/bin
      // symlink back to it). The APPDIR and absolute fallbacks match the order
      // tauri_utils falls back through.
      const parent = dir.slice(0, dir.lastIndexOf('/'))
      candidates.push(`${parent}/lib/${name}`)
      candidates.push(`${parent.slice(0, parent.lastIndexOf('/'))}/lib/${name}`)
      const appdir = Deno.env.get('APPDIR')
      if (appdir) candidates.push(`${appdir}/usr/lib/${name}`)
      candidates.push(`/usr/lib/${name}`)
    }
  }

  for (const candidate of candidates) {
    if (exists(`${candidate}/${RESOURCE_MARKER}`)) return candidate
  }
  return macos !== -1 ? candidates[0] : dir // best guess: a flat layout with no marker
}

/**
 * The `file://` URL of the payload embedded in a `deno compile` binary
 * (`app.assets`, `config.json`, `capabilities.json`), or `null` in dev. The
 * CLI stages it beside the app entry as `.tauri-embed/` and `deno compile`
 * maps it into a virtual FS rooted at the main module, so it is read back
 * relative to {@linkcode Deno.mainModule}. This is how a shipped bundle carries
 * its assets/config/ACL *inside* the executable rather than as sibling files.
 */
export function embeddedDir(): string | null {
  if (!isBundled()) return null
  try {
    return new URL('./.tauri-embed/', Deno.mainModule).href
  } catch {
    return null
  }
}

/** Reads an embedded payload file as bytes (see {@linkcode embeddedDir}), or
 * `null` when not bundled or the file is absent. */
export function readEmbeddedBytes(name: string): Uint8Array | null {
  const dir = embeddedDir()
  if (!dir) return null
  try {
    return Deno.readFileSync(new URL(name, dir))
  } catch {
    return null
  }
}

/** Reads an embedded payload file as UTF-8 text, or `null`. */
export function readEmbeddedText(name: string): string | null {
  const bytes = readEmbeddedBytes(name)
  return bytes === null ? null : new TextDecoder().decode(bytes)
}

/** JSON merge patch (like the CLI's config merging): objects merge
 * recursively, null removes the key, everything else replaces. */
export function mergeConfig(
  target: Record<string, Json>,
  source: Record<string, Json>
): Record<string, Json> {
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

// Capability files carry these extensions; a `schemas/` subfolder holds JSON
// schema files, not capabilities, so it is skipped (mirrors tauri-build).
const CAPABILITY_EXTENSIONS = ['.json', '.json5', '.toml']

function walkCapabilityFiles(dir: string, files: string[] = []): string[] {
  for (const entry of Deno.readDirSync(dir)) {
    const full = `${dir}/${entry.name}`
    if (entry.isDirectory) {
      if (entry.name !== 'schemas') walkCapabilityFiles(full, files)
    } else if (
      CAPABILITY_EXTENSIONS.some((ext) =>
        entry.name.toLowerCase().endsWith(ext)
      )
    ) {
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
export function resolveCapabilities(dirs: (string | null)[]): string[] {
  for (const base of dirs) {
    if (!base) continue
    const capDir = `${base}/capabilities`
    try {
      if (!Deno.statSync(capDir).isDirectory) continue
    } catch {
      continue
    }
    return walkCapabilityFiles(capDir)
      .sort()
      .map((file) => Deno.readTextFileSync(file))
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
  const resolved = dist.startsWith('/')
    ? dist
    : `${base}/${dist.replace(/^\.\//, '')}`
  return dist.endsWith('.assets') ? { archive: resolved } : { dir: resolved }
}
