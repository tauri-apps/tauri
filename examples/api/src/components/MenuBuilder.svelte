<script>
  import { CheckMenuItem } from '@tauri-apps/api/menu'
  import MenuItemBuilder from './MenuItemBuilder.svelte'

  export let items = []

  function addItem(event) {
    items = [
      ...items,
      { item: event.detail.item, options: event.detail.options }
    ]
  }

  function itemIcon(item) {
    if (item.options.icon) {
      return 'i-ph-images-square'
    }
    if (item.item instanceof CheckMenuItem) {
      return 'i-ph-check-duotone'
    }
    if (item.options.item) {
      return 'i-ph-globe-stand'
    }
    return 'i-ph-chat-teardrop-text'
  }

  function itemToString(item) {
    // icon || check|normal || predefined
    return item.options.icon || item.options.text || item.options.item
  }
</script>

<div class="flex flex-col children:grow gap-2">
  <MenuItemBuilder on:new={addItem} />

  <div>
    {#each items as item}
      <div class="flex flex-row gap-1">
        <div class={itemIcon(item)} />
        <p>{itemToString(item)}</p>
      </div>
    {/each}
  </div>
</div>
