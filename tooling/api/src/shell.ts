// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the system shell.
 * Allows you to spawn child processes and manage files and URLs using their default application.
 *
 * This package is also accessible with `window.__TAURI__.shell` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.shell`](https://tauri.app/v1/api/config/#allowlistconfig.shell) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "shell": {
 *         "all": true, // enable all shell APIs
 *         "execute": true, // enable process spawn APIs
 *         "sidecar": true, // enable spawning sidecars
 *         "open": true // enable opening files/URLs using the default program
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * ## Security
 *
 * This API has a scope configuration that forces you to restrict the programs and arguments that can be used.
 *
 * ### Restricting access to the {@link open | `open`} API
 *
 * On the allowlist, `open: true` means that the {@link open} API can be used with any URL,
 * as the argument is validated with the `^https?://` regex.
 * You can change that regex by changing the boolean value to a string, e.g. `open: ^https://github.com/`.
 *
 * ### Restricting access to the {@link Command | `Command`} APIs
 *
 * The `shell` allowlist object has a `scope` field that defines an array of CLIs that can be used.
 * Each CLI is a configuration object `{ name: string, cmd: string, sidecar?: bool, args?: boolean | Arg[] }`.
 *
 * - `name`: the unique identifier of the command, passed to the {@link Command.constructor | Command constructor}.
 * If it's a sidecar, this must be the value defined on `tauri.conf.json > tauri > bundle > externalBin`.
 * - `cmd`: the program that is executed on this configuration. If it's a sidecar, this value is ignored.
 * - `sidecar`: whether the object configures a sidecar or a system program.
 * - `args`: the arguments that can be passed to the program. By default no arguments are allowed.
 *   - `true` means that any argument list is allowed.
 *   - `false` means that no arguments are allowed.
 *   - otherwise an array can be configured. Each item is either a string representing the fixed argument value
 *     or a `{ validator: string }` that defines a regex validating the argument value.
 *
 * #### Example scope configuration
 *
 * CLI: `git commit -m "the commit message"`
 *
 * Configuration:
 * ```json
 * {
 *   "scope": {
 *     "name": "run-git-commit",
 *     "cmd": "git",
 *     "args": ["commit", "-m", { "validator": "\\S+" }]
 *   }
 * }
 * ```
 * Usage:
 * ```typescript
 * import { Command } from '@tauri-apps/api/shell'
 * new Command('run-git-commit', ['commit', '-m', 'the commit message'])
 * ```
 *
 * Trying to execute any API with a program not configured on the scope results in a promise rejection due to denied access.
 *
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

/**
 * @since 1.0.0
 */
interface SpawnOptions {
  /** Current working directory. */
  cwd?: string
  /** Environment variables. set to `null` to clear the process env. */
  env?: { [name: string]: string }
  /**
   * Character encoding for stdout/stderr
   *
   * @since 1.1.0
   *  */
  encoding?: string
}

/** @ignore */
interface InternalSpawnOptions extends SpawnOptions {
  sidecar?: boolean
}

/**
 * @since 1.0.0
 */
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
 * @param program The name of the scoped command.
 * @param onEvent Event handler.
 * @param args Program arguments.
 * @param options Configuration for the process spawn.
 * @returns A promise resolving to the process id.
 */
async function execute(
  onEvent: (event: CommandEvent) => void,
  program: string,
  args: string | string[] = [],
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
      args,
      options,
      onEventFn: transformCallback(onEvent)
    }
  })
}

/**
 * @since 1.0.0
 */
class EventEmitter<E extends string> {
  /** @ignore */
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  private eventListeners: Record<E, Array<(...args: any[]) => void>> =
    Object.create(null)

  /**
   * Alias for `emitter.on(eventName, listener)`.
   *
   * @since 1.1.0
   */
  addListener(eventName: E, listener: (...args: any[]) => void): this {
    return this.on(eventName, listener)
  }

  /**
   * Alias for `emitter.off(eventName, listener)`.
   *
   * @since 1.1.0
   */
  removeListener(eventName: E, listener: (...args: any[]) => void): this {
    return this.off(eventName, listener)
  }

  /**
   * Adds the `listener` function to the end of the listeners array for the
   * event named `eventName`. No checks are made to see if the `listener` has
   * already been added. Multiple calls passing the same combination of `eventName`and `listener` will result in the `listener` being added, and called, multiple
   * times.
   *
   * Returns a reference to the `EventEmitter`, so that calls can be chained.
   *
   * @since 1.0.0
   */
  on(eventName: E, listener: (...args: any[]) => void): this {
    if (eventName in this.eventListeners) {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[eventName].push(listener)
    } else {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[eventName] = [listener]
    }
    return this
  }

  /**
   * Adds a **one-time**`listener` function for the event named `eventName`. The
   * next time `eventName` is triggered, this listener is removed and then invoked.
   *
   * Returns a reference to the `EventEmitter`, so that calls can be chained.
   *
   * @since 1.1.0
   */
  once(eventName: E, listener: (...args: any[]) => void): this {
    const wrapper = (...args: any[]): void => {
      this.removeListener(eventName, wrapper)
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      listener(...args)
    }
    return this.addListener(eventName, wrapper)
  }

  /**
   * Removes the all specified listener from the listener array for the event eventName
   * Returns a reference to the `EventEmitter`, so that calls can be chained.
   *
   * @since 1.1.0
   */
  off(eventName: E, listener: (...args: any[]) => void): this {
    if (eventName in this.eventListeners) {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[eventName] = this.eventListeners[eventName].filter(
        (l) => l !== listener
      )
    }
    return this
  }

  /**
   * Removes all listeners, or those of the specified eventName.
   *
   * Returns a reference to the `EventEmitter`, so that calls can be chained.
   *
   * @since 1.1.0
   */
  removeAllListeners(event?: E): this {
    if (event) {
      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete,security/detect-object-injection
      delete this.eventListeners[event]
    } else {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      this.eventListeners = Object.create(null)
    }
    return this
  }

  /**
   * @ignore
   * Synchronously calls each of the listeners registered for the event named`eventName`, in the order they were registered, passing the supplied arguments
   * to each.
   *
   * @returns `true` if the event had listeners, `false` otherwise.
   */
  emit(eventName: E, ...args: any[]): boolean {
    if (eventName in this.eventListeners) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment,security/detect-object-injection
      const listeners = this.eventListeners[eventName]
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      for (const listener of listeners) listener(...args)
      return true
    }
    return false
  }

  /**
   * Returns the number of listeners listening to the event named `eventName`.
   *
   * @since 1.1.0
   */
  listenerCount(eventName: E): number {
    if (eventName in this.eventListeners)
      // eslint-disable-next-line security/detect-object-injection
      return this.eventListeners[eventName].length
    return 0
  }

  /**
   * Adds the `listener` function to the _beginning_ of the listeners array for the
   * event named `eventName`. No checks are made to see if the `listener` has
   * already been added. Multiple calls passing the same combination of `eventName`and `listener` will result in the `listener` being added, and called, multiple
   * times.
   *
   * Returns a reference to the `EventEmitter`, so that calls can be chained.
   *
   * @since 1.1.0
   */
  prependListener(eventName: E, listener: (...args: any[]) => void): this {
    if (eventName in this.eventListeners) {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[eventName].unshift(listener)
    } else {
      // eslint-disable-next-line security/detect-object-injection
      this.eventListeners[eventName] = [listener]
    }
    return this
  }

  /**
   * Adds a **one-time**`listener` function for the event named `eventName` to the_beginning_ of the listeners array. The next time `eventName` is triggered, this
   * listener is removed, and then invoked.
   *
   * Returns a reference to the `EventEmitter`, so that calls can be chained.
   *
   * @since 1.1.0
   */
  prependOnceListener(eventName: E, listener: (...args: any[]) => void): this {
    const wrapper = (...args: any[]): void => {
      this.removeListener(eventName, wrapper)
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      listener(...args)
    }
    return this.prependListener(eventName, wrapper)
  }
}

/**
 * @since 1.1.0
 */
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
   * import { Command } from '@tauri-apps/api/shell';
   * const command = new Command('node');
   * const child = await command.spawn();
   * await child.write('message');
   * await child.write([0, 1, 2, 3, 4, 5]);
   * ```
   *
   * @returns A promise indicating the success or failure of the operation.
   */
  async write(data: string | Uint8Array): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Shell',
      message: {
        cmd: 'stdinWrite',
        pid: this.pid,
        // correctly serialize Uint8Arrays
        buffer: typeof data === 'string' ? data : Array.from(data)
      }
    })
  }

  /**
   * Kills the child process.
   *
   * @returns A promise indicating the success or failure of the operation.
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
 * import { Command } from '@tauri-apps/api/shell';
 * const command = new Command('node');
 * command.on('close', data => {
 *   console.log(`command finished with code ${data.code} and signal ${data.signal}`)
 * });
 * command.on('error', error => console.error(`command error: "${error}"`));
 * command.stdout.on('data', line => console.log(`command stdout: "${line}"`));
 * command.stderr.on('data', line => console.log(`command stderr: "${line}"`));
 *
 * const child = await command.spawn();
 * console.log('pid:', child.pid);
 * ```
 *
 * @since 1.1.0
 *
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
   * @param program The program name to execute.
   * It must be configured on `tauri.conf.json > tauri > allowlist > shell > scope`.
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
   * import { Command } from '@tauri-apps/api/shell';
   * const command = Command.sidecar('my-sidecar');
   * const output = await command.execute();
   * ```
   *
   * @param program The program to execute.
   * It must be configured on `tauri.conf.json > tauri > allowlist > shell > scope`.
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
   * @returns A promise resolving to the child process handle.
   */
  async spawn(): Promise<Child> {
    return execute(
      (event) => {
        switch (event.event) {
          case 'Error':
            this.emit('error', event.payload)
            break
          case 'Terminated':
            this.emit('close', event.payload)
            break
          case 'Stdout':
            this.stdout.emit('data', event.payload)
            break
          case 'Stderr':
            this.stderr.emit('data', event.payload)
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
   * import { Command } from '@tauri-apps/api/shell';
   * const output = await new Command('echo', 'message').execute();
   * assert(output.code === 0);
   * assert(output.signal === null);
   * assert(output.stdout === 'message');
   * assert(output.stderr === '');
   * ```
   *
   * @returns A promise resolving to the child process output.
   */
  async execute(): Promise<ChildProcess> {
    return new Promise((resolve, reject) => {
      this.on('error', reject)
      const stdout: string[] = []
      const stderr: string[] = []
      this.stdout.on('data', (line: string) => {
        stdout.push(line)
      })
      this.stderr.on('data', (line: string) => {
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
 *
 * The `openWith` value must be one of `firefox`, `google chrome`, `chromium` `safari`,
 * `open`, `start`, `xdg-open`, `gio`, `gnome-open`, `kde-open` or `wslview`.
 *
 * @example
 * ```typescript
 * import { open } from '@tauri-apps/api/shell';
 * // opens the given URL on the default browser:
 * await open('https://github.com/tauri-apps/tauri');
 * // opens the given URL using `firefox`:
 * await open('https://github.com/tauri-apps/tauri', 'firefox');
 * // opens a file using the default program:
 * await open('/path/to/file');
 * ```
 *
 * @param path The path or URL to open.
 * This value is matched against the string regex defined on `tauri.conf.json > tauri > allowlist > shell > open`,
 * which defaults to `^https?://`.
 * @param openWith The app to open the file or URL with.
 * Defaults to the system default application for the specified path type.
 *
 * @since 1.0.0
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

export { Command, Child, EventEmitter, open }
export type { ChildProcess, SpawnOptions }
