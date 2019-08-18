const
  fs = require('fs'),
  path = require('path'),
  resolve = path.resolve,
  join = path.join

function getAppDir() {
  let dir = process.cwd()

  while (dir.length && dir[dir.length - 1] !== path.sep) {
    if (fs.existsSync(join(dir, 'tauri.conf.js'))) {
      return dir
    }

    dir = path.normalize(join(dir, '..'))
  }

  const
    logger = require('./logger')
  warn = logger('app:paths', 'red')

  warn(`⚠️  Error. This command must be executed inside a Tauri project folder.`)
  warn()
  process.exit(1)
}

const appDir = getAppDir(),
  tauriDir = resolve(appDir, 'src-tauri')

module.exports = {
  appDir,
  tauriDir,

  resolve: {
    app: dir => join(appDir, dir),
    tauri: dir => join(tauriDir, dir)
  }
}
