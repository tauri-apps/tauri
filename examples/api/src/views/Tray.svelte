<script>
  import { TrayIcon } from '@tauri-apps/api/tray'
  import MenuBuilder from '../components/MenuBuilder.svelte'
  import { Menu } from '@tauri-apps/api/menu'

  export let onMessage

  let icon = null
  let tooltip = null
  let title = null
  let iconAsTemplate = false
  let menuOnLeftClick = true
  let menuItems = []

  function onItemClick(event) {
    onMessage(`Item ${event.detail.text} clicked`)
  }

  async function create() {
    TrayIcon.new({
      icon,
      tooltip,
      title,
      iconAsTemplate,
      menuOnLeftClick,
      menu: await Menu.new({
        items: menuItems.map((i) => i.item)
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
    <MenuBuilder bind:items={menuItems} on:itemClick={onItemClick} />
  </div>

  <div class="flex">
    <button class="btn" on:click={create} title="Creates the tray icon"
      >Create tray</button
    >
  </div>
</div>
