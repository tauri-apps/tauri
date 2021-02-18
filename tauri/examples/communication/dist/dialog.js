var defaultPathInput = document.getElementById("dialog-default-path");
var filterInput = document.getElementById("dialog-filter");
var multipleInput = document.getElementById("dialog-multiple");
var directoryInput = document.getElementById("dialog-directory");

document.getElementById("open-dialog").addEventListener("click", function () {
  window.__TAURI__.dialog
    .open({
      defaultPath: defaultPathInput.value || null,
      filters: filterInput.value ? [{
        name: 'Tauri Example',
        extensions: filterInput.value.split(',').map(f => f.trim())
      }] : [],
      multiple: multipleInput.checked,
      directory: directoryInput.checked,
    })
    .then(function (res) {
      if (Array.isArray(res)) {
        registerResponse(res);
      } else {
        var pathToRead = res;
        var isFile = pathToRead.match(/\S+\.\S+$/g);
        window.__TAURI__.fs
          .readBinaryFile(pathToRead)
          .then(function (response) {
            if (isFile) {
              if (pathToRead.includes(".png") || pathToRead.includes(".jpg")) {
                arrayBufferToBase64(new Uint8Array(response), function (base64) {
                  var src = "data:image/png;base64," + base64;
                  registerResponse('<img src="' + src + '"></img>');
                });
              } else {
                registerResponse(res);
              }
            } else {
              registerResponse(res);
            }
          })
          .catch(registerResponse(res));
      }
    })
    .catch(registerResponse);
});

document.getElementById("save-dialog").addEventListener("click", function () {
  window.__TAURI__.dialog
    .save({
      defaultPath: defaultPathInput.value || null,
      filters: filterInput.value ? [{
        name: 'Tauri Example',
        extensions: filterInput.value.split(',').map(f => f.trim())
      }] : [],
    })
    .then(registerResponse)
    .catch(registerResponse);
});
