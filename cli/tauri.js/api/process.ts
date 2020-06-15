import tauri from './tauri'

/**
 * spawns a process
 *
 * @param command the name of the cmd to execute e.g. 'mkdir' or 'node'
 * @param [args] command args
 * @return promise resolving to the stdout text
 */
function execute(command: string, args?: string | string[]): Promise<string> {
  return tauri.execute(command, args)
}

export {
  execute
}
