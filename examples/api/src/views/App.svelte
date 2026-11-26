<script lang="ts">
  import {
    show,
    hide,
    setDockVisibility,
    openSettings,
    setKeepScreenOn,
    setPrivacyScreen
  } from '@tauri-apps/api/app'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()
  let dockVisible = $state(true)
  let keepScreenOn = $state(false)
  let privacyScreen = $state(false)

  const userAgent = navigator.userAgent.toLowerCase()
  const isMobile = userAgent.includes('android') || userAgent.includes('iphone')

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

  function openAppSettings() {
    openSettings()
      .then(() => onMessage('Opened app settings'))
      .catch(onMessage)
  }

  function toggleKeepScreenOn() {
    const enabled = !keepScreenOn
    setKeepScreenOn(enabled)
      .then(() => {
        keepScreenOn = enabled
        onMessage(`Keep screen on ${enabled ? 'enabled' : 'disabled'}`)
      })
      .catch(onMessage)
  }

  function togglePrivacyScreen() {
    const enabled = !privacyScreen
    setPrivacyScreen(enabled)
      .then(() => {
        privacyScreen = enabled
        onMessage(`Privacy screen ${enabled ? 'enabled' : 'disabled'}`)
      })
      .catch(onMessage)
  }
</script>

<div class="flex flex-wrap gap-2">
  {#if isMobile}
    <button class="btn" id="open-settings" onclick={openAppSettings}
      >Open app settings</button
    >
    <button class="btn" id="toggle-keep-screen-on" onclick={toggleKeepScreenOn}
      >{keepScreenOn ? 'Disable' : 'Enable'} keep screen on</button
    >
    <button class="btn" id="toggle-privacy-screen" onclick={togglePrivacyScreen}
      >{privacyScreen ? 'Disable' : 'Enable'} privacy screen</button
    >
  {:else}
    <button
      class="btn"
      id="show"
      title="Hides and shows the app after 2 seconds"
      onclick={showApp}>Show</button
    >
    <button class="btn" id="hide" onclick={hideApp}>Hide</button>
    <button
      class="btn"
      id="toggle-dock-visibility"
      onclick={toggleDockVisibility}>Toggle dock visibility</button
    >
  {/if}
</div>
