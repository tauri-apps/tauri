const TauriRunner = require('./runner'),
  TauriInjector = require('./injector'),
  path = require('path')

module.exports = {
  runner: TauriRunner,
  injector: TauriInjector,
  apiTemplatePath: path.resolve(__dirname, '../lib/tauri.js')
}
