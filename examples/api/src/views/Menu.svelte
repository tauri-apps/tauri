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

  function onItemClick(event) {
    onMessage(`Item ${event.detail.text} clicked`)
  }
</script>

<div>
  <MenuBuilder bind:items on:itemClick={onItemClick} />
  <button class="btn" on:click={create}>Create menu</button>
</div>
