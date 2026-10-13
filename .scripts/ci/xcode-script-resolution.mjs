#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Checks that `tauri ios xcode-script` finds the Tauri project for every package manager, project layout
// and way Xcode can be started.
//
// Xcode runs the command the iOS project template generates (`pnpm tauri ios xcode-script ...`, see
// crates/tauri-cli/src/mobile/init.rs) from `<tauri dir>/gen/apple`. Whether the package manager then moves
// to the package root, and which environment variables it sets, depends on the package manager and its
// version, and Xcode inherits the environment of the `tauri ios dev` process that opened it (`--open`) or
// spawned xcodebuild. This script reproduces each combination without Xcode: no dev server is running, so
// the CLI fails reading `<resolved tauri dir>/gen/apple/.tauri/cli-options-server.json`, and that path tells
// which directory it resolved.
//
// The CLI runs in-process through each package manager as with a real install: the fixtures depend on a
// copy of the `@tauri-apps/cli` JS wrapper whose native module is loaded from `--cli-node`.
//
// Usage: node .scripts/ci/xcode-script-resolution.mjs [--cli-node <cli.node>] [--cli-bin <cargo-tauri>]
//   [--pm <regex>] [--latest] [--keep]
// Defaults to the `pnpm build:cli:debug` and `cargo build -p tauri-cli` outputs. `--latest` tests the
// latest release of each package manager instead of the pinned versions.

import { spawn, execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { parseArgs } from 'node:util'

const ROOT = path.resolve(import.meta.dirname, '..', '..')

const { values: opts } = parseArgs({
  options: {
    'cli-node': { type: 'string' },
    'cli-bin': {
      type: 'string',
      default: path.join(ROOT, 'target', 'debug', 'cargo-tauri')
    },
    pm: { type: 'string' },
    latest: { type: 'boolean', default: false },
    keep: { type: 'boolean', default: false }
  }
})

const CLI_NODE = path.resolve(
  opts['cli-node']
    ?? path.join(
      ROOT,
      'packages',
      'cli',
      `cli.${process.platform}-${process.arch}.node`
    )
)
const CLI_BIN = path.resolve(opts['cli-bin'])
for (const f of [CLI_NODE, CLI_BIN]) {
  if (!fs.existsSync(f)) {
    console.error(
      `${f} not found, build the CLI first (pnpm build:cli:debug && cargo build -p tauri-cli)`
    )
    process.exit(1)
  }
}

const WORK = fs.realpathSync(
  fs.mkdtempSync(path.join(os.tmpdir(), 'tauri-xcode-script-'))
)
const TOOLS = path.join(WORK, 'tools')
const FIXTURES = path.join(WORK, 'fixtures')
const FAKE_CLI = path.join(WORK, 'cli')
const BIN = path.join(WORK, 'bin')

// `version` is installed from npm (`pkg@version`), `--latest` swaps it for `latest`
const PMS = [
  { id: 'npm-10', kind: 'npm', pkg: 'npm', version: '10' },
  { id: 'npm-11', kind: 'npm', pkg: 'npm', version: '11' },
  { id: 'npx-11', kind: 'npx', pkg: 'npm', version: '11' },
  { id: 'pnpm-9', kind: 'pnpm', pkg: 'pnpm', version: '9' },
  { id: 'pnpm-10', kind: 'pnpm', pkg: 'pnpm', version: '10' },
  // runs the bin in place, without lifecycle env vars, when called outside a package
  { id: 'pnpm-12.3', kind: 'pnpm', pkg: 'pnpm', version: '12.3' },
  { id: 'pnpm-12', kind: 'pnpm', pkg: 'pnpm', version: '12' },
  { id: 'yarn-1', kind: 'yarn1', pkg: 'yarn', version: '1' },
  {
    id: 'yarn-4-node-modules',
    kind: 'yarn4',
    linker: 'node-modules',
    pkg: '@yarnpkg/cli-dist',
    version: '4'
  },
  {
    id: 'yarn-4-pnp',
    kind: 'yarn4',
    linker: 'pnp',
    pkg: '@yarnpkg/cli-dist',
    version: '4'
  },
  // bun<1.2 does not move to the package root
  { id: 'bun-1.1', kind: 'bun', pkg: 'bun', version: '1.1' },
  { id: 'bun-1', kind: 'bun', pkg: 'bun', version: '1' },
  // uses the `deno` on PATH; dependencies are installed with npm
  { id: 'deno', kind: 'deno', pkg: 'npm', version: '11' },
  { id: 'cargo', kind: 'cargo' }
]

// `pkg` = dir with the package.json owning the `tauri` script, `tauri` = Tauri dir, `ws` = workspace root.
// `decoys` are other Tauri apps sorting before the tested one, to catch a lookup from the workspace root.
const SCENARIOS = [
  { id: 'standalone', pkg: '.', tauri: 'src-tauri' },
  { id: 'custom-tauri-dir', pkg: '.', tauri: 'desktop' },
  { id: 'tauri-dir-is-pkg-root', pkg: '.', tauri: '.' },
  {
    id: 'workspace-member',
    ws: '.',
    pkg: 'apps/app',
    tauri: 'apps/app/src-tauri',
    decoys: ['apps/aaa-other']
  },
  // like tauri-apps/plugins-workspace: the workspace root also depends on the CLI
  {
    id: 'workspace-member-root-cli',
    ws: '.',
    pkg: 'apps/app',
    tauri: 'apps/app/src-tauri',
    rootCli: true,
    decoys: ['apps/aaa-other']
  },
  {
    id: 'workspace-member-nested',
    ws: '.',
    pkg: 'packages/desktop/app',
    tauri: 'packages/desktop/app/src-tauri',
    decoys: ['packages/desktop/aaa-other']
  },
  // the CLI and `tauri` script live in the workspace root, the app member only has a frontend package.json
  {
    id: 'workspace-root-owns-cli',
    ws: '.',
    pkg: '.',
    member: 'apps/app',
    tauri: 'apps/app/src-tauri'
  },
  { id: 'no-package-json', pkg: null, tauri: 'src-tauri', cargoOnly: true }
]

// where `tauri ios dev` was started, which decides the environment Xcode inherits
const MODES = [
  { id: 'clean' }, // Xcode opened on its own
  { id: 'dev@pkg', start: (s) => s.pkg },
  { id: 'dev@tauri', start: (s) => s.tauri },
  { id: 'dev@ws-filter', start: (s) => s.ws, filter: true } // e.g. `pnpm --filter app tauri ios dev`
]

// Combinations where the package manager itself cannot start the CLI. These are allowed to fail before the
// CLI runs (not required to, so newer package manager releases may fix them); a CLI that runs but does
// not find the right Tauri dir is always a failure.
const PM_LIMITATIONS = [
  {
    pm: /^pnpm-12\.3$/,
    mode: 'clean',
    why: 'pnpm@12.3 does not look up bins outside a package'
  },
  {
    pm: /^(npm|yarn-4)/,
    scenario: 'workspace-root-owns-cli',
    why: 'the member package.json has no `tauri` script'
  }
]

const XCODE_ARGS = [
  'ios',
  'xcode-script',
  '--platform',
  'iOS Simulator',
  '--sdk-root',
  '/sdk',
  '--framework-search-paths',
  'x',
  '--header-search-paths',
  'x',
  '--gcc-preprocessor-definitions',
  'DEBUG=1',
  '--configuration',
  'debug',
  'arm64'
]

// matches the `tauri-binary` + `tauri-binary-args` the iOS project template is generated with (init.rs)
function invocation(pm, args, { filter, s } = {}) {
  switch (pm.kind) {
    case 'npm':
      return filter
        ? ['npm', 'run', 'tauri', '-w', s.pkg, '--', ...args]
        : ['npm', 'run', '--', 'tauri', ...args]
    case 'npx':
      return filter ? null : ['npx', 'tauri', ...args]
    case 'pnpm':
      return filter
        ? ['pnpm', '--filter', 'app', 'tauri', ...args]
        : ['pnpm', 'tauri', ...args]
    case 'yarn1':
    case 'yarn4':
      return filter
        ? ['yarn', 'workspace', 'app', 'tauri', ...args]
        : ['yarn', 'tauri', ...args]
    case 'bun':
      return filter
        ? ['bun', '--filter', 'app', 'tauri', ...args]
        : ['bun', 'tauri', ...args]
    case 'deno':
      return filter ? null : ['deno', 'task', 'tauri', ...args]
    case 'cargo':
      return filter ? null : ['cargo', 'tauri', ...args]
  }
}

const which = (bin) => {
  try {
    return execFileSync('/usr/bin/which', [bin]).toString().trim()
  } catch {
    return null
  }
}

function run(cmd, args, { cwd, env, timeout = 120000 }) {
  return new Promise((resolve) => {
    const child = spawn(cmd, args, {
      cwd,
      env,
      stdio: ['ignore', 'pipe', 'pipe']
    })
    let out = ''
    child.stdout.on('data', (d) => (out += d))
    child.stderr.on('data', (d) => (out += d))
    const timer = setTimeout(() => child.kill('SIGKILL'), timeout)
    child.on('error', (e) => {
      clearTimeout(timer)
      resolve({ code: -1, out: out + String(e) })
    })
    child.on('close', (code) => {
      clearTimeout(timer)
      resolve({ code, out })
    })
  })
}

const write = (p, content) => {
  fs.mkdirSync(path.dirname(p), { recursive: true })
  fs.writeFileSync(
    p,
    typeof content === 'string' ? content : JSON.stringify(content, null, 2)
  )
}

function setupCli() {
  for (const f of ['tauri.js', 'main.js']) {
    fs.cpSync(path.join(ROOT, 'packages', 'cli', f), path.join(FAKE_CLI, f))
  }
  write(
    path.join(FAKE_CLI, 'index.js'),
    [
      '// dumps the environment `tauri ios dev` would run with, or loads the native CLI under test',
      'if (process.env.TAURI_XCODE_SCRIPT_TEST_DUMP) {',
      "  require('fs').writeFileSync(process.env.TAURI_XCODE_SCRIPT_TEST_DUMP, JSON.stringify({ cwd: process.cwd(), env: process.env }))",
      '  process.exit(0)',
      '}',
      `module.exports = require(${JSON.stringify(CLI_NODE)})`,
      ''
    ].join('\n')
  )
  write(path.join(FAKE_CLI, 'package.json'), {
    name: '@tauri-apps/cli',
    version: '0.0.0',
    type: 'commonjs',
    main: 'main.js',
    bin: { tauri: 'tauri.js' }
  })
  fs.mkdirSync(BIN, { recursive: true })
  fs.symlinkSync(CLI_BIN, path.join(BIN, 'cargo-tauri'))
}

// installs the package manager from npm and exposes its binaries in a PATH dir of its own
async function setupPm(pm) {
  pm.path = path.join(TOOLS, pm.id, 'path')
  fs.mkdirSync(pm.path, { recursive: true })
  if (pm.kind === 'deno' && !which('deno')) return 'deno is not installed'
  if (!pm.pkg) return null
  const dir = path.join(TOOLS, pm.id)
  const spec = `${pm.pkg}@${opts.latest ? 'latest' : pm.version}`
  const res = await run(
    'npm',
    ['install', '--prefix', dir, '--no-audit', '--no-fund', '--no-save', spec],
    { cwd: WORK, env: process.env, timeout: 300000 }
  )
  if (res.code !== 0) return `failed to install ${spec}:\n${res.out}`
  const bins = path.join(dir, 'node_modules', '.bin')
  for (const bin of fs.readdirSync(bins))
    fs.symlinkSync(path.join(bins, bin), path.join(pm.path, bin))
  const versionBin = {
    npm: 'npm',
    npx: 'npm',
    deno: 'npm',
    pnpm: 'pnpm',
    yarn1: 'yarn',
    yarn4: 'yarn',
    bun: 'bun'
  }
  pm.resolved = (
    await run(path.join(bins, versionBin[pm.kind]), ['--version'], {
      cwd: WORK,
      env: process.env
    })
  ).out.trim()
  return null
}

function env(pm, extra = {}) {
  const system = ['node', 'deno', 'cargo']
    .map(which)
    .filter(Boolean)
    .map((p) => path.dirname(p))
  const dirs = [
    pm.path,
    BIN,
    ...system,
    '/usr/bin',
    '/bin',
    '/usr/sbin',
    '/sbin'
  ]
  return {
    HOME: process.env.HOME,
    USER: process.env.USER,
    TMPDIR: process.env.TMPDIR,
    LANG: 'en_US.UTF-8',
    PATH: [...new Set(dirs)].join(':'),
    RUSTUP_HOME:
      process.env.RUSTUP_HOME ?? path.join(process.env.HOME, '.rustup'),
    CARGO_HOME: process.env.CARGO_HOME ?? path.join(process.env.HOME, '.cargo'),
    RUSTUP_TOOLCHAIN: process.env.RUSTUP_TOOLCHAIN,
    COREPACK_ENABLE_STRICT: '0',
    YARN_ENABLE_IMMUTABLE_INSTALLS: 'false',
    YARN_ENABLE_TELEMETRY: '0',
    YARN_ENABLE_GLOBAL_CACHE: 'false',
    NO_COLOR: '1',
    ...extra
  }
}

function writeTauriDir(dir, name) {
  write(path.join(dir, 'tauri.conf.json'), {
    productName: name,
    version: '0.1.0',
    identifier: `com.tauri.${name}`,
    build: { frontendDist: [] },
    app: { windows: [] }
  })
  write(path.join(dir, 'gen', 'apple', 'project.yml'), `name: ${name}\n`)
}

function packageJson(pm, name, { cli = true, workspaces } = {}) {
  const pkg = { name, version: '0.0.0', private: true }
  if (cli) {
    // deno users run the npm package through deno itself
    pkg.scripts = {
      tauri: pm.kind === 'deno' ? 'deno run -A npm:@tauri-apps/cli' : 'tauri'
    }
    pkg.devDependencies = { '@tauri-apps/cli': `file:${FAKE_CLI}` }
  }
  if (workspaces && pm.kind !== 'pnpm') pkg.workspaces = workspaces
  // pnpm@>=10 and yarn@>=2 switch to the version in `packageManager`, so pin the one under test
  if (pm.kind === 'pnpm') pkg.packageManager = `pnpm@${pm.resolved}`
  if (pm.kind.startsWith('yarn')) pkg.packageManager = `yarn@${pm.resolved}`
  return pkg
}

function createFixture(pm, s) {
  const root = path.join(FIXTURES, pm.id, s.id)
  const at = (p) => path.join(root, p)
  fs.mkdirSync(root, { recursive: true })

  if (s.ws) {
    const members = [s.pkg, s.member, ...(s.decoys ?? [])].filter(
      (p) => p && p !== '.'
    )
    const globs = [...new Set(members.map((p) => `${path.dirname(p)}/*`))]
    const wsPkg = packageJson(pm, 'workspace', {
      cli: s.pkg === '.',
      workspaces: globs
    })
    if (s.rootCli)
      wsPkg.devDependencies = { '@tauri-apps/cli': `file:${FAKE_CLI}` }
    write(at('package.json'), wsPkg)
    if (pm.kind === 'pnpm') {
      write(
        at('pnpm-workspace.yaml'),
        `packages:\n${globs.map((g) => `  - '${g}'`).join('\n')}\n`
      )
    }
    if (s.pkg !== '.')
      write(at(path.join(s.pkg, 'package.json')), packageJson(pm, 'app'))
    if (s.member)
      write(
        at(path.join(s.member, 'package.json')),
        packageJson(pm, 'app', { cli: false })
      )
    for (const decoy of s.decoys ?? []) {
      write(
        at(path.join(decoy, 'package.json')),
        packageJson(pm, path.basename(decoy))
      )
      writeTauriDir(at(path.join(decoy, 'src-tauri')), 'decoy')
    }
  } else if (s.pkg) {
    write(at(path.join(s.pkg, 'package.json')), packageJson(pm, 'app'))
  }
  writeTauriDir(at(s.tauri), 'app')
  if (pm.kind === 'yarn4')
    write(
      at('.yarnrc.yml'),
      `nodeLinker: ${pm.linker}\nenableGlobalCache: false\n`
    )
  if (pm.kind === 'yarn1') write(at('yarn.lock'), '')
  return root
}

async function install(pm, s, root) {
  if (pm.kind === 'cargo' || !s.pkg) return null
  const cmd = {
    npm: ['npm', 'install', '--no-audit', '--no-fund'],
    npx: ['npm', 'install', '--no-audit', '--no-fund'],
    deno: ['npm', 'install', '--no-audit', '--no-fund'],
    pnpm: ['pnpm', 'install'],
    yarn1: ['yarn', 'install', '--no-progress', '--non-interactive'],
    yarn4: ['yarn', 'install'],
    bun: ['bun', 'install']
  }[pm.kind]
  const res = await run(cmd[0], cmd.slice(1), {
    cwd: root,
    env: env(pm),
    timeout: 300000
  })
  return res.code === 0 ? null : `\`${cmd.join(' ')}\` failed:\n${res.out}`
}

// runs `tauri ios dev` with the dump hook to get the environment Xcode would inherit
async function captureDevEnv(pm, s, root, mode) {
  const inv = invocation(pm, ['ios', 'dev', '--open'], {
    filter: mode.filter,
    s
  })
  if (!inv) return { skip: 'no such command' }
  const dump = path.join(root, `.env-${mode.id}.json`)
  await run(inv[0], inv.slice(1), {
    cwd: path.join(root, mode.start(s)),
    env: env(pm, { TAURI_XCODE_SCRIPT_TEST_DUMP: dump })
  })
  if (!fs.existsSync(dump))
    return { skip: `\`${inv.join(' ')}\` does not run from ${mode.start(s)}` }
  const captured = JSON.parse(fs.readFileSync(dump, 'utf8'))
  delete captured.env.TAURI_XCODE_SCRIPT_TEST_DUMP
  return captured
}

async function runXcodeScript(pm, s, root, devEnv) {
  const inv = invocation(pm, XCODE_ARGS)
  const res = await run(inv[0], inv.slice(1), {
    cwd: path.join(root, s.tauri, 'gen', 'apple'),
    env: devEnv ? { ...devEnv, PATH: `${BIN}:${devEnv.PATH}` } : env(pm)
  })
  const tail = res.out.trim().split('\n').slice(-8).join('\n')
  const resolved = res.out.match(
    /failed to read (\S+?)\/gen\/apple\/\.tauri\/cli-options-server\.json/
  )
  if (resolved) {
    const expected = fs.realpathSync(path.join(root, s.tauri))
    const got = fs.realpathSync(resolved[1])
    return got === expected
      ? { status: 'pass' }
      : {
          status: 'wrong-dir',
          detail: `resolved ${path.relative(root, got) || '.'}`
        }
  }
  if (/Couldn't recognize the .* folder as a Tauri project/.test(res.out)) {
    return { status: 'not-found', detail: tail }
  }
  return { status: 'cli-not-reached', detail: tail }
}

async function pool(items, size, fn) {
  let next = 0
  await Promise.all(
    Array.from({ length: size }, async () => {
      while (next < items.length) await fn(items[next++])
    })
  )
}

setupCli()
const pmFilter = opts.pm ? new RegExp(opts.pm) : null
const pms = PMS.filter((pm) => !pmFilter || pmFilter.test(pm.id))
const rows = []
const setupErrors = []

await pool(pms, 4, async (pm) => {
  const error = await setupPm(pm)
  if (error) {
    setupErrors.push(`${pm.id}: ${error}`)
    return
  }
  for (const s of SCENARIOS) {
    if (s.cargoOnly && pm.kind !== 'cargo') continue
    const root = createFixture(pm, s)
    const installError = await install(pm, s, root)
    if (installError) {
      setupErrors.push(`${pm.id} ${s.id}: ${installError}`)
      continue
    }
    for (const mode of MODES) {
      if (mode.filter && !s.ws) continue
      // cargo leaves no package manager environment behind, so it is the same as `clean`
      if (mode.start && pm.kind === 'cargo') continue
      if (mode.id === 'dev@tauri' && s.tauri === s.pkg) continue
      let devEnv = null
      if (mode.start) {
        const captured = await captureDevEnv(pm, s, root, mode)
        if (captured.skip) {
          rows.push({ pm, s, mode, status: 'n/a', detail: captured.skip })
          continue
        }
        devEnv = captured.env
      }
      rows.push({ pm, s, mode, ...(await runXcodeScript(pm, s, root, devEnv)) })
    }
  }
  console.log(`${pm.id} (${pm.resolved ?? pm.kind}) done`)
})

const limitation = (r) =>
  PM_LIMITATIONS.find(
    (l) =>
      l.pm.test(r.pm.id)
      && (!l.scenario || l.scenario === r.s.id)
      && (!l.mode || l.mode === r.mode.id)
  )
const failures = rows.filter(
  (r) =>
    ['wrong-dir', 'not-found'].includes(r.status)
    || (r.status === 'cli-not-reached' && !limitation(r))
)

const order = (list, item) => list.indexOf(item)
rows.sort(
  (a, b) =>
    order(PMS, a.pm) - order(PMS, b.pm)
    || order(SCENARIOS, a.s) - order(SCENARIOS, b.s)
    || order(MODES, a.mode) - order(MODES, b.mode)
)

const counts = {}
for (const r of rows) counts[r.status] = (counts[r.status] ?? 0) + 1
console.log(`\n${rows.length} cases: ${JSON.stringify(counts)}`)
for (const r of rows.filter(
  (r) => r.status === 'cli-not-reached' && limitation(r)
)) {
  console.log(
    `known limitation: ${r.pm.id} ${r.s.id} ${r.mode.id}: ${limitation(r).why}`
  )
}
for (const r of failures) {
  console.log(
    `\nFAIL ${r.pm.id} (${r.pm.resolved ?? r.pm.kind}) ${r.s.id} ${r.mode.id}: ${r.status}`
  )
  if (r.detail) console.log(`  ${r.detail.replace(/\n/g, '\n  ')}`)
}
for (const e of setupErrors) console.log(`\nSETUP ERROR ${e}`)

if (process.env.GITHUB_STEP_SUMMARY) {
  const icon = (r) =>
    r.status === 'pass'
      ? '✅'
      : r.status === 'n/a'
        ? '➖'
        : r.status === 'cli-not-reached' && limitation(r)
          ? '⚪'
          : '❌'
  const cols = MODES.map((m) => m.id)
  let md = `### \`tauri ios xcode-script\` project resolution\n\n${rows.length} cases, ${failures.length} failures`
  md +=
    ' (✅ pass, ❌ fail, ⚪ known package manager limitation, ➖ not applicable)\n\n'
  md += `| package manager | layout | ${cols.join(' | ')} |\n|---|---|${cols.map(() => '---').join('|')}|\n`
  for (const pm of pms) {
    for (const s of SCENARIOS) {
      const cells = cols.map((c) => {
        const r = rows.find((r) => r.pm === pm && r.s === s && r.mode.id === c)
        return r ? icon(r) : ''
      })
      if (cells.some(Boolean))
        md += `| ${pm.id} (${pm.resolved ?? pm.kind}) | ${s.id} | ${cells.join(' | ')} |\n`
    }
  }
  fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, md)
}

if (opts.keep) console.log(`\nfixtures kept in ${WORK}`)
else fs.rmSync(WORK, { recursive: true, force: true })
process.exit(failures.length || setupErrors.length ? 1 : 0)
