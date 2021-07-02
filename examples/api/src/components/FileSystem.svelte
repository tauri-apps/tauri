<script>
  import { readBinaryFile, readDir, Dir } from "@tauri-apps/api/fs";
  import { convertFileSrc } from "@tauri-apps/api/tauri";

  export let onMessage;

  let pathToRead = "";
  let img;

  function getDir() {
    const dirSelect = document.getElementById("dir");
    return dirSelect.value ? parseInt(dir.value) : null;
  }

  function arrayBufferToBase64(buffer, callback) {
    const blob = new Blob([buffer], {
      type: "application/octet-binary",
    });
    const reader = new FileReader();
    reader.onload = function (evt) {
      const dataurl = evt.target.result;
      callback(dataurl.substr(dataurl.indexOf(",") + 1));
    };
    reader.readAsDataURL(blob);
  }

  const DirOptions = Object.keys(Dir)
    .filter((key) => isNaN(parseInt(key)))
    .map((dir) => [dir, Dir[dir]]);

  function read() {
    const isFile = pathToRead.match(/\S+\.\S+$/g);
    const opts = {
      dir: getDir(),
    };
    const promise = isFile
      ? readBinaryFile(pathToRead, opts)
      : readDir(pathToRead, opts);
    promise
      .then(function (response) {
        if (isFile) {
          if (pathToRead.includes(".png") || pathToRead.includes(".jpg")) {
            arrayBufferToBase64(new Uint8Array(response), function (base64) {
              const src = "data:image/png;base64," + base64;
              onMessage('<img src="' + src + '"></img>');
            });
          } else {
            const value = String.fromCharCode.apply(null, response);
            onMessage(
              '<textarea id="file-response" style="height: 400px"></textarea><button id="file-save">Save</button>'
            );
            setTimeout(() => {
              const fileInput = document.getElementById("file-response");
              fileInput.value = value;
              document
                .getElementById("file-save")
                .addEventListener("click", function () {
                  writeFile(
                    {
                      file: pathToRead,
                      contents: fileInput.value,
                    },
                    {
                      dir: getDir(),
                    }
                  ).catch(onMessage);
                });
            });
          }
        } else {
          onMessage(response);
        }
      })
      .catch(onMessage);
  }

  function setSrc() {
    img.src = convertFileSrc(pathToRead)
  }
</script>

<form on:submit|preventDefault={read}>
  <select class="button" id="dir">
    <option value="">None</option>
    {#each DirOptions as dir}
      <option value={dir[1]}>{dir[0]}</option>
    {/each}
  </select>
  <input
    id="path-to-read"
    placeholder="Type the path to read..."
    bind:value={pathToRead}
  />
  <button class="button" id="read">Read</button>
  <button class="button" type="button" on:click={setSrc}>Use as img src</button>

  <img alt="file" bind:this={img}>
</form>
