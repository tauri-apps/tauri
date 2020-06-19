import { promisified } from './tauri'

/**
 * gets the CLI matches
 */
function getMatches(): any {
  return promisified({
    cmd: 'cliMatches'
  })
}

export {
  getMatches
}
