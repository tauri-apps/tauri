const OS_NAME = __TEMPLATE_os_name__

let zoomLevel = 1

const MAX_ZOOM_LEVEL = 10
const MIN_ZOOM_LEVEL = 0.2

window.addEventListener('keydown', (event) => {
  if (event.defaultPrevented) {
    return
  }
  if (OS_NAME === 'macos' ? event.metaKey : event.ctrlKey) {
    if (event.key === '-') {
      zoomLevel -= 0.2
    } else if (event.key === '=') {
      zoomLevel += 0.2
    } else {
      return
    }
    zoomLevel = Math.min(Math.max(zoomLevel, MIN_ZOOM_LEVEL), MAX_ZOOM_LEVEL)
    window.__TAURI_INTERNALS__.invoke('plugin:webview|set_webview_zoom', {
      value: zoomLevel
    })
  }
})
