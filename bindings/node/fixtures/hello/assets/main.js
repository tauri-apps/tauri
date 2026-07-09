// Frontend of the fixture — exercises invoke + events through the injected
// global API (config: app.withGlobalTauri = true).
const status = document.querySelector('#status')

if (!window.__TAURI__) {
  status.textContent = 'window.__TAURI__ missing — global API script was not injected'
} else {
  status.textContent = `bridge ok (${Object.keys(window.__TAURI__).length} modules)`
  const { invoke } = window.__TAURI__.core
  const { listen, emit } = window.__TAURI__.event

  document.querySelector('#greet').addEventListener('click', async () => {
    const name = document.querySelector('#name').value
    try {
      const response = await invoke('greet', { name })
      document.querySelector('#greeting').textContent = response.message
    } catch (error) {
      document.querySelector('#greeting').textContent = `error: ${error}`
    }
  })

  listen('tick', (event) => {
    document.querySelector('#ticks').textContent = event.payload.count
  })

  document.querySelector('#ping').addEventListener('click', () => {
    emit('frontend-ping', { at: new Date().toISOString() })
  })

  // Auto-exercise the bridge once on load so smoke tests can assert on the
  // host-side FIXTURE_STATUS file without clicking anything.
  invoke('greet', { name: 'smoke-test' })
    .then((response) => {
      document.querySelector('#greeting').textContent = response.message
    })
    .catch((error) => {
      document.querySelector('#greeting').textContent = `error: ${error}`
    })
  emit('frontend-ping', { smoke: true })
}
