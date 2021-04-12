// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'

export interface ArgMatch {
  /**
   * string if takes value
   * boolean if flag
   * string[] or null if takes multiple values
   */
  value: string | boolean | string[] | null
  /**
   * Number of occurrences
   */
  occurrences: number
}

export interface SubcommandMatch {
  name: string
  matches: CliMatches
}

export interface CliMatches {
  args: { [name: string]: ArgMatch }
  subcommand: SubcommandMatch | null
}

/**
 * Gets the CLI matches
 *
 * @returns {Promise<CliMatches>} A promise resolving to cli matches
 */
async function getMatches(): Promise<CliMatches> {
  return invokeTauriCommand<CliMatches>({
    __tauriModule: 'Cli',
    message: {
      cmd: 'cliMatches'
    }
  })
}

export { getMatches }
