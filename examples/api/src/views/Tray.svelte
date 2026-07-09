<script lang="ts">
  import { TrayIcon } from '@tauri-apps/api/tray'
  import MenuBuilder, {
    type Item,
    type MenuItemClickDetail
  } from '../components/MenuBuilder.svelte'
  import { Menu } from '@tauri-apps/api/menu'
  import type { ViewProps } from '../App.svelte'

  let { onMessage }: ViewProps = $props()

  let icon = $state<string>()
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

<div class="flex flex-col children:grow gap-2">
  <div class="flex gap-1">
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
      <input type="checkbox" class="checkbox" bind:checked={menuOnLeftClick} />
      Menu on left click
    </label>
  </div>

  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Icon path"
      bind:value={icon}
    />

    <label>
      <input type="checkbox" class="checkbox" bind:checked={iconAsTemplate} />
      Icon as template
    </label>
  </div>

  <div class="flex children:grow">
    <MenuBuilder bind:items={menuItems} itemClick={onItemClick} />
  </div>

  <div class="flex">
    <button class="btn" onclick={create} title="Creates the tray icon"
      >Create tray</button
    >
  </div>
</div>
