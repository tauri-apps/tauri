<script>
  import { writable } from 'svelte/store'
  import { registerShortcut, unregisterShortcut } from '@tauri-apps/api/globalShortcut'

  export let onMessage
  const shortcuts = writable([])
  let shortcut = 'CTRL+X'

  function register() {
    const shortcut_ = shortcut
    registerShortcut(shortcut_, () => {
      onMessage(`Shortcut ${shortcut_} triggered`)
    }).then(() => {
      shortcuts.update(shortcuts_ => [...shortcuts_, shortcut_])
      onMessage(`Shortcut ${shortcut_} registered successfully`)
    }).catch(onMessage)
  }

  function unregister(shortcut) {
    const shortcut_ = shortcut
    unregisterShortcut(shortcut_).then(() => {
      shortcuts.update(shortcuts_ => shortcuts_.filter(s => s !== shortcut_))
      onMessage(`Shortcut ${shortcut_} unregistered`)
    }).catch(onMessage)
  }
</script>

<div style="margin-top: 24px">
  <div>
    <input placeholder="Type a shortcut with '+' as separator..." bind:value={shortcut}>
    <button type="button" on:click={register}>Register</button>
  </div>
  <div>
    {#each $shortcuts as savedShortcut}
    <div>
      {savedShortcut}
      <button type="button" on:click={()=> unregister(savedShortcut)}>Unregister</button>
    </div>
    {/each}
  </div>
</div>