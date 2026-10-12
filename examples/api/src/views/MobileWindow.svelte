<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { supportsMultipleWindows } from '@tauri-apps/api/app'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { Webview } from '@tauri-apps/api/webview'
  import {
    WebviewWindow,
    getAllWebviewWindows
  } from '@tauri-apps/api/webviewWindow'
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  // the window parts that work on mobile: on Android each window is a task of its own,
  // on iOS a scene; extra webviews in a window are laid out by activity embedding on Android
  const isAndroid = navigator.userAgent.toLowerCase().includes('android')
  const currentWindow = getCurrentWindow()

  let supported = $state<boolean | undefined>(undefined)
  let identity = $state('')
  let windowLabels = $state<string[]>([])
  let newWindowLabel = $state('')
  let webviewCount = 0

  async function refreshWindows() {
    windowLabels = (await getAllWebviewWindows()).map((w) => w.label)
  }

  function createWindow() {
    if (!newWindowLabel) return
    // `main-*` so the window gets the run-app capability
    const webview = new WebviewWindow(`main-${newWindowLabel}`)
    webview.once('tauri://error', (e) => {
      onMessage('Error creating new window ' + JSON.stringify(e))
    })
    webview.once('tauri://created', () => {
      onMessage(`window ${webview.label} created`)
      refreshWindows()
    })
  }

  async function closeWindow(label: string) {
    const w = await WebviewWindow.getByLabel(label)
    await w?.close().catch(onMessage)
    refreshWindows()
  }

  // two webviews in a new window; requires the `unstable` Cargo feature (`--features unstable`)
  function multiwebviewWindow() {
    invoke('spawn_multiwebview_window')
      .then((label) => {
        onMessage(`multiwebview window ${label} created`)
        refreshWindows()
      })
      .catch(onMessage)
  }

  // a webview next to this window's own; on Android it is hosted by `EmbeddedWebviewActivity`,
  // which packages/api-e2e/scripts/prepare-android.mjs adds to the Android project
  function addWebview() {
    const webview = new Webview(
      currentWindow,
      `${currentWindow.label}-child-${webviewCount++}`,
      {
        url: 'index.html',
        // ignored on Android, where the system lays the webviews out
        x: 0,
        y: 0,
        width: 400,
        height: 400,
        activityName: 'EmbeddedWebviewActivity'
      }
    )
    webview.once('tauri://error', (e) => {
      onMessage('Error adding webview ' + JSON.stringify(e))
    })
    webview.once('tauri://created', () => {
      onMessage(`webview ${webview.label} added`)
    })
    webview.once('tauri://destroyed', () => {
      onMessage(`webview ${webview.label} destroyed`)
    })
  }

  let unlistenCreated: UnlistenFn | undefined
  onMount(async () => {
    supported = await supportsMultipleWindows()
    identity = await (
      isAndroid ? currentWindow.activityName() : currentWindow.sceneIdentifier()
    ).catch(() => '')
    refreshWindows()
    unlistenCreated = await listen('tauri://window-created', refreshWindows)
  })

  onDestroy(() => {
    unlistenCreated?.()
  })
</script>

<div class="flex flex-col gap-8 mb-4">
  <div class="grid gap-1">
    <h4 class="my-2">This Window</h4>
    <span>
      Label: <code>{currentWindow.label}</code>
      {#if identity}
        <br />
        {isAndroid ? 'Activity' : 'Scene'}: <code>{identity}</code>
      {/if}
      <br />
      Supports multiple windows:
      <code>{supported === undefined ? '...' : supported}</code>
    </span>
  </div>

  <div class="grid gap-1">
    <h4 class="my-2">Create New Window</h4>
    <form
      class="flex gap-2"
      onsubmit={(ev) => {
        createWindow()
        ev.preventDefault()
      }}
    >
      <input
        class="input"
        type="text"
        placeholder="New window label.."
        bind:value={newWindowLabel}
      />
      <button class="btn" type="submit">Create</button>
    </form>
  </div>

  <div class="grid gap-1">
    <h4 class="my-2">Multiple Webviews</h4>
    <div class="flex flex-wrap gap-2">
      <button class="btn" id="multiwebview" onclick={multiwebviewWindow}>
        Multiwebview window
      </button>
      <button class="btn" onclick={addWebview}>Add webview to this window</button>
    </div>
  </div>

  <div class="grid gap-1">
    <h4 class="my-2">Windows</h4>
    <div class="flex flex-col gap-2">
      {#each windowLabels as label}
        <div class="flex items-center gap-2">
          <code class="grow">{label}</code>
          <button class="btn" onclick={() => closeWindow(label)}>Close</button>
        </div>
      {/each}
      <button class="btn justify-self-start" onclick={refreshWindows}>
        Refresh
      </button>
    </div>
  </div>
</div>
