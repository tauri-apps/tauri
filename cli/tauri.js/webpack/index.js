const HtmlWebpackInlineSourcePlugin = require('html-webpack-inline-source-plugin')
const tauriConfig = require('../helpers/tauri-config')
const WebpackShellPlugin = require('webpack-shell-plugin')

const safeTap = (options, cb) => {
  if (options !== undefined) {
    cb()
  }
  return options
}

module.exports.chain = function (chain) {
  const cfg = tauriConfig({
    ctx: {
      debug: process.env.NODE_ENV !== 'production',
      prod: process.env.NODE_ENV === 'production'
    }
  })
  if (!cfg.tauri.embeddedServer.active) {
    chain.optimization.splitChunks({
      chunks: 'all',
      minSize: 0,
      maxSize: Infinity,
      maxAsyncRequests: 1,
      maxInitialRequests: 1,
      automaticNameDelimiter: '~',
      name: true,
      cacheGroups: {
        styles: {
          name: 'styles',
          chunks: 'all'
        },
        commons: {
          name: 'vendors',
          chunks: 'all'
        }
      }
    })

    chain.output.filename('js/app.js')

    if (cfg.ctx.prod) {
      if (cfg.build.extractCSS) {
        chain.plugin('mini-css-extract')
          .tap(options => {
            options[0].filename = 'css/app.css'
            return options
          })
      }

      chain.plugin('html-webpack')
        .tap(options => {
          options[0].inlineSource = '.(js|css)$'
          return options
        })

      chain.module.rule('babel')
        .use('babel-loader')
        .tap(options => safeTap(options, () => {
          options.plugins.push([
            'system-import-transformer', { // needs constant attention
              modules: 'common'
            }
          ])
        }))
    }

    const modules = {
      images: 'url-loader',
      fonts: 'url-loader',
      media: 'url-loader'
    }
    for (const module in modules) {
      chain.module.rule(module)
        .use(modules[module])
        .tap(options => safeTap(options, () => {
          options.limit = undefined
        }))
    }
  }

  if (cfg.ctx.prod && !cfg.tauri.embeddedServer.active) {
    chain.plugin('html-webpack-inline-source')
      .use(HtmlWebpackInlineSourcePlugin)
  }

  if (cfg.tauri.automaticStart.active) {
    chain.plugin('webpack-shell-plugin')
      .use(WebpackShellPlugin, [{
        onBuildEnd: [
          cfg.ctx.prod
            ? `tauri build${cfg.tauri.automaticStart.buildArgs.join(' ')}`
            : `tauri dev${cfg.tauri.automaticStart.devArgs.join(' ')}`
        ]
      }])
  }
}
