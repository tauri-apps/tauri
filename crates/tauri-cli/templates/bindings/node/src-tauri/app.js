// Your app code — runs in a worker_thread with a live event loop.
import { app } from '@tauri-apps/node/worker'

// Frontend calls this with `invoke('greet', { name })`.
app.command('greet', ({ name }) => {
  return { message: `Hello, ${name}! You've been greeted from Node.js.` }
})

app.on('ready', () => {
  console.log('Tauri app ready')
})
