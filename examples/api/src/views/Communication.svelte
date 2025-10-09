<script>
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  import { Channel, invoke } from '@tauri-apps/api/core'
  import { onMount, onDestroy } from 'svelte'

  let { onMessage } = $props()
  const unlisten = []

  const webviewWindow = getCurrentWebviewWindow()

  onMount(async () => {
    const unlistenFn1 = await webviewWindow.listen('rust-event', onMessage)
    unlisten.push(unlistenFn1)
    const unlistenFn2 = await webviewWindow.listen('raw-rust-event', (event) => {
      onMessage({
        event: event.event,
        id: event.id,
        payload: Array.from(new Uint8Array(event.payload)),
      })
    })
    unlisten.push(unlistenFn2)
  })
  onDestroy(() => {
    for (const unlistenFn of unlisten) {
      unlistenFn()
    }
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

  function emitRawEvent() {
    webviewWindow.emit('raw-js-event', new Uint8Array([1, 2, 3]))
  }
</script>

<div>
  <button class="btn" id="log" onclick={log}>Call Log API</button>
  <button class="btn" id="request" onclick={performRequest}>
    Call Request (async) API
  </button>
  <button class="btn" id="event" onclick={emitEvent}>
    Send event to Rust
  </button>
  <button class="btn" id="event" onclick={emitRawEvent}>
    Send raw event to Rust
  </button>
  <button class="btn" id="request" onclick={echo}> Echo </button>
  <button class="btn" id="request" onclick={spam}> Spam </button>
</div>
