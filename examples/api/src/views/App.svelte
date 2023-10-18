<script>
  import { show, hide } from '@tauri-apps/api/app'

  export let onMessage

  function showApp() {
    hideApp()
      .then(() => {
        setTimeout(() => {
          show()
            .then(() => onMessage('Shown app'))
            .catch(onMessage)
        }, 2000)
      })
      .catch(onMessage)
  }

  function hideApp() {
    return hide()
      .then(() => onMessage('Hide app'))
      .catch(onMessage)
  }
</script>

<div>
  <button
    class="btn"
    id="show"
    title="Hides and shows the app after 2 seconds"
    on:click={showApp}>Show</button
  >
  <button class="btn" id="hide" on:click={hideApp}>Hide</button>
</div>
