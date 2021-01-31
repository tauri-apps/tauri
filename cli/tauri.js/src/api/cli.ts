import { runOnRustCli } from '../helpers/rust-cli'

interface Args {
  [key: string]: string | Object
}

function runCliCommand(
  command: string,
  args: Args
): { pid: number; promise: Promise<void> } {
  const argsArray = []
  for (const argName in args) {
    const argValue = args[argName]
    if (argValue === false) {
      continue
    }
    argsArray.push(`--${argName}`)
    if (argValue === true) {
      continue
    }
    argsArray.push(
      typeof argValue === 'string' ? argValue : JSON.stringify(argValue)
    )
  }
  return runOnRustCli(command, argsArray)
}

export const dev = (args: Args): { pid: number; promise: Promise<void> } =>
  runCliCommand('dev', args)
export const build = (args: Args): { pid: number; promise: Promise<void> } =>
  runCliCommand('build', args)
