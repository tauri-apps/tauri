// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { runOnRustCli } from '../helpers/rust-cli'

interface Args {
  [key: string]: string | Object
}

interface Cmd {
  pid: number
  promise: Promise<void>
}

function toKebabCase(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, '$1-$2')
    .replace(/\s+/g, '-')
    .toLowerCase()
}

async function runCliCommand(command: string, args?: Args): Promise<Cmd> {
  const argsArray = []
  for (const [argName, argValue] of Object.entries(args ?? {})) {
    if (argValue === false) {
      continue
    }
    argsArray.push(`--${toKebabCase(argName)}`)
    if (argValue === true) {
      continue
    }
    argsArray.push(
      typeof argValue === 'string' ? argValue : JSON.stringify(argValue)
    )
  }
  return await runOnRustCli(command, argsArray)
}

export const init = async (args?: Args): Promise<Cmd> =>
  await runCliCommand('init', args)
export const dev = async (args?: Args): Promise<Cmd> =>
  await runCliCommand('dev', args)
export const build = async (args?: Args): Promise<Cmd> =>
  await runCliCommand('build', args)
