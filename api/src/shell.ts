// SPDX-License-Identifier: Apache-2.0 OR MIT

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
  signal: number | null
  stdout: string
  stderr: string
}

class EventEmitter<E> {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  eventListeners: { [key: string]: Array<(arg: any) => void> } = Object.create(
    null
  )

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
  pid: number

  constructor(pid: number) {
    this.pid = pid
  }

  async write(data: string | number[]): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Shell',
      message: {
        cmd: 'stdinWrite',
        pid: this.pid,
        buffer: data
      }
    })
  }

  async kill(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Shell',
      message: {
        cmd: 'killChild',
        pid: this.pid
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
  pid: number | null = null

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
            this._emit('error', event.payload)
            break
          case 'Terminated':
            this._emit('close', event.payload)
            break
          case 'Stdout':
            this.stdout._emit('data', event.payload)
            break
          case 'Stderr':
            this.stderr._emit('data', event.payload)
            break
        }
      },
      this.args
    ).then((pid) => new Child(pid))
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
      this.on('close', (payload: TerminatedPayload) => {
        resolve({
          code: payload.code,
          signal: payload.signal,
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
  payload: V
}

interface TerminatedPayload {
  code: number | null
  signal: number | null
}

type CommandEvent =
  | Event<'Stdout', string>
  | Event<'Stderr', string>
  | Event<'Terminated', TerminatedPayload>
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
