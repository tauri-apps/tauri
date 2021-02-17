import { invoke } from './tauri'

export interface ArgMatch {
  /**
   * string if takes value
   * boolean if flag
   * string[] or null if takes multiple values
   */
  value: string | boolean | string[] | null
  /**
   * number of occurrences
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
 * gets the CLI matches
 */
async function getMatches(): Promise<CliMatches> {
  return invoke<CliMatches>({
    __tauriModule: 'Cli',
    message: {
      cmd: 'cliMatches'
    }
  })
}

export { getMatches }
