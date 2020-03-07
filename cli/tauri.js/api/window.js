import tauri from './tauri'

function setTitle (title) {
  tauri.setTitle(title)
}

function open (url) {
  tauri.open(url)
}

export {
  setTitle,
  open
}
