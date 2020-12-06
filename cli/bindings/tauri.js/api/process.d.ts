/**
 * spawns a process
 *
 * @param command the name of the cmd to execute e.g. 'mkdir' or 'node'
 * @param [args] command args
 * @return promise resolving to the stdout text
 */
declare function execute(command: string, args?: string | string[]): Promise<string>;
export { execute };
