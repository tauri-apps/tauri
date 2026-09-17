<script lang="ts">
  import { TrayIcon } from '@tauri-apps/api/tray'
  import MenuBuilder, {
    reorderMenuItems,
    type Item,
    type MenuItemClickDetail,
    type MenuItems
  } from '../components/MenuBuilder.svelte'
  import { Menu } from '@tauri-apps/api/menu'
  import type { ViewProps } from '../App.svelte'
  import { onDestroy } from 'svelte'

  let { onMessage }: ViewProps = $props()

  let icon = $state<string>('../../.icons/tray_icon.png')
  let tooltip = $state<string>()
  let title = $state<string>()
  let iconAsTemplate = $state(false)
  let menuOnLeftClick = $state(true)
  let menuItems = $state<Item[]>([])

  let menu: Menu | undefined
  let tray = $state<TrayIcon | undefined>()

  function onItemClick(detail: MenuItemClickDetail) {
    onMessage(`Item ${detail.text} clicked`)
  }

  async function create() {
    try {
      menu = await Menu.new({
        items: menuItems.map((i) => i.menu).filter(Boolean) as MenuItems[]
      })
      tray = await TrayIcon.new({
        icon,
        tooltip,
        title,
        iconAsTemplate,
        menuOnLeftClick,
        menu,
        action: (event) => onMessage(event)
      })
    } catch (error) {
      menu?.close()
      menu = undefined
      tray?.close()
      tray = undefined
      onMessage(error)
    }
  }

  onDestroy(() => {
    menu?.close()
    tray?.close()
  })
</script>

<div class="grid gap-8 mb-4">
  <MenuBuilder
    bind:items={menuItems}
    itemClick={onItemClick}
    onItemAdded={async (item) => {
      if (item.menu) {
        await menu?.append(item.menu)
      }
    }}
    onItemRemoved={async (item) => {
      if (item.menu) {
        await menu?.remove(item.menu)
      }
    }}
    onItemMoved={(item, toIndex) => {
      if (menu) {
        reorderMenuItems(menu, item, toIndex)
      }
    }}
  />

  <div class="flex items-center gap-8">
    <div class="grid gap-2 grid-cols-3 items-center">
      <input
        class="input grow"
        type="text"
        placeholder="Title"
        bind:value={title}
      />

      <input
        class="input grow"
        type="text"
        placeholder="Tooltip"
        bind:value={tooltip}
      />

      <label class="flex gap-2">
        <input
          type="checkbox"
          class="checkbox"
          bind:checked={menuOnLeftClick}
        />
        Menu on left click
      </label>

      <input
        class="input col-span-2"
        type="text"
        placeholder="Icon path"
        bind:value={icon}
      />

      <label class="flex gap-2">
        <input type="checkbox" class="checkbox" bind:checked={iconAsTemplate} />
        Icon as template
      </label>
    </div>

    <div class="flex">
      {#if tray}
        <button
          class="btn"
          onclick={() => {
            tray?.close()
            tray = undefined
          }}
          title="Remove the tray icon">Remove tray</button
        >
      {:else}
        <button class="btn" onclick={create} title="Creates the tray icon"
          >Create tray</button
        >
      {/if}
    </div>
  </div>
</div>
