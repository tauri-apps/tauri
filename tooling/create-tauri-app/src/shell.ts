// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import execa, { Options, ExecaChildProcess } from 'execa'

export const shell = async (
  command: string,
  args?: string[],
  options?: Options,
  log: boolean = false
): Promise<ExecaChildProcess> => {
  try {
    if (options && options.shell === true) {
      const stringCommand = [command, ...(args ?? [])].join(' ')
      if (log) console.log(`[running]: ${stringCommand}`)
      return await execa(stringCommand, {
        stdio: 'inherit',
        cwd: process.cwd(),
        env: process.env,
        ...options
      })
    } else {
      if (log) console.log(`[running]: ${command}`)
      return await execa(
        command,
        args.filter((e) => e !== ''),
        {
          stdio: 'inherit',
          cwd: process.cwd(),
          env: process.env,
          ...options
        }
      )
    }
  } catch (error) {
    console.error('Error with command: %s', command)
    throw new Error(error)
  }
}
