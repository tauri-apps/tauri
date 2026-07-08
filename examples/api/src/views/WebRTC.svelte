<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  let video: HTMLVideoElement
  let mediaStream: MediaStream | undefined

  const constraints: MediaStreamConstraints = {
    audio: true,
    video: true
  }

  function handleSuccess(stream: MediaStream) {
    const settings = stream.getTracks().map((track) => track.getSettings())
    onMessage(`Got streams: ${JSON.stringify(settings, null, 2)}`)
    const videoTracks = stream.getVideoTracks()
    onMessage(`Using video device: ${videoTracks[0]?.label ?? 'Unknown'}`)
    // @ts-expect-error
    window.stream = mediaStream // make variable available to browser console
    if (video) {
      video.srcObject = stream
    }
  }

  function handleError(error: unknown) {
    if (!(error instanceof DOMException)) {
      onMessage(`getUserMedia error: ${error}`)
      return
    }

    if (error.name === 'ConstraintNotSatisfiedError') {
      // const v = constraints.video
      // onMessage(
      //   `The resolution ${v.width.exact}x${v.height.exact} px is not supported by your device.`
      // )
      onMessage(
        `The constraints ${constraints} can not be satisified by your device.`
      )
    } else if (error.name === 'PermissionDeniedError') {
      onMessage(
        'Permissions have not been granted to use your camera and '
          + 'microphone, you need to allow the page access to your devices in '
          + 'order for the demo to work.'
      )
    }
    onMessage(`getUserMedia error: ${error}`)
  }

  onMount(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia(constraints)
      handleSuccess(stream)
    } catch (e) {
      handleError(e)
    }
  })

  onDestroy(() => {
    for (const track of mediaStream?.getTracks() ?? []) {
      track.stop()
    }
  })
</script>

<div class="flex flex-col gap-2">
  <div class="note-red grow">Not available for Linux</div>
  <video id="localVideo" autoplay playsinline bind:this={video}>
    <track kind="captions" />
  </video>
</div>
