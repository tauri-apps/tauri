/**
 * Access the system shell.
 * Allows you to spawn child processes and manage files and URLs using their default application.
 *
 * This package is also accessible with `window.__TAURI__.shell` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "shell": {
 *         "all": true, // enable all shell APIs
 *         "execute": true, // enable process spawn APIs
 *         "open": true // enable opening files/URLs using the default program
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */
interface SpawnOptions {
    /** Current working directory. */
    cwd?: string;
    /** Environment variables. set to `null` to clear the process env. */
    env?: {
        [name: string]: string;
    };
}
interface ChildProcess {
    /** Exit code of the process. `null` if the process was terminated by a signal on Unix. */
    code: number | null;
    /** If the process was terminated by a signal, represents that signal. */
    signal: number | null;
    /** The data that the process wrote to `stdout`. */
    stdout: string;
    /** The data that the process wrote to `stderr`. */
    stderr: string;
}
declare class EventEmitter<E extends string> {
    /** @ignore  */
    private eventListeners;
    /** @ignore  */
    private addEventListener;
    /** @ignore  */
    _emit(event: E, payload: any): void;
    /**
     * Listen to an event from the child process.
     *
     * @param event The event name.
     * @param handler The event handler.
     *
     * @return The `this` instance for chained calls.
     */
    on(event: E, handler: (arg: any) => void): EventEmitter<E>;
}
declare class Child {
    /** The child process `pid`. */
    pid: number;
    constructor(pid: number);
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
    write(data: string | number[]): Promise<void>;
    /**
     * Kills the child process.
     *
     * @return A promise indicating the success or failure of the operation.
     */
    kill(): Promise<void>;
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
declare class Command extends EventEmitter<'close' | 'error'> {
    /** @ignore Program to execute. */
    private readonly program;
    /** @ignore Program arguments */
    private readonly args;
    /** @ignore Spawn options. */
    private readonly options;
    /** Event emitter for the `stdout`. Emits the `data` event. */
    readonly stdout: EventEmitter<"data">;
    /** Event emitter for the `stderr`. Emits the `data` event. */
    readonly stderr: EventEmitter<"data">;
    /**
     * Creates a new `Command` instance.
     *
     * @param program The program to execute.
     * @param args Program arguments.
     * @param options Spawn options.
     */
    constructor(program: string, args?: string | string[], options?: SpawnOptions);
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
    static sidecar(program: string, args?: string | string[], options?: SpawnOptions): Command;
    /**
     * Executes the command as a child process, returning a handle to it.
     *
     * @return A promise resolving to the child process handle.
     */
    spawn(): Promise<Child>;
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
    execute(): Promise<ChildProcess>;
}
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
declare function open(path: string, openWith?: string): Promise<void>;
export { Command, Child, open };
export type { ChildProcess, SpawnOptions };
