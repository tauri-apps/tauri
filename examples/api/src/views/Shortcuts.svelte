<script>
  import { writable } from 'svelte/store'
  import {
    register as registerShortcut,
    unregister as unregisterShortcut,
    unregisterAll as unregisterAllShortcuts
  } from '@tauri-apps/api/globalShortcut'

  export let onMessage
  const shortcuts = writable([])
  let shortcut = 'CmdOrControl+X'

  function register() {
    const shortcut_ = shortcut
    registerShortcut(shortcut_, () => {
      onMessage(`Shortcut ${shortcut_} triggered`)
    })
      .then(() => {
        shortcuts.update((shortcuts_) => [...shortcuts_, shortcut_])
        onMessage(`Shortcut ${shortcut_} registered successfully`)
      })
      .catch(onMessage)
  }

  function unregister(shortcut) {
    const shortcut_ = shortcut
    unregisterShortcut(shortcut_)
      .then(() => {
        shortcuts.update((shortcuts_) =>
          shortcuts_.filter((s) => s !== shortcut_)
        )
        onMessage(`Shortcut ${shortcut_} unregistered`)
      })
      .catch(onMessage)
  }

  function unregisterAll() {
    unregisterAllShortcuts()
      .then(() => {
        shortcuts.update(() => [])
        onMessage(`Unregistered all shortcuts`)
      })
      .catch(onMessage)
  }
</script>

<div class="flex gap-1">
  <input
    class="input grow"
    placeholder="Type a shortcut with '+' as separator..."
    bind:value={shortcut}
  />
  <button class="btn" type="button" on:click={register}>Register</button>
</div>
<br />
<div class="flex flex-col gap-1">
  {#each $shortcuts as savedShortcut}
    <div class="flex justify-between">
      {savedShortcut}
      <button
        class="btn"
        type="button"
        on:click={() => unregister(savedShortcut)}>Unregister</button
      >
    </div>
  {/each}
  {#if $shortcuts.length > 1}
    <br />
    <button class="btn" type="button" on:click={unregisterAll}
      >Unregister all</button
    >
  {/if}
</div>
