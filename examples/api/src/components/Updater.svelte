<script>

  // This example show how updater events work when dialog is disabled.
  // This allow you to use custom dialog for the updater.
  // This is your responsability to restart the application after you receive the STATUS: DONE.

  import { listen, emit } from "@tauri-apps/api/event";

  export let onMessage;

  listen("tauri://update-available", onUpdateAvailable);
  listen("tauri://update-status", onMessage);

  function checkUpdate() {
    document.getElementById("check_update").disabled = true;
    emit("tauri://update");
  }

  function installUpdate() {
    emit("tauri://update-install");
  }

  function onUpdateAvailable(data) {
    onMessage(data);
    const checkUpdateButton = document.getElementById("check_update");
    const startUpdateButton = document.getElementById("start_update");

    checkUpdateButton.classList.add("hidden");
    startUpdateButton.classList.remove("hidden");
  }

</script>

<div>
  <button class="button" id="check_update" on:click={checkUpdate}>Check update</button>
  <button class="button hidden" id="start_update" on:click={installUpdate}>Install update</button>
</div>
