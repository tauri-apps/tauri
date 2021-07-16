<script>
  import { getMatches } from "@tauri-apps/api/cli";

  export let onMessage;

  function cliMatches() {
    getMatches().then(onMessage).catch(onMessage);
  }
</script>

<div>
  This binary can be run on the terminal and takes the following arguments:
  <ul>
    <li>--config PATH</li>
    <li>--theme light|dark|system</li>
    <li>--verbose</li>
  </ul>
  Additionally, it has a <i>update --background</i> subcommand.
  Note that the arguments are only parsed, not implemented.
  <br>
  <button class="button" id="cli-matches" on:click={cliMatches}>
    Get matches
  </button>
</div>
