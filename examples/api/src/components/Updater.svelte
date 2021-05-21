<script>
  import { onMount, onDestroy } from "svelte";
  
  // This example show how updater events work when dialog is disabled.
  // This allow you to use custom dialog for the updater.
  // This is your responsability to restart the application after you receive the STATUS: DONE.

  import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
  import { listen } from "@tauri-apps/api/event";
  import { relaunch } from "@tauri-apps/api/process";

  export let onMessage;
  let unlisten;

  onMount(async () => {
    unlisten = await listen("tauri://update-status", onMessage)
  })
  onDestroy(() => {
    if (unlisten) {
      unlisten()
    }
  })

  async function check() {
    try {
      document.getElementById("check_update").classList.add("hidden");

      const {shouldUpdate, manifest} = await checkUpdate();
      onMessage(`Should update: ${shouldUpdate}`);
      onMessage(manifest);

      if (shouldUpdate) {
        document.getElementById("start_update").classList.remove("hidden");
      }
    } catch(e) {
      onMessage(e);
    }
  }

  async function install() {
    try {
      document.getElementById("start_update").classList.add("hidden");

      await installUpdate();
      onMessage("Installation complete, restart required.");
      await relaunch();

    } catch(e) {
      onMessage(e);
    }
  }


</script>

<div>
  <button class="button" id="check_update" on:click={check}>Check update</button>
  <button class="button hidden" id="start_update" on:click={install}>Install update</button>
</div>
