// listen our main events, triggered when a new update is available
window.tauri.listen("update-available", function (res) {
  // make our install button visible
  document.getElementById("updater-install").classList.toggle("hidden");
  // INSTALL UPDATE (click event)
  document.getElementById("updater-install").addEventListener(
    "click",
    function () {
      window.tauri.emit("updater-install");
    },
  );

  // listen our status changes
  window.tauri.listen("update-install-status", function (res) {
    console.log("UPDATE STATUS ", res);
  });

  // Send results in console
  console.log("New version available: ");
  console.log(res);
});
