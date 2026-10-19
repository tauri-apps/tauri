<script lang="ts">
  import { Menu, Submenu, NativeIcon } from '@tauri-apps/api/menu'
  import MenuBuilder, {
    reorderMenuItems,
    type Item,
    type MenuItemClickDetail,
    type MenuItems
  } from '../components/MenuBuilder.svelte'
  import { defaultWindowIcon } from '@tauri-apps/api/app'
  import type { ViewProps } from '../App.svelte'
  import { onDestroy } from 'svelte'
  import type { Image } from '@tauri-apps/api/image'

  let { onMessage }: ViewProps = $props()
  let items = $state<Item[]>([])

  let menu: Menu | undefined
  let popupMenu: Menu | undefined
  let submenu: Submenu | undefined

  const macOS = navigator.userAgent.includes('Macintosh')

  async function createSubmenu(): Promise<Submenu> {
    return await Submenu.new({
      text: 'app',
      items: items.map((i) => i.menu).filter(Boolean) as MenuItems[]
    })
  }

  async function createSubmenuWithNativeIcon(): Promise<Submenu> {
    return await Submenu.new({
      text: 'Submenu with NativeIcon',
      icon: NativeIcon.Folder,
      items: items.map((i) => i.menu).filter(Boolean) as MenuItems[]
    })
  }

  async function createSubmenuWithImageIcon(): Promise<Submenu> {
    let icon: Image | undefined
    try {
      icon = (await defaultWindowIcon())!
      return await Submenu.new({
        text: 'Submenu with Image',
        icon,
        items: items.map((i) => i.menu).filter(Boolean) as MenuItems[]
      })
    } finally {
      icon?.close()
    }
  }

  async function setMenu(newSubmenu: Submenu) {
    menu?.close()
    menu = undefined
    submenu?.close()
    submenu = newSubmenu

    menu = await Menu.new({
      items: [submenu]
    })
    await (macOS ? menu.setAsAppMenu() : menu.setAsWindowMenu())
  }

  async function create() {
    await setMenu(await createSubmenu())
  }

  async function createWithNativeIcon() {
    await setMenu(await createSubmenuWithNativeIcon())
  }

  async function createWithImageIcon() {
    await setMenu(await createSubmenuWithImageIcon())
  }

  async function popup() {
    popupMenu?.close()
    popupMenu = undefined
    popupMenu = await Menu.new({
      items: items.map((i) => i.menu).filter(Boolean) as MenuItems[]
    })
    await popupMenu.popup()
  }

  function onItemClick(detail: MenuItemClickDetail) {
    onMessage(`Item ${detail.text} clicked`)
  }

  onDestroy(() => {
    menu?.close()
    submenu?.close()
    popupMenu?.close()
  })
</script>

<div class="grid gap-8 mb-4">
  <MenuBuilder
    bind:items
    itemClick={onItemClick}
    onItemAdded={async (item) => {
      if (item.menu) {
        await submenu?.append(item.menu)
      }
    }}
    onItemRemoved={async (item) => {
      if (item.menu) {
        await submenu?.remove(item.menu)
      }
    }}
    onItemMoved={(item, toIndex) => {
      if (submenu) {
        reorderMenuItems(submenu, item, toIndex)
      }
    }}
  />
  <div class="flex gap-2">
    <button class="btn" onclick={create}>Create menu</button>
    <button class="btn" onclick={createWithNativeIcon}
      >Create menu with NativeIcon</button
    >
    <button class="btn" onclick={createWithImageIcon}
      >Create menu with Image icon</button
    >
    <button class="btn" onclick={popup}>Popup</button>
  </div>
</div>
