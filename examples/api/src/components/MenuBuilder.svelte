<script>
  import { CheckMenuItem } from '@tauri-apps/api/menu'
  import MenuItemBuilder from './MenuItemBuilder.svelte'

  let { items = $bindable([]), itemClick } = $props()

  function addItem({ item, options }) {
    items = [...items, { item, options }]
  }

  function onItemClick(detail) {
    itemClick(detail)
  }

  function itemIcon(item) {
    if (item.options.icon) {
      return 'i-ph-images-square'
    }
    if (item.item instanceof CheckMenuItem) {
      return item.options.checked ? 'i-ph-check-duotone' : 'i-ph-square-duotone'
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
  <MenuItemBuilder newItem={addItem} itemClick={onItemClick} />

  <div>
    {#each items as item}
      <div class="flex flex-row gap-1 items-center">
        <div class={itemIcon(item)}></div>
        <p>{itemToString(item)}</p>
      </div>
    {/each}
  </div>
</div>
