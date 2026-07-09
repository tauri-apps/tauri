// Fixture entry point — parks the OS main thread in the Tauri event loop and
// runs ./app.js in a worker. Run with: node bindings/node/fixtures/hello/main.js
import { readFileSync } from 'node:fs'
import { launch } from '../../src/index.js'

const config = JSON.parse(readFileSync(new URL('./tauri.conf.json', import.meta.url), 'utf8'))

launch(new URL('./app.js', import.meta.url), {
  config,
  assetsDir: new URL('./assets/', import.meta.url),
  commands: ['greet']
})
