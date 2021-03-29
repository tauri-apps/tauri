import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

/**
 * spawns a process
 *
 * @param program the name of the program to execute e.g. 'mkdir' or 'node'
 * @param sidecar whether the program is a sidecar or a system program
 * @param [args] command args
 * @return promise resolving to the stdout text
 */
async function execute(
  program: string,
  sidecar: boolean,
  onEvent: (event: CommandEvent) => void,
  args?: string | string[]
): Promise<number> {
  if (typeof args === 'object') {
    Object.freeze(args)
  }

  return invokeTauriCommand<number>({
    __tauriModule: 'Shell',
    message: {
      cmd: 'execute',
      program,
      sidecar,
      onEventFn: transformCallback(onEvent),
      args: typeof args === 'string' ? [args] : args
    }
  })
}

interface ChildProcess {
  code: number | null
  stdout: string
  stderr: string
}

class EventEmitter<E> {
  eventListeners: { [key: string]: Array<(arg: any) => void> } = {}

  private addEventListener(event: string, handler: (arg: any) => void): void {
    if (event in this.eventListeners) {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[event].push(handler)
    } else {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[event] = [handler]
    }
  }

  _emit(event: E, payload: any): void {
    if (event in this.eventListeners) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      const listeners = this.eventListeners[event as any]
      for (const listener of listeners) {
        listener(payload)
      }
    }
  }

  on(event: E, handler: (arg: any) => void): EventEmitter<E> {
    this.addEventListener(event as any, handler)
    return this
  }
}

class Child {
  id: number

  constructor(id: number) {
    this.id = id
  }

  async kill(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Shell',
      message: {
        cmd: 'killChild',
        id: this.id
      }
    })
  }
}

class Command extends EventEmitter<'close' | 'error'> {
  program: string
  args: string[]
  sidecar = false
  stdout = new EventEmitter<'data'>()
  stderr = new EventEmitter<'data'>()

  constructor(program: string, args: string | string[] = []) {
    super()
    this.program = program
    this.args = typeof args === 'string' ? [args] : args
  }

  /**
   * Creates a command to execute the given sidecar binary
   *
   * @param {string} program  Binary name
   *
   * @return {Command}
   */
  static sidecar(program: string, args: string | string[] = []): Command {
    const instance = new Command(program, args)
    instance.sidecar = true
    return instance
  }

  async spawn(): Promise<Child> {
    return execute(
      this.program,
      this.sidecar,
      (event) => {
        switch (event.event) {
          case 'Error':
            this._emit('error', event.value)
            break
          case 'Finish':
            this._emit('close', event.value)
            break
          case 'Stdout':
            this.stdout._emit('data', event.value)
            break
          case 'Stderr':
            this.stderr._emit('data', event.value)
            break
        }
      },
      this.args
    ).then((id) => new Child(id))
  }

  async execute(): Promise<ChildProcess> {
    return new Promise((resolve, reject) => {
      this.on('error', reject)
      const stdout: string[] = []
      const stderr: string[] = []
      this.stdout.on('data', (line) => {
        stdout.push(line)
      })
      this.stderr.on('data', (line) => {
        stderr.push(line)
      })
      this.on('close', (code) => {
        resolve({
          code: code as number,
          stdout: stdout.join('\n'),
          stderr: stderr.join('\n')
        })
      })
      this.spawn().catch(reject)
    })
  }
}

interface Event<T, V> {
  event: T
  value: V
}
type CommandEvent =
  | Event<'Stdout', string>
  | Event<'Stderr', string>
  | Event<'Finish', number | null>
  | Event<'Error', string>

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

export { Command, Child, open }
