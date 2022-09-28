window.__TAURI_ISOLATION_HOOK__ = (payload) => {
  console.log('hook', payload)
  return payload
}
