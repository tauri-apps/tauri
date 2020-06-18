var urlInput = document.getElementById('url')

addClickEnterHandler(
  document.getElementById('open-url'),
  urlInput,
  function () {
    window.__TAURI__.open(urlInput.value)
  }
)

var titleInput = document.getElementById('title')

addClickEnterHandler(
  document.getElementById('set-title'),
  titleInput,
  function () {
    window.__TAURI__.setTitle(titleInput.value)
  }
)
