<script>
  import { Menu } from '@tauri-apps/api/menu'
  import MenuBuilder from '../components/MenuBuilder.svelte'

  export let onMessage
  let items = []

  async function create() {
    const menu = await Menu.new({
      items: items.map((i) => i.item)
    })
    await menu.setAsWindowMenu('main')
  }
</script>

<div>
  <MenuBuilder bind:items />
  <button class="btn" on:click={create}>Create menu</button>
</div>
