<script>
  import * as api from '$api'

  let name = api.NAME

  function greet() {
    api.greet(name)
  }
</script>

<div>
  <input bind:value={name} />
  <button on:click={greet}>Greet</button>
</div>
