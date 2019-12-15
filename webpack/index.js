const WebpackShellPlugin = require('webpack-shell-plugin')
const TauriRequirePlugin = require('./plugins/tauri-require').plugin

module.exports.chain = function (chain, { automaticStart = false } = {}) {
  if (automaticStart) {
    chain.plugin('webpack-shell-plugin')
      .use(WebpackShellPlugin, [{
        onBuildEnd: [process.env.NODE_ENV === 'production' ? 'tauri build' : 'tauri dev']
      }])
  }

  chain.plugin('tauri-require')
    .use(TauriRequirePlugin)
}
