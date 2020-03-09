import tauri from './tauri'

/**
 * sets the window title
 *
 * @param {string} title the new title
 */
function setTitle (title) {
  tauri.setTitle(title)
}

/**
 * opens an URL on the user default browser
 *
 * @param {string} url the URL to open
 */
function open (url) {
  tauri.open(url)
}

export {
  setTitle,
  open
}
