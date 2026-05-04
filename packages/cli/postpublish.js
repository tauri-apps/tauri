// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const { execFileSync } = require('node:child_process')
const { readdirSync } = require('node:fs')
const { join } = require('node:path')

function run(command, args, cwd = process.cwd()) {
  execFileSync(command, args, {
    cwd,
    stdio: 'inherit',
    env: process.env
  })
}

const cliDir = process.cwd()
const npmDir = join(cliDir, 'npm')
const publishTag = process.env.npm_config_tag || 'latest'

console.log(
  `Publishing platform npm packages from postpublish hook using tag "${publishTag}"...`
)

for (const entry of readdirSync(npmDir, { withFileTypes: true })) {
  if (!entry.isDirectory()) {
    continue
  }

  const pkgDir = join(npmDir, entry.name)
  run('npm', ['publish', '--tag', publishTag, '--ignore-scripts'], pkgDir)
}
