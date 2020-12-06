import { invoke } from './tauri'

/**
 * sets the window title
 *
 * @param title the new title
 */
function setTitle(title: string): void {
  invoke({
    cmd: 'setTitle',
    title
  })
}

/**
 * opens an URL on the user default browser
 *
 * @param url the URL to open
 */
function open(url: string): void {
  invoke({
    cmd: 'open',
    uri: url
  })
}

export { setTitle, open }
