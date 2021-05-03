// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

/**
 * Access the system shell.
 * Allows you to spawn child processes and manage files and URLs using their default application.
 * @packageDocumentation
 */

interface SpawnOptions {
  /** Current working directory. */
  cwd?: string
  /** Environment variables. set to `null` to clear the process env. */
  env?: { [name: string]: string }
}

/** @ignore */
interface InternalSpawnOptions extends SpawnOptions {
  sidecar?: boolean
}

interface ChildProcess {
  /** Exit code of the process. `null` if the process was terminated by a signal on Unix. */
  code: number | null
  /** If the process was terminated by a signal, represents that signal. */
  signal: number | null
  /** The data that the process wrote to `stdout`. */
  stdout: string
  /** The data that the process wrote to `stderr`. */
  stderr: string
}

/**
 * Spawns a process.
 *
 * @ignore
 * @param program The name of the program to execute e.g. 'mkdir' or 'node'.
 * @param sidecar Whether the program is a sidecar or a system program.
 * @param onEvent Event handler.
 * @param args Program arguments.
 * @param options Configuration for the process spawn.
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
  /** @ignore  */
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  private eventListeners: {
    [key: string]: Array<(arg: any) => void>
  } = Object.create(null)

  /** @ignore  */
  private addEventListener(event: string, handler: (arg: any) => void): void {
    if (event in this.eventListeners) {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[event].push(handler)
    } else {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[event] = [handler]
    }
  }

  /** @ignore  */
  _emit(event: E, payload: any): void {
    if (event in this.eventListeners) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      const listeners = this.eventListeners[event as any]
      for (const listener of listeners) {
        listener(payload)
      }
    }
  }

  /**
   * Listen to an event from the child process.
   *
   * @param event The event name.
   * @param handler The event handler.
   *
   * @return The `this` instance for chained calls.
   */
  on(event: E, handler: (arg: any) => void): EventEmitter<E> {
    this.addEventListener(event as any, handler)
    return this
  }
}

class Child {
  /** The child process `pid`. */
  pid: number

  constructor(pid: number) {
    this.pid = pid
  }

  /**
   * Writes `data` to the `stdin`.
   *
   * @param data The message to write, either a string or a byte array.
   * @example
   * ```typescript
   * const command = new Command('node')
   * const child = await command.spawn()
   * await child.write('message')
   * await child.write([0, 1, 2, 3, 4, 5])
   * ```
   *
   * @return A promise indicating the success or failure of the operation.
   */
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

  /**
   * Kills the child process.
   *
   * @return A promise indicating the success or failure of the operation.
   */
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

/**
 * The entry point for spawning child processes.
 * It emits the `close` and `error` events.
 * @example
 * ```typescript
 * const command = new Command('node')
 * command.on('close', data => {
 *   console.log(`command finished with code ${data.code} and signal ${data.signal}`)
 * })
 * command.on('error', error => console.error(`command error: "${error}"`))
 * command.stdout.on('data', line => console.log(`command stdout: "${line}"`))
 * command.stderr.on('data', line => console.log(`command stderr: "${line}"`))
 *
 * const child = await command.spawn()
 * console.log('pid:', child.pid)
 * ```
 */
class Command extends EventEmitter<'close' | 'error'> {
  /** @ignore Program to execute. */
  private readonly program: string
  /** @ignore Program arguments */
  private readonly args: string[]
  /** @ignore Spawn options. */
  private readonly options: InternalSpawnOptions
  /** Event emitter for the `stdout`. Emits the `data` event. */
  readonly stdout = new EventEmitter<'data'>()
  /** Event emitter for the `stderr`. Emits the `data` event. */
  readonly stderr = new EventEmitter<'data'>()

  /**
   * Creates a new `Command` instance.
   *
   * @param program The program to execute.
   * @param args Program arguments.
   * @param options Spawn options.
   */
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
   * Creates a command to execute the given sidecar program.
   * @example
   * ```typescript
   * const command = Command.sidecar('my-sidecar')
   * const output = await command.execute()
   * ```
   *
   * @param program The program to execute.
   * @param args Program arguments.
   * @param options Spawn options.
   * @returns
   */
  static sidecar(
    program: string,
    args: string | string[] = [],
    options?: SpawnOptions
  ): Command {
    const instance = new Command(program, args, options)
    instance.options.sidecar = true
    return instance
  }

  /**
   * Executes the command as a child process, returning a handle to it.
   *
   * @return A promise resolving to the child process handle.
   */
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

  /**
   * Executes the command as a child process, waiting for it to finish and collecting all of its output.
   * @example
   * ```typescript
   * const output = await new Command('echo', 'message').execute()
   * assert(output.code === 0)
   * assert(output.signal === null)
   * assert(output.stdout === 'message')
   * assert(output.stderr === '')
   * ```
   *
   * @return A promise resolving to the child process output.
   */
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

/**
 * Describes the event message received from the command.
 */
interface Event<T, V> {
  event: T
  payload: V
}

/**
 * Payload for the `Terminated` command event.
 */
interface TerminatedPayload {
  /** Exit code of the process. `null` if the process was terminated by a signal on Unix. */
  code: number | null
  /** If the process was terminated by a signal, represents that signal. */
  signal: number | null
}

/** Events emitted by the child process. */
type CommandEvent =
  | Event<'Stdout', string>
  | Event<'Stderr', string>
  | Event<'Terminated', TerminatedPayload>
  | Event<'Error', string>

/**
 * Opens a path or URL with the system's default app,
 * or the one specified with `openWith`.
 * @example
 * ```typescript
 * // opens the given URL on the default browser:
 * await open('https://github.com/tauri-apps/tauri')
 * // opens the given URL using `firefox`:
 * await open('https://github.com/tauri-apps/tauri', 'firefox')
 * // opens a file using the default program:
 * await open('/path/to/file')
 * ```
 *
 * @param path The path or URL to open.
 * @param openWith The app to open the file or URL with. Defaults to the system default application for the specified path type.
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
