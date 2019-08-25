const HtmlWebpackInlineSourcePlugin = require('html-webpack-inline-source-plugin')

module.exports.chain = function (chain, cfg) {

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

    chain.output.filename(`js/app.js`)

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
        .tap(options => {
          options.plugins.push([
            'system-import-transformer', { // needs constant attention
              modules: 'common'
            }
          ])
          return options
        })
    }

    const modules = {
      images: 'url-loader',
      fonts: 'url-loader',
      media: 'url-loader'
    }
    for (const module in modules) {
      chain.module.rule(module)
        .use(modules[module])
        .tap(options => {
          options.limit = undefined
          return options
        })
    }
  }

  if (cfg.ctx.prod && !cfg.tauri.embeddedServer.active) {
    chain.plugin('html-webpack-inline-source')
      .use(HtmlWebpackInlineSourcePlugin)
  }
}
