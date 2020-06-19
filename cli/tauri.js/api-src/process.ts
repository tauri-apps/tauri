import { promisified } from './tauri'

/**
 * spawns a process
 *
 * @param command the name of the cmd to execute e.g. 'mkdir' or 'node'
 * @param [args] command args
 * @return promise resolving to the stdout text
 */
async function execute(command: string, args?: string | string[]): Promise<string> {
  if (typeof args === 'object') {
    Object.freeze(args)
  }

  return promisified({
    cmd: 'execute',
    command,
    args: typeof args === 'string' ? [args] : args
  });
}

export {
  execute
}
