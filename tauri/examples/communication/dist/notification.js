function sendNotification() {
  new Notification("Notification title", {
    body: "This is the notification body",
  });
}

document.getElementById("notification").addEventListener("click", function () {
  if (Notification.permission === "default") {
    Notification.requestPermission()
      .then(function (response) {
        if (response === "granted") {
          sendNotification();
        } else {
          registerResponse("Permission is " + response);
        }
      })
      .catch(registerResponse);
  } else if (Notification.permission === "granted") {
    sendNotification();
  } else {
    registerResponse("Permission is denied");
  }
});
