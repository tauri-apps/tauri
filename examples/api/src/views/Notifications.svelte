<script>
  export let onMessage

  // send the notification directly
  // the backend is responsible for checking the permission
  function _sendNotification() {
    new Notification('Notification title', {
      body: 'This is the notification body'
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

<button class="btn" id="notification" on:click={_sendNotification}>
  Send test notification
</button>
