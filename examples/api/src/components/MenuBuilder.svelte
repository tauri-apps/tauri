<script lang="ts">
  import MenuItemBuilder from './MenuItemBuilder.svelte'
  import type {
    BuiltMenuItem,
    MenuItemClickDetail,
    MenuItemClickHandler
  } from '../types'

  let {
    items = $bindable<BuiltMenuItem[]>([]),
    itemClick
  }: { items?: BuiltMenuItem[]; itemClick: MenuItemClickHandler } = $props()

  function addItem(newItem: BuiltMenuItem) {
    items = [...items, newItem]
  }

  function onItemClick(detail: MenuItemClickDetail) {
    itemClick(detail)
  }

  function itemIcon(item: BuiltMenuItem) {
    if (item.kind === 'Icon' && item.options.icon) {
      return 'i-ph-images-square'
    }
    if (item.kind === 'Check') {
      return item.options.checked ? 'i-ph-check-duotone' : 'i-ph-square-duotone'
    }
    if (item.kind === 'Predefined' && item.options.item) {
      return 'i-ph-globe-stand'
    }
    return 'i-ph-chat-teardrop-text'
  }

  function itemToString(item: BuiltMenuItem) {
    return (
      // icon
      ('icon' in item.options && item.options.icon)
      // check|normal
      || ('text' in item.options && item.options.text)
      // predefined
      || ('item' in item.options && item.options.item)
    )
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
