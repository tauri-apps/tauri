<script>
  import { getMatches } from "@tauri-apps/api/cli";

  export let onMessage;

  function cliMatches() {
    getMatches()
      .then(onMessage)
      .catch(onMessage);
  }
</script>

<div>
  <button class="button" id="cli-matches" on:click={cliMatches}>
    Get matches
  </button>
</div>