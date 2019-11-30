const logger = require('./logger')
const log = logger('app:spawn')
const warn = logger('app:spawn', 'red')
const crossSpawn = require('cross-spawn')

/*
 Returns pid, takes onClose
 */
module.exports.spawn = function (cmd, params, cwd, onClose) {
  log(`Running "${cmd} ${params.join(' ')}"`)
  log()

  const runner = crossSpawn(
    cmd,
    params, {
      stdio: 'inherit',
      stdout: 'inherit',
      stderr: 'inherit',
      cwd
    }
  )

  runner.on('close', code => {
    log()
    if (code) {
      log(`Command "${cmd}" failed with exit code: ${code}`)
    }

    onClose && onClose(code)
  })

  return runner.pid
}

/*
 Returns nothing, takes onFail
 */
module.exports.spawnSync = function (cmd, params, cwd, onFail) {
  log(`[sync] Running "${cmd} ${params.join(' ')}"`)
  log()

  const runner = crossSpawn.sync(
    cmd,
    params, {
      stdio: 'inherit',
      stdout: 'inherit',
      stderr: 'inherit',
      cwd
    }
  )

  if (runner.status || runner.error) {
    warn()
    warn(`⚠️  Command "${cmd}" failed with exit code: ${runner.status}`)
    if (runner.status === null) {
      warn(`⚠️  Please globally install "${cmd}"`)
    }
    onFail && onFail()
    process.exit(1)
  }
}
