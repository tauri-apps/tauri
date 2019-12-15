const WebpackShellPlugin = require('webpack-shell-plugin')
const TauriRequirePlugin = require('./plugins/tauri-require').plugin

module.exports.chain = function (chain, { automaticStart = false } = {}) {
  if (automaticStart) {
    chain.plugin('webpack-shell-plugin')
      .use(WebpackShellPlugin, [{
        onBuildEnd: [ cfg.ctx.prod ? `tauri build` : `tauri dev` ]
      }])
  }

  chain.plugin('tauri-require')
    .use(TauriRequirePlugin)
}
