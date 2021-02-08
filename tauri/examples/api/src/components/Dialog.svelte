<script>
  import {
    open,
    save
  } from '@tauri-apps/api/dialog'
  import {
    readBinaryFile
  } from '@tauri-apps/api/fs'

  export let onMessage
  let defaultPath = null
  let filter = null
  let multiple = false
  let directory = false

  function openDialog() {
    open({
      defaultPath: defaultPath,
      filter: filter,
      multiple: multiple,
      directory: directory
    }).then(function (res) {
      var pathToRead = res
      var isFile = pathToRead.match(/\S+\.\S+$/g)
      readBinaryFile(pathToRead).then(function (response) {
        if (isFile) {
          if (pathToRead.includes('.png') || pathToRead.includes('.jpg')) {
            arrayBufferToBase64(new Uint8Array(response), function (base64) {
              var src = 'data:image/png;base64,' + base64
              onMessage('<img src="' + src + '"></img>')
            })
          } else {
            onMessage(res)
          }
        } else {
          onMessage(res)
        }
      }).catch(onMessage(res))
    }).catch(onMessage)
  }

  function saveDialog() {
    save({
      defaultPath: defaultPath,
      filter: filter,
    }).then(onMessage).catch(onMessage)
  }

</script>

<div style="margin-top: 24px">
  <input id="dialog-default-path" placeholder="Default path" bind:value={defaultPath} />
  <input id="dialog-filter" placeholder="Extensions filter" bind:value={filter} />
  <div>
    <input type="checkbox" id="dialog-multiple" bind:checked={multiple} />
    <label for="dialog-multiple">Multiple</label>
  </div>
  <div>
    <input type="checkbox" id="dialog-directory" bind:checked={directory} />
    <label for="dialog-directory">Directory</label>
  </div>

  <button class="button" id="open-dialog" on:click={openDialog}>Open dialog</button>
  <button class="button" id="save-dialog" on:click={saveDialog}>Open save dialog</button>
</div>