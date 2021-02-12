import { invoke } from './tauri'

/**
 * sets the window title
 *
 * @param title the new title
 */
function setTitle(title: string): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setTitle',
      title
    }
  })
}

export { setTitle }
