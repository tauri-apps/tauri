// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

interface SpawnOptions {
  // Current working directory.
  cwd?: string
  // Environment variables. set to `null` to clear the process env.
  env?: { [name: string]: string }
}

interface InternalSpawnOptions extends SpawnOptions {
  sidecar?: boolean
}

interface ChildProcess {
  code: number | null
  signal: number | null
  stdout: string
  stderr: string
}

/**
 * Spawns a process.
 *
 * @param program The name of the program to execute e.g. 'mkdir' or 'node'
 * @param sidecar Whether the program is a sidecar or a system program
 * @param onEvent
 * @param [args] Command args
 * @returns A promise resolving to the process id.
 */
async function execute(
  onEvent: (event: CommandEvent) => void,
  program: string,
  args?: string | string[],
  options?: InternalSpawnOptions
): Promise<number> {
  if (typeof args === 'object') {
    Object.freeze(args)
  }

  return invokeTauriCommand<number>({
    __tauriModule: 'Shell',
    message: {
      cmd: 'execute',
      program,
      args: typeof args === 'string' ? [args] : args,
      options,
      onEventFn: transformCallback(onEvent)
    }
  })
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
  options: InternalSpawnOptions
  stdout = new EventEmitter<'data'>()
  stderr = new EventEmitter<'data'>()
  pid: number | null = null

  constructor(
    program: string,
    args: string | string[] = [],
    options?: SpawnOptions
  ) {
    super()
    this.program = program
    this.args = typeof args === 'string' ? [args] : args
    this.options = options ?? {}
  }

  /**
   * Creates a command to execute the given sidecar binary.
   *
   * @param program Binary name
   * @returns
   */
  static sidecar(program: string, args: string | string[] = []): Command {
    const instance = new Command(program, args)
    instance.options.sidecar = true
    return instance
  }

  async spawn(): Promise<Child> {
    return execute(
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
      this.program,
      this.args,
      this.options
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
 * Opens a path or URL with the system's default app,
 * or the one specified with `openWith`.
 *
 * @param path the path or URL to open
 * @param [openWith] the app to open the file or URL with
 * @returns
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
export type { ChildProcess, SpawnOptions }
