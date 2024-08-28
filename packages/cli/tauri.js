#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const cli = require('./main')
const path = require('path')

const [bin, script, ...args] = process.argv
const binStem = path.parse(bin).name.toLowerCase()

// We want to make a helpful binary name for the underlying CLI helper, if we
// can successfully detect what command likely started the execution.
let binName

// deno run -A --unstable --node-modules-dir npm:@tauri-apps/cli
if (bin === '@tauri-apps/cli') {
  binName = '@tauri-apps/cli'
}
// Even if started by a package manager, the binary will be NodeJS.
// Some distribution still use "nodejs" as the binary name.
else if (binStem.match(/(nodejs|node|bun)\-?([0-9]*)*$/g)) {
  const managerStem = process.env.npm_execpath
    ? path.parse(process.env.npm_execpath).name.toLowerCase()
    : null
  if (managerStem) {
    let manager
    switch (managerStem) {
      // Only supported package manager that has a different filename is npm.
      case 'npm-cli':
        manager = 'npm'
        break

      // Yarn, pnpm, and bun have the same stem name as their bin.
      // We assume all unknown package managers do as well.
      default:
        manager = managerStem
        break
    }

    binName = `${manager} run ${process.env.npm_lifecycle_event}`
  } else {
    // Assume running NodeJS if we didn't detect a manager from the env.
    // We normalize the path to prevent the script's absolute path being used.
    const scriptNormal = path.normalize(path.relative(process.cwd(), script))
    binName = `${binStem} ${scriptNormal}`
  }
} else {
  // We don't know what started it, assume it's already stripped.
  args.unshift(bin)
}

cli.run(args, binName).catch((err) => {
  cli.logError(err.message)
  process.exit(1)
})
