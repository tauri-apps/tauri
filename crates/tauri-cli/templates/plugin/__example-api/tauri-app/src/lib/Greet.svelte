<script>
  import { invoke } from "@tauri-apps/api/core"

  let name = $state("");
  let greetMsg = $state("")

  async function greet(){
    // Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/#commands
    greetMsg = await invoke("greet", { name })
  }
</script>

<div>
  <div class="row">
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button onclick={greet}>
      Greet
    </button>
  </div>
  <p>{greetMsg}</p>
</div>

