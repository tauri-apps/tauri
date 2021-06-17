/**
 * This script is used to rename the binary with the platform specific postfix.
 * When `tauri build` is ran, it looks for the binary name appended with the platform specific postfix.
 */

const execa = require('execa')
const fs = require('fs')

async function main() {
  const rustTargetInfo = JSON.parse(
    (
      await execa(
        'rustc',
        ['-Z', 'unstable-options', '--print', 'target-spec-json'],
        {
          env: {
            RUSTC_BOOTSTRAP: 1
          }
        }
      )
    ).stdout
  )
  const platformPostfix = rustTargetInfo['llvm-target']
  fs.renameSync(
    'src-tauri/binaries/app',
    `src-tauri/binaries/app-${platformPostfix}`
  )
}

main().catch((e) => {
  throw e
})
