<script>
  import { setTitle, open } from "@tauri-apps/api/window";

  let urlValue = "https://tauri.studio";

  function openUrl() {
    open(urlValue);
  }

  function setWindowTitle() {
    setTitle(urlValue);
  }
</script>

<form style="margin-top: 24px" on:submit|preventDefault={setWindowTitle}>
  <input id="title" value="Awesome Tauri Example!" />
  <button class="button" type="submit">Set title</button>
</form>
<form style="margin-top: 24px" on:submit|preventDefault={openUrl}>
  <input id="url" bind:value={urlValue} />
  <button class="button" id="open-url">
    Open URL
  </button>
</form>