<script>
  import { onMount, onDestroy } from 'svelte'

  // This example show how updater events work when dialog is disabled.
  // This allow you to use custom dialog for the updater.
  // This is your responsibility to restart the application after you receive the STATUS: DONE.

  import { checkUpdate, installUpdate } from '@tauri-apps/api/updater'
  import { listen } from '@tauri-apps/api/event'
  import { relaunch } from '@tauri-apps/api/process'

  export let onMessage
  let unlisten

  onMount(async () => {
    unlisten = await listen('tauri://update-status', onMessage)
  })
  onDestroy(() => {
    if (unlisten) {
      unlisten()
    }
  })

  let isChecking, isInstalling, newUpdate

  async function check() {
    isChecking = true
    try {
      const { shouldUpdate, manifest } = await checkUpdate()
      onMessage(`Should update: ${shouldUpdate}`)
      onMessage(manifest)

      newUpdate = shouldUpdate
    } catch (e) {
      onMessage(e)
    } finally {
      isChecking = false
    }
  }

  async function install() {
    isInstalling = true
    try {
      await installUpdate()
      onMessage('Installation complete, restart required.')
      await relaunch()
    } catch (e) {
      onMessage(e)
    } finally {
      isInstalling = false
    }
  }
</script>

<div class="flex children:grow children:h10">
  {#if !isChecking && !newUpdate}
    <button class="btn" on:click={check}>Check update</button>
  {:else if !isInstalling && newUpdate}
    <button class="btn" on:click={install}>Install update</button>
  {:else}
    <button
      class="btn text-accentText dark:text-darkAccentText flex items-center justify-center"
      ><div class="spinner animate-spin" /></button
    >
  {/if}
</div>

<style>
  .spinner {
    height: 1.2rem;
    width: 1.2rem;
    border-radius: 50rem;
    color: currentColor;
    border: 2px dashed currentColor;
  }
</style>
