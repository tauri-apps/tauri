// Host-side app code — runs in a worker_thread with a live event loop.
// Mirrors the commands and events of examples/api/src-tauri/src (cmd.rs +
// lib.rs), so the shared Svelte frontend drives Node.js instead of Rust.
import { app } from '../../src/worker.js'
import appMenu from './app-menu-plugin.js'

// plugin:app-menu|toggle / |popup — wired up here (native side + ACL were set
// up by launch() on the main thread).
app.plugin(appMenu)

// Communication view: "Call Log API". Rust logs it through an ACL-scoped
// command; here we just print it (the frontend ignores the return value).
app.command('log_operation', ({ event, payload }) => {
  console.log('[app] log_operation:', event, payload ?? '')
})

// Communication view: "Call Request (async) API".
app.command('perform_request', ({ endpoint, body }) => {
  console.log('[app] perform_request:', endpoint, JSON.stringify(body))
  return { message: 'message response' }
})

// Communication view: "Echo". The Rust command echoes the raw request body
// (an object or a bare array), so we return the payload untouched.
app.command('echo', (payload) => payload)

// Communication view: "Spam". The Rust command streams 1..=1000 over an
// ipc::Channel; channels are not yet bridged across the FFI boundary, so this
// resolves without streaming. TODO(tauri-ffi): channel support.
app.command('spam', () => {})

// Communication view: "Send event to Rust" emits `js-event`; reply with
// `rust-event`, mirroring the listener in examples/api/src-tauri/src/lib.rs.
app.listen('js-event', (payload) => {
  console.log('[app] js-event:', JSON.stringify(payload))
  app.emit('rust-event', { data: 'something else' })
})

app.on('ready', () =>
  console.log('[app] ready — serving examples/api frontend')
)
app.on('exit', () => console.log('[app] exit'))
