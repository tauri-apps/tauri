<script>
  import { setTitle, open } from "@tauri-apps/api/window";

  let urlValue = "https://tauri.studio";
  let windowTitle = 'Awesome Tauri Example!';

  function openUrl() {
    open(urlValue);
  }

  function setWindowTitle() {
    setTitle(windowTitle);
  }
</script>

<form style="margin-top: 24px" on:submit|preventDefault={setWindowTitle}>
  <input id="title" bind:value={windowTitle} />
  <button class="button" type="submit">Set title</button>
</form>
<form style="margin-top: 24px" on:submit|preventDefault={openUrl}>
  <input id="url" bind:value={urlValue} />
  <button class="button" id="open-url">
    Open URL
  </button>
</form>