#!/usr/bin/env node

const cmds = ['init', 'help', 'icon', 'info', 'deps']
const rustCliCmds = ['dev', 'build']
const fs = require('fs');
const cmd = process.argv[2]
const chalk = require('chalk');
const package = require('../package.json');
const figlet = require('figlet'); // ascii art 
/**
 * @description This is the bootstrapper that in turn calls subsequent
 * Tauri Commands
 *
 * @param {string|array} command
 */
const tauri = function (command) {
  if (typeof command === 'object') {
    // technically we just care about an array
    command = command[0]
  }

  if (rustCliCmds.includes(command)) {
    const { runOnRustCli } = require('../dist/helpers/rust-cli')
    if (process.argv && !process.env.test) {
      process.argv.splice(0, 3)
    }
    runOnRustCli(command, process.argv || [])
    return
  }

  if (
    !command ||
    command === '-h' ||
    command === '--help' ||
    command === 'help'
  ) {
    
    console.log(chalk.cyan(figlet.textSync('Tauri')));
    console.log(chalk.green('Description'))
    console.log('This is the Tauri CLI');
    console.log(chalk.magenta('Usage'));
    console.log('$ tauri ${cmds.join('|')}')
    console.log(chalk.blue('Options'))
    console.log('--help, -h     Displays this message \n  --version, -v  Displays the Tauri CLI version');
    process.exit(0)
    // eslint-disable-next-line no-unreachable
    return false // do this for node consumers and tests
  }

  if (command === '-v' || command === '--version') {
    console.log(chalk.green(`${package.version}`));
    // also added colors to beautify the cli
    return false // do this for node consumers and tests
  }
  if (command ==='-no-update-notifier || command === '--no-update-notifier'){
      const noUpdates = true;
      }

  if (cmds.includes(command)) {
    if (process.argv && !process.env.test) {
      process.argv.splice(2, 1)
    }
    console.log(`[tauri]: running ${command}`)
    // eslint-disable-next-line security/detect-non-literal-require
    if (['init'].includes(command)) {
      require(`./tauri-${command}`)(process.argv.slice(2))
    } else {
      require(`./tauri-${command}`)
    }
  } else {
    console.log(`Invalid command ${command}. Use one of ${cmds.join(', ')}.`)
  }
}
// notifying updates.
const pkg = JSON.parse(fs.readFileSync(__dirname + '/../package.json')); 
if (pkg.version.indexOf('0.0.0') !== 0 && noUpdates !== true) {
  require('update-notifier')({ pkg }).notify();
}

module.exports = {
  tauri
}

tauri(cmd)
