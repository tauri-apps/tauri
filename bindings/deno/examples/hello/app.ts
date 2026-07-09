// Example app code — runs in a Worker with a live event loop.
import { app } from '../../worker.ts'
import demoPlugin from './demo-plugin.ts'

const STATUS = Deno.env.get('FIXTURE_STATUS')

function log(...args: unknown[]) {
  const line = args.map(String).join(' ')
  console.log('[app]', line)
  if (STATUS) Deno.writeTextFileSync(STATUS, line + '\n', { append: true })
}

// Wire up the plugin's command handlers (its native side + ACL were set up by
// launch() on the main thread). No `plugin:demo|echo` strings in app code.
app.plugin(demoPlugin)

app.command('greet', ({ name }) => {
  log('greet:', name)
  return { message: `Hello ${name}! Greetings from Deno ${Deno.version.deno}.` }
})

let childCount = 0
app.command('open-window', async () => {
  const label = `child-${++childCount}`
  const win = await app.createWindow({
    label,
    url: 'index.html',
    title: `Child (${label})`,
    width: 420,
    height: 320
  })
  win.free()
  return { label }
})

app.on('ready', async () => {
  log('ready')

  // Exercise the window API against the config-declared main window.
  const main = app.getWindow('main')!
  log('main-title:', main.title())
  const { width, height } = main.innerSize()
  log('main-size:', width > 0 && height > 0 ? 'ok' : `bad ${width}x${height}`)
  log('labels:', JSON.stringify(app.windowLabels()))
  main.setTitle('Tauri FFI — Deno (ready)')

  // Runtime window creation: hidden child, checked and torn down again.
  const child = await app.createWindow({
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

app.on('window-event', (message) => log('window-event:', message.label, message.event.kind))

app.listen('frontend-ping', (payload) => log('frontend-ping:', JSON.stringify(payload)))
app.listen('plugin-init-ran', (payload) => log('plugin-init:', payload.ran ? 'ran' : 'missing'))
app.listen('plugin-echo-ok', (payload) => log('plugin-echo:', JSON.stringify(payload)))

let count = 0
setInterval(() => app.emit('tick', { count: ++count }), 1000)
