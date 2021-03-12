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
 * opens a path or URL with the system's default app,
 * or the one specified with `openWith`
 *
 * @param path the path or URL to open
 * @param openWith the app to open the file or URL with
 */
async function open(path: string, openWith?: string): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Shell',
    message: {
      cmd: 'open',
      path,
      with: openWith
    }
  })
}

export { execute, open }
