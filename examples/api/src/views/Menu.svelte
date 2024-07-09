<script>
  import { Menu, Submenu } from '@tauri-apps/api/menu'
  import MenuBuilder from '../components/MenuBuilder.svelte'

  export let onMessage
  let items = []
  let menu = null
  let submenu = null
  let menuItemCount = 0

  const macOS = navigator.userAgent.includes('Macintosh')

  async function createSubmenu() {
    submenu = await Submenu.new({
      text: 'app',
      items: items.map((i) => i.item)
    })
  }

  async function create() {
    await createSubmenu()
    menuItemCount = items.length
    menu = await Menu.new({
      items: [submenu]
    })
    await (macOS ? menu.setAsAppMenu() : menu.setAsWindowMenu())
  }

  async function popup() {
    if (!submenu || menuItemCount !== items.length) {
      await createSubmenu()
    }
    // we can't popup the same menu because it's the app menu (it crashes on macOS)
    const m = await Menu.new({ items: [submenu] })
    m.popup()
  }

  function onItemClick(event) {
    onMessage(`Item ${event.detail.text} clicked`)
  }
</script>

<div class="grid gap-4">
  <MenuBuilder bind:items on:itemClick={onItemClick} />
  <div>
    <button class="btn" on:click={create}>Create menu</button>
    <button class="btn" on:click={popup}>Popup</button>
  </div>
</div>
