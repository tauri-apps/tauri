<script lang="ts">
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import { Channel, invoke } from '@tauri-apps/api/core'
  import { onMount, onDestroy } from 'svelte'
  import type { UnlistenFn } from '@tauri-apps/api/event'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  const webviewWindow = getCurrentWebviewWindow()

  let unlisten: UnlistenFn | undefined
  onMount(async () => {
    unlisten = await webviewWindow.listen('rust-event', onMessage)
  })
  onDestroy(() => {
    unlisten?.()
  })

  function log() {
    invoke('log_operation', {
      event: 'tauri-click',
      payload: 'this payload is optional because we used Option in Rust'
    })
  }

  function performRequest() {
    invoke('perform_request', {
      endpoint: 'dummy endpoint arg',
      body: {
        id: 5,
        name: 'test'
      }
    })
      .then(onMessage)
      .catch(onMessage)
  }

  function echo() {
    invoke('echo', {
      message: 'Tauri JSON request!'
    })
      .then(onMessage)
      .catch(onMessage)

    invoke('echo', [1, 2, 3]).then(onMessage).catch(onMessage)
  }

  function spam() {
    const channel = new Channel()
    channel.onmessage = onMessage
    invoke('spam', { channel })
  }

  function emitEvent() {
    webviewWindow.emit('js-event', 'this is the payload string')
  }
</script>

<div class="flex gap-2">
  <button class="btn" id="log" onclick={log}>Call Log API</button>
  <button class="btn" id="request" onclick={performRequest}>
    Call Request (async) API
  </button>
  <button class="btn" id="event" onclick={emitEvent}>
    Send event to Rust
  </button>
  <button class="btn" id="request" onclick={echo}>Echo</button>
  <button class="btn" id="request" onclick={spam}>Spam</button>
</div>
