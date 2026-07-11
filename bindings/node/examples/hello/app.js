// Fixture app code — runs in a worker_thread with a live event loop.
import { appendFileSync } from 'node:fs'
import { app } from '../../src/worker.js'
import demoPlugin from './demo-plugin.js'

// Smoke tests set FIXTURE_STATUS to a file path and assert on its contents.
function log(...args) {
  console.log('[app]', ...args)
  if (process.env.FIXTURE_STATUS) {
    appendFileSync(process.env.FIXTURE_STATUS, `${args.join(' ')}\n`)
  }
}

// Wire up the plugin's command handlers (its native side + ACL were set up by
// launch() on the main thread). No `plugin:demo|echo` strings in app code.
app.plugin(demoPlugin)

app.command('greet', ({ name }) => {
  log('greet:', name)
  return { message: `Hello ${name}! Greetings from Node.js ${process.version}.` }
})

let childCount = 0
app.command('open-window', () => {
  const label = `child-${++childCount}`
  const win = app.createWindow({
    label,
    url: 'index.html',
    title: `Child ${childCount}`,
    width: 420,
    height: 320
  })
  win.free()
  return { label }
})

app.on('ready', () => {
  log('ready')

  // Exercise the window API against the config-declared main window.
  const main = app.getWindow('main')
  log('main-title:', main.title())
  log('main-url:', main.url()) // dev server URL under `tauri dev`, tauri://localhost in production
  const { width, height } = main.innerSize()
  log('main-size:', width > 0 && height > 0 ? 'ok' : `bad ${width}x${height}`)
  log('labels:', JSON.stringify(app.windowLabels()))
  main.setTitle('Tauri FFI — Node.js (ready)')

  // Platform-gated APIs: same symbols on every platform, ERR_UNSUPPORTED (-6)
  // where the feature does not exist.
  if (process.platform === 'darwin') {
    app.setActivationPolicy('regular')
    main.setTitleBarStyle('visible')
    log('platform-api:', main.nsWindow() > 0 ? 'ok' : 'bad ns_window')
  }
  try {
    main.hwnd()
    log('platform-gate:', process.platform === 'win32' ? 'ok' : 'hwnd unexpectedly succeeded')
  } catch (e) {
    log('platform-gate:', /only supported on Windows/.test(e.message) ? 'ok' : `bad ${e.message}`)
  }

  // Runtime window creation: hidden child, checked and torn down again.
  const child = app.createWindow({
    label: 'child',
    url: 'index.html',
    title: 'child',
    width: 320,
    height: 240,
    visible: false
  })
  log('child-created:', child.label(), child.isVisible() ? 'visible' : 'hidden')
  child.destroy()
  child.free()
  main.free()
  log('child-destroyed')
})
app.on('window-event', (msg) => log('window-event:', msg.label, msg.event.kind))
app.on('exit', () => log('exit'))

// Frontend -> host events.
app.listen('frontend-ping', (payload) => log('frontend-ping:', JSON.stringify(payload)))
app.listen('plugin-init-ran', (payload) => log('plugin-init:', payload.ran ? 'ran' : 'missing'))
app.listen('plugin-echo-ok', (payload) => log('plugin-echo:', JSON.stringify(payload)))

// Host -> frontend events, once a second.
let count = 0
setInterval(() => app.emit('tick', { count: ++count }), 1000)
