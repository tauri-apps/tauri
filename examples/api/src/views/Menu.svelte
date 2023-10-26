<script>
  import { Menu, Submenu } from '@tauri-apps/api/menu'
  import MenuBuilder from '../components/MenuBuilder.svelte'

  export let onMessage
  let items = []
  let menu = null
  let menuItemCount = 0

  const macOS = navigator.userAgent.includes('Macintosh')

  async function create() {
    const submenu = await Submenu.new({
      text: 'app',
      items: items.map((i) => i.item)
    })
    menuItemCount = items.length
    menu = await Menu.new({
      items: [submenu]
    })
    await (macOS ? menu.setAsAppMenu() : menu.setAsWindowMenu())
  }

  async function popup() {
    if (!menu || menuItemCount !== items.length) {
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
