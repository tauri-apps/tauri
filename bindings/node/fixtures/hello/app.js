// Fixture app code — runs in a worker_thread with a live event loop.
import { appendFileSync } from 'node:fs'
import { app } from '../../src/worker.js'

// Smoke tests set FIXTURE_STATUS to a file path and assert on its contents.
function log(...args) {
  console.log('[app]', ...args)
  if (process.env.FIXTURE_STATUS) {
    appendFileSync(process.env.FIXTURE_STATUS, `${args.join(' ')}\n`)
  }
}

app.command('greet', ({ name }) => {
  log('greet:', name)
  return { message: `Hello ${name}! Greetings from Node.js ${process.version}.` }
})

app.on('ready', () => log('ready'))
app.on('window-event', (msg) => log('window-event:', msg.label, msg.event.kind))
app.on('exit', () => log('exit'))

// Frontend -> host events.
app.listen('frontend-ping', (payload) => log('frontend-ping:', JSON.stringify(payload)))

// Host -> frontend events, once a second.
let count = 0
setInterval(() => app.emit('tick', { count: ++count }), 1000)
