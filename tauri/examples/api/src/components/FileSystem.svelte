<script>
  import {
    readBinaryFile,
    readDir,
    Dir,
  } from '@tauri-apps/api/fs'

  export let onMessage

  function getDir() {
    var dirSelect = document.getElementById('dir')
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

  const DirOptions = Object.keys(Dir)
    .filter(key => isNaN(parseInt(key)))
    .map(dir => [dir, Dir[dir]]);

  function read() {
    var pathToRead = pathInput.value
    var isFile = pathToRead.match(/\S+\.\S+$/g)
    var opts = {
      dir: getDir()
    }
    var promise = isFile ? readBinaryFile(pathToRead, opts) : readDir(pathToRead, opts)
    promise.then(function (response) {
      if (isFile) {
        if (pathToRead.includes('.png') || pathToRead.includes('.jpg')) {
          arrayBufferToBase64(new Uint8Array(response), function (base64) {
            var src = 'data:image/png;base64,' + base64
            onMessage('<img src="' + src + '"></img>')
          })
        } else {
          var value = String.fromCharCode.apply(null, response)
          onMessage('<textarea id="file-response" style="height: 400px"></textarea><button id="file-save">Save</button>')
          var fileInput = document.getElementById('file-response')
          fileInput.value = value
          document.getElementById('file-save').addEventListener('click', function () {
            writeFile({
              file: pathToRead,
              contents: fileInput.value
            }, {
              dir: getDir()
            }).catch(onMessage)
          })
        }
      } else {
        onMessage(response)
      }
    }).catch(onMessage)
  }
</script>

<form style="margin-top: 24px" on:submit|preventDefault={read}>
  <select class="button" id="dir">
    <option value="">None</option>
    {#each DirOptions as dir}
    <option value={dir[1]}>{dir[0]}</option>
    {/each}
  </select>
  <input id="path-to-read" placeholder="Type the path to read..." />
  <button class="button" id="read">Read</button>
</form>