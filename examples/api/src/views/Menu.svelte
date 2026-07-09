<script lang="ts">
  import { Menu, Submenu, NativeIcon } from '@tauri-apps/api/menu'
  import MenuBuilder, {
    type Item,
    type MenuItemClickDetail
  } from '../components/MenuBuilder.svelte'
  import { defaultWindowIcon } from '@tauri-apps/api/app'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()
  let items = $state<Item[]>([])
  let menu = $state<Menu | null>(null)
  let submenu = $state<Submenu | null>(null)
  let menuItemCount = 0

  const macOS = navigator.userAgent.includes('Macintosh')

  async function createSubmenu(): Promise<Submenu> {
    submenu = await Submenu.new({
      text: 'app',
      items: items.map((i) => i.menu).filter(Boolean) as NonNullable<
        Item['menu']
      >[]
    })
    return submenu
  }

  async function createSubmenuWithNativeIcon(): Promise<Submenu> {
    submenu = await Submenu.new({
      text: 'Submenu with NativeIcon',
      icon: NativeIcon.Folder,
      items: items.map((i) => i.menu).filter(Boolean) as NonNullable<
        Item['menu']
      >[]
    })
    return submenu
  }

  async function createSubmenuWithImageIcon(): Promise<Submenu> {
    const icon = await defaultWindowIcon()
    submenu = await Submenu.new({
      text: 'Submenu with Image',
      ...(icon ? { icon } : {}),
      items: items.map((i) => i.menu).filter(Boolean) as NonNullable<
        Item['menu']
      >[]
    })
    return submenu
  }

  async function setMenu(submenu: Submenu) {
    menuItemCount = items.length
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
    if (!submenu || menuItemCount !== items.length) {
      await createSubmenu()
    }
    if (submenu) {
      // we can't popup the same menu because it's the app menu (it crashes on macOS)
      const m = await Menu.new({ items: [submenu] })
      m.popup()
    }
  }

  function onItemClick(detail: MenuItemClickDetail) {
    onMessage(`Item ${detail.text} clicked`)
  }
</script>

<div class="grid gap-4 mb-4">
  <MenuBuilder bind:items itemClick={onItemClick} />
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
