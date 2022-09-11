const Window = window.__TAURI__.window.Window

const routeSelect = document.querySelector('#route')
const link = document.querySelector('#link')

routeSelect.addEventListener('change', (event) => {
  link.href = event.target.value
})

document.querySelector('#go').addEventListener('click', () => {
  window.location.href = window.location.origin + '/' + routeSelect.value
})

document.querySelector('#open-window').addEventListener('click', () => {
  new Window(Math.random().toString().replace('.', ''), {
    url: routeSelect.value
  })
})
