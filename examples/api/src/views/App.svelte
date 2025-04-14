<script>
  import { show, hide, setTheme, setDockVisibility } from '@tauri-apps/api/app'

  let { onMessage } = $props()
  /** @type {import('@tauri-apps/api/window').Theme | 'auto'} */
  let theme = $state('auto')
  let dockVisible = $state(true)

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

  async function toggleDockVisibility() {
    await setDockVisibility(!dockVisible)
    dockVisible = !dockVisible
  }
</script>

<div>
  <button
    class="btn"
    id="show"
    title="Hides and shows the app after 2 seconds"
    onclick={showApp}>Show</button
  >
  <button class="btn" id="hide" onclick={hideApp}>Hide</button>
  <button class="btn" id="switch-theme" onclick={switchTheme}
    >Switch Theme ({theme})</button
  >
  <button class="btn" id="toggle-dock-visibility" onclick={toggleDockVisibility}>Toggle dock visibility</button>
</div>
