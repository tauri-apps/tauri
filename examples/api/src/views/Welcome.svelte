<script>
  import { invoke } from '@tauri-apps/api/primitives'

  function contextMenu() {
    invoke('popup_context_menu')
  }
</script>

<div>
  <p>
    This is a demo of Tauri's API capabilities using the <code
      >@tauri-apps/api</code
    > package. It's used as the main validation app, serving as the test bed of our
    development process. In the future, this app will be used on Tauri's integration
    tests.
  </p>

  <button class="btn" on:click={contextMenu}>Context menu</button>
</div>
