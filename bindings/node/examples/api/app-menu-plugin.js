// The `app-menu` plugin the shared frontend expects. The Welcome view's
// "Context menu" button invokes `plugin:app-menu|popup`, and Ctrl/Cmd+B
// invokes `plugin:app-menu|toggle`. examples/api wires these to real menus in
// Rust; here we just acknowledge them so the invoke resolves cleanly. Build
// out with the worker menu API (app.createMenu / app.setAppMenu / window
// hide/showMenu) to make them do something.
import { definePlugin } from '../../src/plugin.js'

export default definePlugin('app-menu')
  .command('toggle', () => {
    console.log('[app] app-menu toggle')
  })
  .command('popup', () => {
    console.log('[app] app-menu popup')
  })
