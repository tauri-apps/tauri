var dirSelect = document.getElementById('dir')
function getDir () {
  return dirSelect.value ? parseInt(dir.value) : null
}

function arrayBufferToBase64(buffer, callback) {
  var blob = new Blob([buffer], {
    type: 'application/octet-binary'
  })
  var reader = new FileReader()
  reader.onload = function (evt) {
    var dataurl = evt.target.result
    callback(dataurl.substr(dataurl.indexOf(',') + 1))
  }
  reader.readAsDataURL(blob)
}

var pathInput = document.getElementById('path-to-read')

addClickEnterHandler(
  document.getElementById('read'),
  pathInput,
  function () {
    var pathToRead = pathInput.value
    var isFile = pathToRead.match(/\S+\.\S+$/g)
    var opts = { dir: getDir() }
    var promise = isFile ? window.tauri.readBinaryFile(pathToRead, opts) : window.tauri.readDir(pathToRead, opts)
    promise.then(function (response) {
      if (isFile) {
        if (pathToRead.includes('.png') || pathToRead.includes('.jpg')) {
          arrayBufferToBase64(new Uint8Array(response), function (base64) {
            var src = 'data:image/png;base64,' + base64
            registerResponse('<img src="' + src + '"></img>')
          })
        } else {
          var value = String.fromCharCode.apply(null, response)
          registerResponse('<textarea id="file-response" style="height: 400px"></textarea><button id="file-save">Save</button>')
          var fileInput = document.getElementById('file-response')
          fileInput.value = value
          document.getElementById('file-save').addEventListener('click', function () {
            window.tauri.writeFile({
              file: pathToRead,
              contents: fileInput.value
            }, {
              dir: getDir()
            }).catch(registerResponse)
          })
        }
      } else {
        registerResponse(response)
      }
    }).catch(registerResponse)
  }
)
