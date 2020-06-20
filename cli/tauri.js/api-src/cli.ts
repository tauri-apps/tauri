import { CliMatches } from './types/cli'
import { promisified } from './tauri'

/**
 * gets the CLI matches
 */
async function getMatches(): Promise<CliMatches> {
  return await promisified<CliMatches>({
    cmd: 'cliMatches'
  })
}

export {
  getMatches
}
