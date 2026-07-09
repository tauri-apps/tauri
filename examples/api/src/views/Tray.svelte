<script lang="ts">
  import { TrayIcon } from '@tauri-apps/api/tray'
  import MenuBuilder, {
    type Item,
    type MenuItemClickDetail
  } from '../components/MenuBuilder.svelte'
  import { Menu } from '@tauri-apps/api/menu'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  let icon = $state<string>('../../.icons/tray_icon.png')
  let tooltip = $state<string>()
  let title = $state<string>()
  let iconAsTemplate = $state(false)
  let menuOnLeftClick = $state(true)
  let menuItems = $state<Item[]>([])

  function onItemClick(detail: MenuItemClickDetail) {
    onMessage(`Item ${detail.text} clicked`)
  }

  async function create() {
    TrayIcon.new({
      icon,
      tooltip,
      title,
      iconAsTemplate,
      menuOnLeftClick,
      menu: await Menu.new({
        items: menuItems.map((i) => i.menu).filter(Boolean) as NonNullable<
          Item['menu']
        >[]
      }),
      action: (event) => onMessage(event)
    }).catch(onMessage)
  }
</script>

<div class="grid gap-8 mb-4">
  <MenuBuilder bind:items={menuItems} itemClick={onItemClick} />

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

      <label>
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

      <label>
        <input type="checkbox" class="checkbox" bind:checked={iconAsTemplate} />
        Icon as template
      </label>
    </div>

    <div class="flex">
      <button class="btn" onclick={create} title="Creates the tray icon"
        >Create tray</button
      >
    </div>
  </div>
</div>
