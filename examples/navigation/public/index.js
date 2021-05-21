const WebviewWindow = window.__TAURI__.window.WebviewWindow

const routeSelect = document.querySelector('#route')
const link = document.querySelector('#link')

routeSelect.addEventListener('change', (event) => {
  link.href = event.target.value
})

document.querySelector('#go').addEventListener('click', () => {
  window.location.href = (window.location.origin + '/' + routeSelect.value)
})

document.querySelector('#open-window').addEventListener('click', () => {
  new WebviewWindow(Math.random().toString(), {
    url: routeSelect.value
  })
})
