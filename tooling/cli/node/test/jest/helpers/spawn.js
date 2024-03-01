// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const crossSpawn = require('cross-spawn')
const logger = require('./logger')

const log = logger('app:spawn')
const warn = logger('app:spawn')

/*
  Returns pid, takes onClose
 */
module.exports.spawn = (
  cmd,
  params,
  cwd,
  onClose
) => {
  log(`Running "${cmd} ${params.join(' ')}"`)
  log()

  // TODO: move to execa?
  const runner = crossSpawn(cmd, params, {
    stdio: 'inherit',
    cwd,
    env: process.env
  })

  runner.on('close', (code) => {
    log()
    if (code) {
      // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
      log(`Command "${cmd}" failed with exit code: ${code}`)
    }

    // eslint-disable-next-line @typescript-eslint/prefer-optional-chain
    onClose && onClose(code || 0, runner.pid || 0)
  })

  return runner.pid || 0
}

/*
  Returns nothing, takes onFail
 */
module.exports.spawnSync = (
  cmd,
  params,
  cwd,
  onFail
) => {
  log(`[sync] Running "${cmd} ${params.join(' ')}"`)
  log()

  const runner = crossSpawn.sync(cmd, params, {
    stdio: 'inherit',
    cwd
  })

  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  if (runner.status || runner.error) {
    warn()
    // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
    warn(`⚠️  Command "${cmd}" failed with exit code: ${runner.status}`)
    if (runner.status === null) {
      warn(`⚠️  Please globally install "${cmd}"`)
    }
    // eslint-disable-next-line @typescript-eslint/prefer-optional-chain
    onFail && onFail()
    process.exit(1)
  }
}
