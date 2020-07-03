import {
  open,
  save
} from 'tauri/api/dialog'
import {
  readBinaryFile
} from 'tauri/api/fs'

export default () => {
  var defaultPathInput = document.getElementById('dialog-default-path')
  var filterInput = document.getElementById('dialog-filter')
  var multipleInput = document.getElementById('dialog-multiple')
  var directoryInput = document.getElementById('dialog-directory')

  document.getElementById('open-dialog').addEventListener('click', function () {
    open({
      defaultPath: defaultPathInput.value || null,
      filter: filterInput.value || null,
      multiple: multipleInput.checked,
      directory: directoryInput.checked
    }).then(function (res) {
      var pathToRead = res
      var isFile = pathToRead.match(/\S+\.\S+$/g)
      readBinaryFile(pathToRead).then(function (response) {
        if (isFile) {
          if (pathToRead.includes('.png') || pathToRead.includes('.jpg')) {
            arrayBufferToBase64(new Uint8Array(response), function (base64) {
              var src = 'data:image/png;base64,' + base64
              registerResponse('<img src="' + src + '"></img>')
            })
          } else {
            registerResponse(res)
          }
        } else {
          registerResponse(res)
        }
      }).catch(registerResponse(res))
    }).catch(registerResponse)
  })

  document.getElementById('save-dialog').addEventListener('click', function () {
    save({
      defaultPath: defaultPathInput.value || null,
      filter: filterInput.value || null
    }).then(registerResponse).catch(registerResponse)
  })
}
