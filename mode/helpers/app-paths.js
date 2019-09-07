const
  { existsSync } = require('fs'),
  { resolve, join, normalize, sep } = require('path')

/**
 *
 * @returns {{length}|*}
 */
function getAppDir() {
  let dir = process.cwd()
  let count = 0

  // only go up three folders max
  while (dir.length && dir[dir.length - 1] !== sep && count <= 2) {
    if (existsSync(join(dir, 'tauri.conf.js'))) {
      return dir
    }
    count++
    dir = normalize(join(dir, '..'))
  }

  // just return the current directory
  return process.cwd()
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
