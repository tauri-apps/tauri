<script lang="ts">
  import { onMount } from 'svelte'
  import { convertFileSrc, invoke } from '@tauri-apps/api/core'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  type AssetMedia = {
    // file name -> absolute path, null until the files are downloaded
    files: Record<string, string> | null
    out_of_scope: string
  }

  let media = $state<AssetMedia | null>(null)
  let downloading = $state(false)

  // `convertFileSrc` turns a file path into an URL served by the asset protocol:
  // `asset://localhost/<path>` or `http://asset.localhost/<path>` depending on the platform.
  const sources = $derived(
    media?.files
      ? Object.fromEntries(
          Object.entries(media.files).map(([name, path]) => [
            name,
            convertFileSrc(path)
          ])
        )
      : null
  )

  async function loadMedia(download: boolean) {
    downloading = download
    try {
      media = await invoke<AssetMedia>('asset_media', { download })
      if (download) {
        onMessage(media.files)
      }
    } catch (e) {
      onMessage(e)
    } finally {
      downloading = false
    }
  }

  // The asset protocol answers `Range` requests, which is what lets the
  // webview seek in audio and video files without loading them entirely.
  async function fetchRange() {
    if (!sources) return
    try {
      const response = await fetch(sources['video.mp4'], {
        headers: { Range: 'bytes=0-1023' }
      })
      const body = await response.arrayBuffer()
      onMessage({
        status: response.status,
        contentType: response.headers.get('content-type'),
        contentRange: response.headers.get('content-range'),
        bytes: body.byteLength
      })
    } catch (e) {
      onMessage(`fetch failed: ${e}`)
    }
  }

  // Paths that `app > security > assetProtocol > scope` does not allow are rejected.
  async function fetchOutOfScope() {
    if (!media) return
    try {
      const response = await fetch(convertFileSrc(media.out_of_scope))
      onMessage({ path: media.out_of_scope, status: response.status })
    } catch (e) {
      onMessage(`fetch failed: ${e}`)
    }
  }

  function onMediaEvent(event: Event) {
    const element = event.currentTarget as HTMLMediaElement | HTMLImageElement
    // the console renders HTML, so the tag name is logged without the angle brackets
    const tag = element.tagName.toLowerCase()
    if (event.type === 'error') {
      const error = 'error' in element ? element.error : null
      const reason = error ? `: ${error.message || `code ${error.code}`}` : ''
      onMessage(`${tag} failed to load${reason}`)
    } else if (element instanceof HTMLMediaElement) {
      onMessage(`${tag} loaded, duration ${element.duration.toFixed(1)}s`)
    } else {
      onMessage(
        `${tag} loaded, ${element.naturalWidth}x${element.naturalHeight}`
      )
    }
  }

  onMount(() => loadMedia(false))
</script>

<div class="flex flex-col gap-4">
  <p class="note">
    The asset protocol serves files from disk to the webview. The files below
    are downloaded to the app cache directory, which the asset protocol scope
    allows.
  </p>

  {#if !sources}
    <div>
      <button
        class="btn"
        id="download-media"
        disabled={downloading || !media}
        onclick={() => loadMedia(true)}
      >
        {downloading ? 'Downloading...' : 'Download media files'}
      </button>
    </div>
  {:else}
    <div class="flex flex-wrap gap-4">
      <div class="flex flex-col gap-2">
        <h4 class="my-0">Image</h4>
        <img
          id="asset-image"
          class="max-w-60 rd-1"
          src={sources['image.jpg']}
          alt="A grapefruit slice"
          onload={onMediaEvent}
          onerror={onMediaEvent}
        />
      </div>

      <div class="flex flex-col gap-2">
        <h4 class="my-0">Video</h4>
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
          id="asset-video"
          class="max-w-100 rd-1"
          controls
          playsinline
          onloadedmetadata={onMediaEvent}
          onerror={onMediaEvent}
        >
          <source
            src={sources['video.mp4']}
            type="video/mp4"
            onerror={() => onMessage('video.mp4 source failed to load')}
          />
          <source
            src={sources['video.webm']}
            type="video/webm"
            onerror={() => onMessage('video.webm source failed to load')}
          />
        </video>
      </div>

      <div class="flex flex-col gap-2">
        <h4 class="my-0">Audio</h4>
        <audio
          id="asset-audio"
          controls
          src={sources['audio.mp3']}
          onloadedmetadata={onMediaEvent}
          onerror={onMediaEvent}
        ></audio>
      </div>
    </div>

    <div class="flex flex-wrap gap-2">
      <button class="btn" id="fetch-range" onclick={fetchRange}>
        Fetch a byte range
      </button>
      <button class="btn" id="fetch-out-of-scope" onclick={fetchOutOfScope}>
        Fetch a file outside the scope
      </button>
    </div>
  {/if}
</div>
