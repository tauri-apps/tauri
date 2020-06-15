import tauri from './tauri'

/**
 * sets the window title
 *
 * @param title the new title
 */
function setTitle(title: string): void {
  tauri.setTitle(title)
}

/**
 * opens an URL on the user default browser
 *
 * @param url the URL to open
 */
function open(url: string): void {
  tauri.open(url)
}

export {
  setTitle,
  open
}
