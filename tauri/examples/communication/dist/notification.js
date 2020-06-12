document.getElementById('notification').addEventListener('click', function () {
  new Notification({
    title: 'Notification title',
    body: 'This is the notification body'
  })
})
