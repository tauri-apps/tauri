import { existsSync } from 'fs'
import { join, normalize, resolve, sep } from 'path'

const getAppDir = (): string => {
  let dir = process.cwd()
  let count = 0

  // only go up three folders max
  while (dir.length > 0 && dir.endsWith(sep) && count <= 2) {
    if (existsSync(join(dir, 'tauri.conf.json'))) {
      return dir
    }
    count++
    dir = normalize(join(dir, '..'))
  }

  // just return the current directory
  return process.cwd()
}

const appDir = getAppDir()
const tauriDir = resolve(appDir, 'src-tauri')

const resolveDir = {
  app: (dir: string) => resolve(appDir, dir),
  tauri: (dir: string) => resolve(tauriDir, dir)
}

export { appDir, tauriDir, resolveDir as resolve }
