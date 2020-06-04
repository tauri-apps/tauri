import tauri from './tauri'

/**
 * spawns a process
 *
 * @param {String} command the name of the cmd to execute e.g. 'mkdir' or 'node'
 * @param {(String[]|String)} [args] command args
 * @return {Promise<String>} promise resolving to the stdout text
 */
function execute (command, args) {
  return tauri.execute(command, args)
}

export {
  execute
}
