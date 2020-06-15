import tauri from './tauri'

/**
 * gets the CLI matches
 */
function getMatches() {
  return tauri.cliMatches()
}

export {
  getMatches
}
