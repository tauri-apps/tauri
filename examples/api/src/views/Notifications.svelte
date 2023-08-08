<script>
  import { type } from '@tauri-apps/api/os'

  export let onMessage

  // send the notification directly
  // the backend is responsible for checking the permission
  async function _sendNotification() {
    const osType = await type()
    new Notification('Notification title', {
      body: 'This is the notification body',
      sound:
        osType === 'Windows_NT'
          ? 'Default'
          : osType === 'Linux'
          ? 'dialog-information'
          : osType === 'Darwin'
          ? 'NSUserNotificationDefaultSoundName'
          : undefined
    })
  }

  // alternatively, check the permission ourselves
  function sendNotification() {
    if (Notification.permission === 'default') {
      Notification.requestPermission()
        .then(function (response) {
          if (response === 'granted') {
            _sendNotification()
          } else {
            onMessage('Permission is ' + response)
          }
        })
        .catch(onMessage)
    } else if (Notification.permission === 'granted') {
      _sendNotification()
    } else {
      onMessage('Permission is denied')
    }
  }
</script>

<button class="btn" id="notification" on:click={sendNotification}>
  Send test notification
</button>
