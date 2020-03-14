var urlInput = document.getElementById('url')

addClickEnterHandler(
  document.getElementById('open-url'),
  urlInput,
  function () {
    window.tauri.open(urlInput.value)
  }
)

var titleInput = document.getElementById('title')

addClickEnterHandler(
  document.getElementById('set-title'),
  titleInput,
  function () {
    window.tauri.setTitle(titleInput.value)
  }
)
