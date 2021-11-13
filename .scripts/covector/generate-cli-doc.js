const childProcess = require('child_process')
const path = require('path')
const fs = require('fs')
const rustCliPath = path.join(__dirname, '../../tooling/cli.rs/target/debug/cargo-tauri')
const templatePath = path.join(__dirname, '../../docs/.templates/cli.md')
const targetPath = path.join(__dirname, '../../docs/api/cli.md')
const template = fs.readFileSync(templatePath, 'utf8')

const commands = ['info', 'init', 'plugin init', 'dev', 'build']

let doc = template

for (const cmd of commands) {
  const output = childProcess.execSync(`${rustCliPath} ${cmd} --help`).toString().split('\n')
  output.splice(0, 2)
  output.splice(-1)
  doc = doc.replace(`{${cmd}}`, '```\n' + output.join('\n') + '\n```')
}

fs.writeFileSync(targetPath, doc)
