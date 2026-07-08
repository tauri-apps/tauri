<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import type { MessageHandler } from '../types'

  let { onMessage }: { onMessage: MessageHandler } = $props()

  const constraints: MediaStreamConstraints = (window.constraints = {
    audio: true,
    video: true
  })

  function handleSuccess(stream: MediaStream) {
    const video = document.querySelector<HTMLVideoElement>('video')
    const videoTracks = stream.getVideoTracks()
    onMessage('Got stream with constraints:', constraints)
    onMessage(`Using video device: ${videoTracks[0]?.label ?? 'Unknown'}`)
    window.stream = stream // make variable available to browser console
    if (video) {
      video.srcObject = stream
    }
  }

  function handleError(error: unknown) {
    if (!(error instanceof DOMException)) {
      onMessage('getUserMedia error:', error)
      return
    }

    if (error.name === 'ConstraintNotSatisfiedError') {
      const v = constraints.video
      const exact =
        typeof v === 'object' && 'width' in v && 'height' in v
          ? {
              width:
                typeof v.width === 'object' && 'exact' in v.width
                  ? v.width.exact
                  : undefined,
              height:
                typeof v.height === 'object' && 'exact' in v.height
                  ? v.height.exact
                  : undefined
            }
          : undefined
      onMessage(
        `The resolution ${exact?.width ?? 'requested'}x${exact?.height ?? 'requested'} px is not supported by your device.`
      )
    } else if (error.name === 'PermissionDeniedError') {
      onMessage(
        'Permissions have not been granted to use your camera and '
          + 'microphone, you need to allow the page access to your devices in '
          + 'order for the demo to work.'
      )
    }
    onMessage(`getUserMedia error: ${error.name}`, error)
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
    window.stream?.getTracks().forEach(function (track) {
      track.stop()
    })
  })
</script>

<div class="flex flex-col gap-2">
  <div class="note-red grow">Not available for Linux</div>
  <video id="localVideo" autoplay playsinline>
    <track kind="captions" />
  </video>
</div>
