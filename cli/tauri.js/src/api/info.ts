const os = require('os')
const spawn = require('cross-spawn').sync
const chalk = require('chalk')
const path = require('path')
const fs = require('fs')

interface DirInfo {
  path: string
  name: string
  type?: 'folder'|'file'
  children?: DirInfo[]
}

function dirTree(filename: string): DirInfo {
  const stats = fs.lstatSync(filename)
  const info: DirInfo = {
    path: filename,
    name: path.basename(filename)
  }

  if (stats.isDirectory()) {
    info.type = 'folder'
    info.children = fs.readdirSync(filename).map(function (child: string) {
      return dirTree(filename + '/' + child)
    });
  } else {
    info.type = 'file'
  }

  return info
}

function getVersion (command: string, args: string[] = [], formatter?: (output: string) => string) {
  try {
    const child = spawn(command, [...args, '--version'])
    if (child.status === 0) {
      const output = String(child.output[1])
      return chalk.green(formatter === undefined ? output : formatter(output)).replace('\n', '')
    }
    return chalk.red('Not installed')
  }
  catch (err) {
    return chalk.red('Not installed')
  }
}

interface Info {
  section?: boolean
  key: string
  value?: string
}

function printInfo (info: Info) {
  console.log(`${info.section ? '\n' : ''}${info.key}${info.value === undefined ? '' : ' - ' + info.value}`)
}

module.exports = () => {
  printInfo({ key: 'Operating System', value: chalk.green(`${os.type()}(${os.release()}) - ${os.platform()}/${os.arch()}`), section: true })
  printInfo({ key: 'Node.js environment', section: true })
  printInfo({ key: '  Node.js', value: chalk.green(process.version.slice(1)) })
  printInfo({ key: '  tauri.js', value: chalk.green(require('../../package.json').version) })
  printInfo({ key: 'Rust environment', section: true })
  printInfo({ key: '  Rust', value: getVersion('rustc', [], output => output.split(' ')[1]) })
  printInfo({ key: '  tauri-cli', value: getVersion('cargo', ['tauri-cli']) })
  printInfo({ key: 'Global packages', section: true })
  printInfo({ key: '  NPM', value: getVersion('npm') })
  printInfo({ key: '  yarn', value: getVersion('yarn') })
  printInfo({ key: 'App directory structure', section: true })

  const { appDir } = require('../helpers/app-paths')
  const tree = dirTree(appDir)
  for (const artifact of (tree.children || [])) {
    if (artifact.type === 'folder') {
      console.log(`/${artifact.name}`)
    }
  }
}
