// Your app code — runs in a Worker with a live event loop.
import { app } from '@tauri-apps/deno/worker'

// Frontend calls this with `invoke('greet', { name })`.
app.command('greet', ({ name }: { name: string }) => {
  return { message: `Hello, ${name}! You've been greeted from Deno.` }
})

app.on('ready', () => {
  console.log('Tauri app ready')
})
