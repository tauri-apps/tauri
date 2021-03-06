import { invokeTauriCommand } from './helpers/tauri'

/**
 * spawns a process
 *
 * @param command the name of the cmd to execute e.g. 'mkdir' or 'node'
 * @param [args] command args
 * @return promise resolving to the stdout text
 */
async function execute(
  command: string,
  args?: string | string[]
): Promise<string> {
  if (typeof args === 'object') {
    Object.freeze(args)
  }

  return invokeTauriCommand<string>({
    __tauriModule: 'Shell',
    message: {
      cmd: 'execute',
      command,
      args: typeof args === 'string' ? [args] : args
    }
  })
}

/**
 * opens an URL on the user default browser
 *
 * @param url the URL to open
 */
async function open(url: string): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Shell',
    message: {
      cmd: 'open',
      uri: url
    }
  })
}

export { execute, open }
