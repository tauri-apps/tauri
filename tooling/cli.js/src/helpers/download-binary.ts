import { promisify } from 'util'
import stream from 'stream'
import fs from 'fs'
import path from 'path'
import { bootstrap } from 'global-agent'
import { fileURLToPath } from 'url'
import { createRequire } from 'module'

const currentDirName = path.dirname(fileURLToPath(import.meta.url))

const require = createRequire(import.meta.url)
/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires */
const got = require('got')
/* eslint-enable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires */

// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
const pipeline = promisify(stream.pipeline)

const downloads: { [url: string]: boolean } = {}

async function downloadBinaryRelease(
  url: string,
  outPath: string
): Promise<void> {
  // const url = `https://github.com/tauri-apps/binary-releases/releases/download/${tag}/${asset}`

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

  bootstrap({
    environmentVariableNamespace: ''
  })

  // TODO: Check hash of download
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, security/detect-non-literal-fs-filename
  await pipeline(got.stream(url), fs.createWriteStream(outPath)).catch(
    (e: unknown) => {
      try {
        // eslint-disable-next-line security/detect-non-literal-fs-filename
        fs.unlinkSync(outPath)
      } catch {}
      throw e
    }
  )
  // eslint-disable-next-line security/detect-object-injection
  downloads[url] = true
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  fs.chmodSync(outPath, 0o700)
  console.log('Download Complete')
}

async function downloadRustup(): Promise<void> {
  let url: string
  if (process.platform === 'win32' && process.arch === 'ia32') {
    url = 'https://win.rustup.rs/i686'
  } else if (process.platform === 'win32' && process.arch === 'x64') {
    url = 'https://win.rustup.rs/x86_64'
  } else {
    url = 'https://sh.rustup.rs'
  }

  const assetName =
    process.platform === 'win32' ? 'rustup-init.exe' : 'rustup-init.sh'

  console.log('Downloading Rustup...')
  return await downloadBinaryRelease(
    url,
    path.join(currentDirName, `../../bin/${assetName}`)
  )
}

export { downloadRustup }
