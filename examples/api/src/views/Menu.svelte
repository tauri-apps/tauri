<script>
  import { Menu } from '@tauri-apps/api/menu'
  import MenuBuilder from '../components/MenuBuilder.svelte'

  export let onMessage
  let items = []
  let menu = null

  async function create() {
    menu = await Menu.new({
      items: items.map((i) => i.item)
    })
    await menu.setAsWindowMenu()
  }

  async function popup() {
    if (!menu) {
      await create()
    }
    menu.popup()
  }

  function onItemClick(event) {
    onMessage(`Item ${event.detail.text} clicked`)
  }
</script>

<div>
  <MenuBuilder bind:items on:itemClick={onItemClick} />
  <button class="btn" on:click={create}>Create menu</button>
  <button class="btn" on:click={popup}>Popup</button>
</div>
