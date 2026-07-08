<script lang="ts">
  import { show, hide, setDockVisibility } from '@tauri-apps/api/app'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()
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

  async function toggleDockVisibility() {
    await setDockVisibility(!dockVisible)
    dockVisible = !dockVisible
  }
</script>

<div class="flex gap-2">
  <button
    class="btn"
    id="show"
    title="Hides and shows the app after 2 seconds"
    onclick={showApp}>Show</button
  >
  <button class="btn" id="hide" onclick={hideApp}>Hide</button>
  <button class="btn" id="toggle-dock-visibility" onclick={toggleDockVisibility}
    >Toggle dock visibility</button
  >
</div>
