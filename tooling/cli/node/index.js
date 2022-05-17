const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      return readFileSync('/usr/bin/ldd', 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

function getNativeBinding(platform, arch) {
  let platform_arch = `${platform}-${arch}`
  const supported = ["android-arm64", "android-arm", "win32-x64", "win32-ia32", "win32-arm64",
    "darwin-x64", "darwin-arm64", "freebsd-x64", "linux-x64", "linux-arm64", "linux-arm", ]
  if (!supported.includes(platform_arch)) {
    throw new Error(`Unsupported architecture: ${platform_arch}`)
  }

  if (platform == "win32") platform_arch += "-msvc";
  if (platform == "linux") platform_arch += (arch == "arm") ? "-gnueabihf" : isMusl() ? "-musl" : "-gnu";

  const localName = `cli.${platform_arch}.node`
  const remoteName = `@tauri-apps/cli-${platform_arch}`

  let nativeBinding = null, loadError = null
  let localFileExisted = existsSync(join(__dirname, localName))
  try {
    if (localFileExisted) {
      nativeBinding = require(`./${localName}`)
    } else {
      nativeBinding = require(remoteName)
    }
  } catch (e) {
    loadError = e
  }

  return {
    nativeBinding: nativeBinding,
    loadError: loadError
  }
}


let nb = getNativeBinding(platform, arch)

if (!nb.nativeBinding) {
  if (nb.loadError) {
    throw nb.loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { run } = nb.nativeBinding

module.exports.run = run
