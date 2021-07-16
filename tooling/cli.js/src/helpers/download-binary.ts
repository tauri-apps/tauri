import stream from 'stream'
import { promisify } from 'util'
import fs from 'fs'
import got from 'got'
import { CargoManifest } from '../types/cargo'
import path from 'path'
const pipeline = promisify(stream.pipeline)

// Webpack reads the file at build-time, so this becomes a static var
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
const tauriCliManifest = require('../../../cli.rs/Cargo.toml') as CargoManifest

const downloads: { [url: string]: boolean } = {}

async function downloadBinaryRelease(
  tag: string,
  asset: string,
  outPath: string
): Promise<void> {
  const url = `https://github.com/tauri-apps/binary-releases/releases/download/${tag}/${asset}`

  const removeDownloadedCliIfNeeded = (): void => {
    try {
      if (!(url in downloads)) {
        // eslint-disable-next-line security/detect-non-literal-fs-filename
        fs.unlinkSync(outPath)
      }
    } finally {
      process.exit()
    }
  }

  // on exit, we remove the `tauri-cli` file if the download didn't complete
  process.on('exit', removeDownloadedCliIfNeeded)
  process.on('SIGINT', removeDownloadedCliIfNeeded)
  process.on('SIGTERM', removeDownloadedCliIfNeeded)
  process.on('SIGHUP', removeDownloadedCliIfNeeded)
  process.on('SIGBREAK', removeDownloadedCliIfNeeded)

  // TODO: Check hash of download
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, security/detect-non-literal-fs-filename
  await pipeline(got.stream(url), fs.createWriteStream(outPath)).catch((e) => {
    try {
      // eslint-disable-next-line security/detect-non-literal-fs-filename
      fs.unlinkSync(outPath)
    } catch {}
    throw e
  })
  // eslint-disable-next-line security/detect-object-injection
  downloads[url] = true
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  fs.chmodSync(outPath, 0o700)
  console.log('Download Complete')
}

async function downloadCli(): Promise<void> {
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
  const extension = platform === 'windows' ? '.exe' : ''
  const outPath = path.join(__dirname, `../../bin/tauri-cli${extension}`)
  console.log('Downloading Rust CLI...')
  await downloadBinaryRelease(
    `tauri-cli-v${version}`,
    `tauri-cli_${platform}${extension}`,
    outPath
  )
}

async function downloadRustup(): Promise<void> {
  const assetName =
    process.platform === 'win32' ? 'rustup-init.exe' : 'rustup-init.sh'
  console.log('Downloading Rustup...')
  return await downloadBinaryRelease(
    'rustup',
    assetName,
    path.join(__dirname, `../../bin/${assetName}`)
  )
}

export { downloadCli, downloadRustup }
