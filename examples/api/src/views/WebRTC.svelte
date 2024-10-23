<script>
  import { onMount, onDestroy } from 'svelte'
  export let onMessage

  const constraints = (window.constraints = {
    audio: true,
    video: true
  })

  function handleSuccess(stream) {
    const video = document.querySelector('video')
    const videoTracks = stream.getVideoTracks()
    onMessage('Got stream with constraints:', constraints)
    onMessage(`Using video device: ${videoTracks[0].label}`)
    window.stream = stream // make variable available to browser console
    video.srcObject = stream
  }

  function handleError(error) {
    if (error.name === 'ConstraintNotSatisfiedError') {
      const v = constraints.video
      onMessage(
        `The resolution ${v.width.exact}x${v.height.exact} px is not supported by your device.`
      )
    } else if (error.name === 'PermissionDeniedError') {
      onMessage(
        'Permissions have not been granted to use your camera and ' +
          'microphone, you need to allow the page access to your devices in ' +
          'order for the demo to work.'
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
