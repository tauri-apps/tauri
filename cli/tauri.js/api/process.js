import tauri from './tauri'

/**
 * spawns a process
 *
 * @param {string} command the name of the cmd to execute e.g. 'mkdir' or 'node'
 * @param {(string[]|string)} args command args
 * @return {Promise<string>} promise resolving to the stdout text
 */
function execute (command, args) {
  return tauri.execute(command, args)
}

export {
  execute
}
