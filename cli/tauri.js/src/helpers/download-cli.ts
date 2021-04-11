import stream from 'stream'
import { promisify } from 'util'
import fs from 'fs'
import got from 'got'
import { CargoManifest } from '../types/cargo'
import path from 'path'
const pipeline = promisify(stream.pipeline)

// Webpack reads the file at build-time, so this becomes a static var
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
const tauriCliManifest = require('../../../core/Cargo.toml') as CargoManifest

const downloadCli = async (): Promise<void> => {
  const version = tauriCliManifest.package.version
  let platform: string = process.platform
  if (platform === 'win32') {
    platform = 'windows'
  } else if (platform === 'linux') {
    platform = 'linux'
  } else if (platform === 'darwin') {
    platform = 'macos'
  } else {
    throw Error('Unsupported platform')
  }
  const exe = platform === 'windows' ? '.exe' : ''
  const url = `https://github.com/nklayman/tauri-binary-releases/releases/download/tauri-cli-v${version}/tauri-cli_${platform}${exe}`
  const outPath = path.join(__dirname, `../../bin/tauri-cli${exe}`)

  console.log('Downloading Tauri CLI')
  // TODO: Check hash of download
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, security/detect-non-literal-fs-filename
  await pipeline(got.stream(url), fs.createWriteStream(outPath))
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  fs.chmodSync(outPath, 0o700)
  console.log('Download Complete')
}

export { downloadCli }
