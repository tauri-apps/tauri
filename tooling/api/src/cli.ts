// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Parse arguments from your Command Line Interface.
 *
 * This package is also accessible with `window.__TAURI__.cli` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

interface ArgMatch {
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

interface SubcommandMatch {
  name: string
  matches: CliMatches
}

interface CliMatches {
  args: { [name: string]: ArgMatch }
  subcommand: SubcommandMatch | null
}

/**
 * Parse the arguments provided to the current process and get the matches using the configuration defined `tauri.conf.json > tauri > cli`.
 *
 * @returns A promise resolving to the parsed arguments.
 */
async function getMatches(): Promise<CliMatches> {
  return invokeTauriCommand<CliMatches>({
    __tauriModule: 'Cli',
    message: {
      cmd: 'cliMatches'
    }
  })
}

export type { ArgMatch, SubcommandMatch, CliMatches }

export { getMatches }
