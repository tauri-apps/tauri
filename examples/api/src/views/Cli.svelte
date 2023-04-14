<script>
  import { invoke } from '@tauri-apps/api/tauri'

  export let onMessage

  function cliMatches() {
    invoke('plugin:cli|cli_matches').then(onMessage).catch(onMessage)
  }
</script>

<p>
  This binary can be run from the terminal and takes the following arguments:
  <code class="code-block flex flex-wrap my-2">
    <pre>
  --config &lt;PATH&gt;
  --theme &lt;light|dark|system&gt;
  --verbose</pre>
  </code>
  Additionally, it has a <code>update --background</code> subcommand.
</p>
<br />
<div class="note">
  Note that the arguments are only parsed, not implemented.
</div>
<br />
<br />
<button class="btn" id="cli-matches" on:click={cliMatches}>
  Get matches
</button>
