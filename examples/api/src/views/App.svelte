<script>
  import { show, hide, setTheme } from '@tauri-apps/api/app'

  export let onMessage
  /** @type {import('@tauri-apps/api/window').Theme | 'auto'} */
  let theme = 'auto'

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

  async function switchTheme() {
    switch (theme) {
      case 'dark':
        theme = 'light'
        break
      case 'light':
        theme = 'auto'
        break
      case 'auto':
        theme = 'dark'
        break
    }
    setTheme(theme === 'auto' ? null : theme)
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
  <button class="btn" id="hide" on:click={switchTheme}>Switch Theme ({theme})</button>
</div>
