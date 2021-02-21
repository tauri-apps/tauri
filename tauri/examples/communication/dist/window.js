var urlInput = document.getElementById("url");

addClickEnterHandler(
  document.getElementById("open-url"),
  urlInput,
  function () {
    window.__TAURI__.shell.open(urlInput.value);
  }
);

var titleInput = document.getElementById("title");

addClickEnterHandler(
  document.getElementById("set-title"),
  titleInput,
  function () {
    window.__TAURI__.window.manager.setTitle(titleInput.value);
  }
);
