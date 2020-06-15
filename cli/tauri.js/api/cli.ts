import tauri from './tauri'

/**
 * gets the CLI matches
 */
function getMatches(): any {
  return tauri.cliMatches()
}

export {
  getMatches
}
