const express = require('express')
const cors = require('cors')
const app = express()
app.use(cors())
app.use(express.json())
const port = 7000
let appPid

app.post('/reply', (req, res) => {
  if (req.body && req.body.msg !== 'TEST') {
    throw new Error(`unexpected reply ${JSON.stringify(req.body)}`)
  }
  console.log('App event replied')
  exit(0)
})

const server = app.listen(port, () => console.log(`Test listening on port ${port}!`))

const exit = code => {
  server.close()
  process.kill(appPid)
  process.exit(code)
}

const path = require('path')
const dist = path.resolve(__dirname, 'dist')

const build = require('../cli/tauri.js/dist/api/build')
build({
  build: {
    devPath: dist
  },
  ctx: {
    debug: true
  },
  tauri: {
    embeddedServer: {
      active: true
    }
  }
}).then(() => {
  const spawn = require('../cli/tauri.js/dist/helpers/spawn').spawn
  const artifactPath = path.resolve(__dirname, 'src-tauri/target/debug/app')
  appPid = spawn(
    process.platform === 'win32' ? `${artifactPath}.exe` : artifactPath.replace('debug/app', 'debug/./app'),
    [],
    null
  )

  // if it didn't reply, throw an error
  setTimeout(() => {
    throw new Error("App didn't reply")
  }, 2000)
})
