<script>
  import { writeText, readText } from '@tauri-apps/api/clipboard'

  export let onMessage
  let text = 'clipboard message'

  function write() {
    writeText(text)
      .then(() => {
        onMessage('Wrote to the clipboard')
      })
      .catch(onMessage)
  }

  function read() {
    readText()
      .then((contents) => {
        onMessage(`Clipboard contents: ${contents}`)
      })
      .catch(onMessage)
  }
</script>

<div class="flex gap-1">
  <input
    class="grow input"
    placeholder="Text to write to the clipboard"
    bind:value={text}
  />
  <button class="btn" type="button" on:click={write}>Write</button>
  <button class="btn" type="button" on:click={read}>Read</button>
</div>
