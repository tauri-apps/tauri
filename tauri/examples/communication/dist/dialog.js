var defaultPathInput = document.getElementById('dialog-default-path')
var filterInput = document.getElementById('dialog-filter')
var multipleInput = document.getElementById('dialog-multiple')
var directoryInput = document.getElementById('dialog-directory')

document.getElementById('open-dialog').addEventListener('click', function () {
  window.tauri.openDialog({
    defaultPath: defaultPathInput.value || null,
    filter: filterInput.value || null,
    multiple: multipleInput.checked,
    directory: directoryInput.checked
  }).then(registerResponse).catch(registerResponse)
})

document.getElementById('save-dialog').addEventListener('click', function () {
  window.tauri.saveDialog({
    defaultPath: defaultPathInput.value || null,
    filter: filterInput.value || null
  }).then(registerResponse).catch(registerResponse)
})
